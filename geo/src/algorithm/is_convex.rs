use crate::kernels::*;
use crate::{Coord, GeoNum, LineString};

/// 用于测试[ `LineString` ]的凸性。
/// 当封闭的`LineString`包围一个[凸集]时，被称为_凸_。
/// 如果三个连续顶点中没有共线的，它被称为_严格凸_。
/// 所有顶点在同一条直线上称为_共线_。
///
/// # 备注
///
/// - 共线性不要求`LineString`是封闭的，但其他谓词是这样。
///
/// - 这个定义与[多边形的凸性][凸集]的概念密切相关。
///   特别地，一个[`多边形`](crate::Polygon)是凸的，当且仅当它的`外部边界`是凸的，
///   且`内部`为空。
///
/// - [`ConvexHull`]算法总是返回一个严格凸的`LineString`，除非输入为空或共线。
///   [`graham_hull`]算法提供包含共线点的选项，生成一个（可能非严格）的凸`LineString`。
///
/// # 极端情况
///
/// - 一个空的`LineString`的凸性和共线性是_未指定的_，不应倚赖于此。
///
/// - 一个封闭的`LineString`最多包含三个坐标（包括可能重复的第一个坐标）是
///   既凸又共线的。然而，严格凸性是_未指定的_，不应被倚赖。
///
/// [convex combination]: //en.wikipedia.org/wiki/Convex_combination
/// [convex set]: //en.wikipedia.org/wiki/Convex_set
/// [`ConvexHull`]: crate::ConvexHull
/// [`graham_hull`]: crate::convex_hull::graham_hull
pub trait IsConvex {
    /// 测试并获取形状是否为凸的方向。
    /// 如果`allow_collinear`为真，测试严格凸性，只接受提供的特定方向。
    ///
    /// 如果以下任何一个成立，返回值为`None`：
    ///
    /// 1. 形状不是凸的
    ///
    /// 2. 形状不是严格凸的，并且`allow_collinear`为假
    ///
    /// 3. 指定了一个方向，且存在三个连续顶点既非共线也不是在指定方向。
    ///
    /// 在所有其他情况下，返回值是形状的方向，或者如果所有顶点在同一条线上，
    /// 返回`Orientation::Collinear`。
    ///
    /// **注意。** 这个谓词不同于`is_collinear`，因为后者要求输入是封闭的。
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation>;

    /// 测试形状是否为凸的。
    fn is_convex(&self) -> bool {
        self.convex_orientation(true, None).is_some()
    }

    /// 测试形状是否为逆时针方向的凸。
    fn is_ccw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::CounterClockwise))
            .is_some()
    }

    /// 测试形状是否为顺时针方向的凸。
    fn is_cw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::Clockwise))
            .is_some()
    }

    /// 测试形状是否为严格凸。
    fn is_strictly_convex(&self) -> bool {
        self.convex_orientation(false, None).is_some()
    }

    /// 测试形状是否为逆时针方向的严格凸。
    fn is_strictly_ccw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::CounterClockwise))
            == Some(Orientation::CounterClockwise)
    }

    /// 测试形状是否为顺时针方向的严格凸。
    fn is_strictly_cw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::Clockwise)) == Some(Orientation::Clockwise)
    }

    /// 测试形状是否在一条线上。
    fn is_collinear(&self) -> bool;
}

impl<T: GeoNum> IsConvex for LineString<T> {
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation> {
        if !self.is_closed() || self.0.is_empty() {
            None
        } else {
            is_convex_shaped(&self.0[1..], allow_collinear, specific_orientation)
        }
    }

    fn is_collinear(&self) -> bool {
        self.0.is_empty()
            || is_convex_shaped(&self.0[1..], true, Some(Orientation::Collinear)).is_some()
    }
}

/// 验证一系列坐标是否为凸的工具函数。
/// 它验证对于所有`0 <= i < n`，在位置`i`，`i+1`，`i+2`（模`n`）的顶点具有相同的方向，
/// 可选择性地接受共线的三元组，并期望特定的方向。
/// 除非所有东西都是共线的，否则输出为`None`或唯一的非共线方向。
fn is_convex_shaped<T>(
    coords: &[Coord<T>],
    allow_collinear: bool,
    specific_orientation: Option<Orientation>,
) -> Option<Orientation>
where
    T: GeoNum,
{
    let n = coords.len();

    let orientation_at = |i: usize| {
        let coord = coords[i];
        let next = coords[(i + 1) % n];
        let nnext = coords[(i + 2) % n];
        (i, T::Ker::orient2d(coord, next, nnext))
    };

    let find_first_non_collinear = (0..n).map(orientation_at).find_map(|(i, orientation)| {
        match orientation {
            Orientation::Collinear => {
                // 如果接受共线，继续，否则停止。
                if allow_collinear {
                    None
                } else {
                    Some((i, orientation))
                }
            }
            _ => Some((i, orientation)),
        }
    });

    let (i, first_non_collinear) = if let Some((i, orientation)) = find_first_non_collinear {
        match orientation {
            Orientation::Collinear => {
                // 只有在!allow_collinear时发生
                assert!(!allow_collinear);
                return None;
            }
            _ => (i, orientation),
        }
    } else {
        // 空的或所有东西都是共线并且被允许。
        return Some(Orientation::Collinear);
    };

    // 如果期望一个特定的方向，则只接受该方向。
    if let Some(req_orientation) = specific_orientation {
        if req_orientation != first_non_collinear {
            return None;
        }
    }

    // 现在，我们在其余坐标中预期一个固定的方向。循环检查每个都匹配它。
    if ((i + 1)..n)
        .map(orientation_at)
        .all(|(_, orientation)| match orientation {
            Orientation::Collinear => allow_collinear,
            orientation => orientation == first_non_collinear,
        })
    {
        Some(first_non_collinear)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::line_string;

    #[test]
    fn test_corner_cases() {
        // 此测试仅用于确保没有因越界访问而导致恐慌。
        let empty: LineString = line_string!();
        assert!(empty.is_collinear());
        assert!(!empty.is_convex());
        assert!(!empty.is_strictly_ccw_convex());

        let one = line_string![(x: 0., y: 0.)];
        assert!(one.is_collinear());
        assert!(one.is_convex());
        assert!(one.is_cw_convex());
        assert!(one.is_ccw_convex());
        assert!(one.is_strictly_convex());
        assert!(!one.is_strictly_ccw_convex());
        assert!(!one.is_strictly_cw_convex());

        let one_rep = line_string![(x: 0, y: 0), (x: 0, y: 0)];
        assert!(one_rep.is_collinear());
        assert!(one_rep.is_convex());
        assert!(one_rep.is_cw_convex());
        assert!(one_rep.is_ccw_convex());
        assert!(!one_rep.is_strictly_convex());
        assert!(!one_rep.is_strictly_ccw_convex());
        assert!(!one_rep.is_strictly_cw_convex());

        let mut two = line_string![(x: 0, y: 0), (x: 1, y: 1)];
        assert!(two.is_collinear());
        assert!(!two.is_convex());

        two.close();
        assert!(two.is_cw_convex());
        assert!(two.is_ccw_convex());
        assert!(!two.is_strictly_convex());
        assert!(!two.is_strictly_ccw_convex());
        assert!(!two.is_strictly_cw_convex());
    }
}
