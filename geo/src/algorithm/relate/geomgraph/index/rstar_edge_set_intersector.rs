use super::super::{Edge, GeometryGraph};
use super::{EdgeSetIntersector, Segment, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

use rstar::{RTree, RTreeNum};

pub(crate) struct RStarEdgeSetIntersector;

impl<F> EdgeSetIntersector<F> for RStarEdgeSetIntersector
where
    F: GeoFloat + RTreeNum,
{
    fn compute_intersections_within_set(
        &self,
        graph: &GeometryGraph<F>,
        check_for_self_intersecting_edges: bool, // 检查自身交集标志
        segment_intersector: &mut SegmentIntersector<F>, // 用于处理分段交集的对象
    ) {
        let edges = graph.edges(); // 获取所有的边对象

        let tree = graph.get_or_build_tree(); // 获取或构建 R* 树
        for (segment_0, segment_1) in tree.intersection_candidates_with_other_tree(&tree) {
            // 遍历可能的相交线段对
            if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
                // 检查是否需要忽略自身交集
                let edge_0 = &edges[segment_0.edge_idx]; // 获取第一个边
                let edge_1 = &edges[segment_1.edge_idx]; // 获取第二个边
                segment_intersector.add_intersections(
                    edge_0,
                    segment_0.segment_idx,
                    edge_1,
                    segment_1.segment_idx,
                ); // 添加相交信息
            }
        }
    }

    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>, // 第一个几何图
        graph_1: &GeometryGraph<'a, F>, // 第二个几何图
        segment_intersector: &mut SegmentIntersector<F>, // 用于处理分段交集的对象
    ) {
        let edges_0 = graph_0.edges(); // 获取第一个几何图的边
        let edges_1 = graph_1.edges(); // 获取第二个几何图的边

        let tree_0 = graph_0.get_or_build_tree(); // 获取或构建第一个 R* 树
        let tree_1 = graph_1.get_or_build_tree(); // 获取或构建第二个 R* 树

        for (segment_0, segment_1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            // 遍历可能的相交线段对
            let edge_0 = &edges_0[segment_0.edge_idx]; // 获取第一个边
            let edge_1 = &edges_1[segment_1.edge_idx]; // 获取第二个边
            segment_intersector.add_intersections(
                edge_0,
                segment_0.segment_idx,
                edge_1,
                segment_1.segment_idx,
            ); // 添加相交信息
        }
    }
}
