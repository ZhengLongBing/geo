// 使用 crate 中的 LineIntersection 模块
pub(crate) use crate::LineIntersection;
use crate::{Coord, GeoFloat, Line};

/// 定义一个用于计算两条直线交点的接口
pub(crate) trait LineIntersector<F: GeoFloat> {
    /// 计算两条直线 l1 和 l2 的交点，如果有交点则返回 `LineIntersection`，否则返回 `None`
    fn compute_intersection(&mut self, l1: Line<F>, l2: Line<F>) -> Option<LineIntersection<F>>;
}
