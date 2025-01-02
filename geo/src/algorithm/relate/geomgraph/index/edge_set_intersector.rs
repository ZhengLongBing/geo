use super::super::{Edge, GeometryGraph};
use super::SegmentIntersector;
use crate::{Coord, GeoFloat};

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) trait EdgeSetIntersector<F: GeoFloat> {
    /// 计算集合内边的所有交集，将这些交集记录在相交的边上。
    ///
    /// `edges`: 要检查的边集合，会被修改以记录任何交集。
    /// `check_for_self_intersecting_edges`: 如果为 false，则不会检查边与自身的交集。
    /// `segment_intersector`: 要使用的 SegmentIntersector
    fn compute_intersections_within_set(
        &self,
        graph: &GeometryGraph<F>,                        // 图形图对象
        check_for_self_intersecting_edges: bool,         // 是否检查自身交集标志
        segment_intersector: &mut SegmentIntersector<F>, // 用于处理分段交集的对象
    );

    /// 计算两个边集合之间的所有交集，将这些交集记录在相交的边上。
    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>, // 第一个图形图对象
        graph_1: &GeometryGraph<'a, F>, // 第二个图形图对象
        segment_intersector: &mut SegmentIntersector<F>, // 用于处理分段交集的对象
    );
}
