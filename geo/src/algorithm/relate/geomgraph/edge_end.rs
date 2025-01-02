use super::{CoordNode, Edge, Label, Quadrant};
use crate::{coord, Coord, GeoFloat};

use std::cell::RefCell;
use std::fmt;

/// 模型化一个连接到节点的边的末端。
///
/// EdgeEnd 的方向是由从初始点到下一个点的射线方向决定的。
///
/// EdgeEnd 可以通过它们的 EdgeEndKey 来比较，根据"相对于 x 轴来说，a 的角度比 b 大"的顺序。
///
/// 此排序用于在一个节点周围对 EdgeEnd 进行排序。
///
/// 这是基于 [JTS 的 EdgeEnd 版本 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeEnd.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEnd<F>
where
    F: GeoFloat,
{
    label: Label,
    key: EdgeEndKey<F>,
}

#[derive(Clone)]
pub(crate) struct EdgeEndKey<F>
where
    F: GeoFloat,
{
    // 边的两个坐标
    coord_0: Coord<F>,
    coord_1: Coord<F>,
    // 差值坐标
    delta: Coord<F>,
    // 象限
    quadrant: Option<Quadrant>,
}

impl<F: GeoFloat> fmt::Debug for EdgeEndKey<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgeEndKey")
            .field(
                "coords",
                &format!("{:?} -> {:?}", &self.coord_0, &self.coord_1),
            )
            .field("quadrant", &self.quadrant)
            .finish()
    }
}

impl<F> EdgeEnd<F>
where
    F: GeoFloat,
{
    // 创建一个新的 EdgeEnd
    pub fn new(coord_0: Coord<F>, coord_1: Coord<F>, label: Label) -> EdgeEnd<F> {
        let delta = coord_1 - coord_0;
        let quadrant = Quadrant::new(delta.x, delta.y);
        EdgeEnd {
            label,
            key: EdgeEndKey {
                coord_0,
                coord_1,
                delta,
                quadrant,
            },
        }
    }

    // 获取标签
    pub fn label(&self) -> &Label {
        &self.label
    }

    // 获取可修改的标签
    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    // 获取起始坐标
    pub fn coordinate(&self) -> &Coord<F> {
        &self.key.coord_0
    }

    // 获取 EdgeEndKey
    pub fn key(&self) -> &EdgeEndKey<F> {
        &self.key
    }
}

impl<F> std::cmp::Eq for EdgeEndKey<F> where F: GeoFloat {}

impl<F> std::cmp::PartialEq for EdgeEndKey<F>
where
    F: GeoFloat,
{
    // 判等
    fn eq(&self, other: &EdgeEndKey<F>) -> bool {
        self.delta == other.delta
    }
}

impl<F> std::cmp::PartialOrd for EdgeEndKey<F>
where
    F: GeoFloat,
{
    // 部分比较
    fn partial_cmp(&self, other: &EdgeEndKey<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F> std::cmp::Ord for EdgeEndKey<F>
where
    F: GeoFloat,
{
    // 全比较
    fn cmp(&self, other: &EdgeEndKey<F>) -> std::cmp::Ordering {
        self.compare_direction(other)
    }
}

impl<F> EdgeEndKey<F>
where
    F: GeoFloat,
{
    // 比较方向
    pub(crate) fn compare_direction(&self, other: &EdgeEndKey<F>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.delta == other.delta {
            return Ordering::Equal;
        }

        match (self.quadrant, other.quadrant) {
            (Some(q1), Some(q2)) if q1 > q2 => Ordering::Greater,
            (Some(q1), Some(q2)) if q1 < q2 => Ordering::Less,
            _ => {
                use crate::kernels::{Kernel, Orientation};
                match F::Ker::orient2d(other.coord_0, other.coord_1, self.coord_1) {
                    Orientation::Clockwise => Ordering::Less,
                    Orientation::CounterClockwise => Ordering::Greater,
                    Orientation::Collinear => Ordering::Equal,
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // 测试排序
    fn test_ord() {
        let fake_label = Label::empty_line_or_point();
        let edge_end_1 = EdgeEnd::new(Coord::zero(), coord! { x: 1.0, y: 1.0 }, fake_label.clone());
        let edge_end_2 = EdgeEnd::new(Coord::zero(), coord! { x: 1.0, y: 1.0 }, fake_label.clone());
        assert_eq!(
            edge_end_1.key().cmp(edge_end_2.key()),
            std::cmp::Ordering::Equal
        );

        // edge_end_3 顺时针方向在 edge_end_1 的右边
        let edge_end_3 = EdgeEnd::new(Coord::zero(), coord! { x: 1.0, y: -1.0 }, fake_label);
        assert_eq!(
            edge_end_1.key().cmp(edge_end_3.key()),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            edge_end_3.key().cmp(edge_end_1.key()),
            std::cmp::Ordering::Greater
        );
    }
}
