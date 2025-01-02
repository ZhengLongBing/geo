mod i_overlay_integration;
#[cfg(test)]
mod tests;

use i_overlay_integration::convert::{multi_polygon_from_shapes, ring_to_shape_path};
use i_overlay_integration::BoolOpsCoord;
pub use i_overlay_integration::BoolOpsNum;

use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};
use crate::winding_order::{Winding, WindingOrder};

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::overlay::FloatOverlay;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;

/// 几何体上的布尔运算。
///
/// 布尔运算是将几何体视为二维平面子集的集合运算。支持的运算有：交集、并集、对称差（异或），以及一对二维几何体的差集和自我剪裁一个一维几何体。
///
/// 这些操作是基于[`Polygon`]和[`MultiPolygon`]几何体实现的。
///
/// # 有效性
///
/// 请注意，这些操作只有在*有效*几何体上才是严格定义的。然而，只要多边形的内部位于其对应的外部内，通常实现都可以正常工作。
///
/// 算法会处理并忽略面积为0的退化二维几何体。特别是，与一个空几何体的`并集`操作应能去除退化并修复无效的多边形，只要内外条件如上所述得以满足。
///
/// # 性能
///
/// 对于大量[`Polygon`]或[`MultiPolygon`]的并集操作，使用[`unary_union`]将获得更好的性能。
pub trait BooleanOps {
    type Scalar: BoolOpsNum;

    /// 几何体的外部和内部环。
    ///
    /// 环的顺序并不重要，因为拓扑算法通过计数交叉点来确定多边形的内部和外部。
    ///
    /// 要求环来自有效几何体，环之间不得重叠。
    /// 对于MultiPolygon，这要求其任何多边形的内部不可以重叠。
    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>>;

    fn boolean_op(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        op: OpType,
    ) -> MultiPolygon<Self::Scalar> {
        let subject = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let clip = other.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.overlay(&clip, op.into(), FillRule::EvenOdd);
        multi_polygon_from_shapes(shapes)
    }

    /// 返回`self`和`other`共享的重叠区域。
    fn intersection(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }

    /// 将`self`和`other`的区域合并成一个单一的几何体，消除重叠并合并边界。
    fn union(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }

    /// 位于`self`或`other`中但不在两个之中的区域。
    fn xor(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }

    /// `self`中不存在的区域。
    fn difference(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }

    /// 使用self剪裁一维几何体。
    ///
    /// 如果`invert`为false，返回位于`self`内的`ls`部分（称为集合论交集），否则返回差异（`ls - self`）。
    fn clip(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        let subject: Vec<Vec<_>> = multi_line_string
            .iter()
            .map(|line_string| line_string.coords().map(|c| BoolOpsCoord(*c)).collect())
            .collect();

        let clip = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();

        let clip_rule = ClipRule {
            invert,
            boundary_included: true,
        };
        let paths = subject.clip_by(&clip, FillRule::EvenOdd, clip_rule);
        i_overlay_integration::convert::multi_line_string_from_paths(paths)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

/// 高效的合并多个相邻或重叠的几何体的[并集](BooleanOps::union)
///
/// 这通常比将一堆几何体一个一个地合并要快得多。
///
/// 注意：几何体的方向可以是任意的，但顺序必须一致，而且每个多边形的内部环必须与其外部方向相反。
///
/// 参见 [Orient] 获取更多信息。
///
/// [Orient]: crate::algorithm::orient::Orient
///
/// # 参数
///
/// `boppables`: 要合并在一起的`Polygon`或`MultiPolygons`的集合。
///
/// 返回所有输入的并集。
///
/// # 例子
///
/// ```
/// use geo::algorithm::unary_union;
/// use geo::wkt;
///
/// let right_piece = wkt!(POLYGON((4. 0.,4. 4.,8. 4.,8. 0.,4. 0.)));
/// let left_piece = wkt!(POLYGON((0. 0.,0. 4.,4. 4.,4. 0.,0. 0.)));
///
/// // 不接触左右两部分
/// let separate_piece = wkt!(POLYGON((14. 10.,14. 14.,18. 14.,18. 10.,14. 10.)));
///
/// let polygons = vec![left_piece, separate_piece, right_piece];
/// let actual_output = unary_union(&polygons);
///
/// let expected_output = wkt!(MULTIPOLYGON(
///     // 左右部分已合并
///     ((0. 0., 0. 4., 8. 4., 8. 0.,  0. 0.)),
///     // 独立部分仍保持独立
///     ((14. 10., 14. 14., 18. 14.,18. 10., 14. 10.))
/// ));
/// assert_eq!(actual_output, expected_output);
/// ```
pub fn unary_union<'a, B: BooleanOps + 'a>(
    boppables: impl IntoIterator<Item = &'a B>,
) -> MultiPolygon<B::Scalar> {
    let mut winding_order: Option<WindingOrder> = None;
    let subject = boppables
        .into_iter()
        .flat_map(|boppable| {
            let rings = boppable.rings();
            rings
                .map(|ring| {
                    if winding_order.is_none() {
                        winding_order = ring.winding_order();
                    }
                    ring_to_shape_path(ring)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let fill_rule = if winding_order == Some(WindingOrder::Clockwise) {
        FillRule::Positive
    } else {
        FillRule::Negative
    };

    let shapes = FloatOverlay::with_subj(&subject).overlay(OverlayRule::Subject, fill_rule);
    multi_polygon_from_shapes(shapes)
}

impl<T: BoolOpsNum> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        std::iter::once(self.exterior()).chain(self.interiors())
    }
}

impl<T: BoolOpsNum> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        self.iter().flat_map(BooleanOps::rings)
    }
}
