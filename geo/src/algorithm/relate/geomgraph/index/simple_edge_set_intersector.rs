use super::super::{Edge, GeometryGraph};
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

// 简单边集交互器结构体
pub(crate) struct SimpleEdgeSetIntersector;

impl SimpleEdgeSetIntersector {
    // 创建一个新的简单边集交互器实例
    pub fn new() -> Self {
        SimpleEdgeSetIntersector
    }

    // 计算边之间的交集
    fn compute_intersects<F: GeoFloat>(
        &self,
        edge0: &Rc<RefCell<Edge<F>>>,                    // 边0的引用
        edge1: &Rc<RefCell<Edge<F>>>,                    // 边1的引用
        segment_intersector: &mut SegmentIntersector<F>, // 用于分段交集的对象
    ) {
        let edge0_coords_len = edge0.borrow().coords().len() - 1; // 边0的顶点数减1以获得段数
        let edge1_coords_len = edge1.borrow().coords().len() - 1; // 边1的顶点数减1以获得段数
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge1_coords_len {
                // 迭代两个边上的段，并计算交集
                segment_intersector.add_intersections(edge0, i0, edge1, i1); // 添加交集信息
            }
        }
    }
}

// 实现边集交互器接口
impl<F: GeoFloat> EdgeSetIntersector<F> for SimpleEdgeSetIntersector {
    // 计算集合内的所有边的交集
    fn compute_intersections_within_set(
        &self,
        graph: &GeometryGraph<F>,                        // 几何图
        check_for_self_intersecting_edges: bool,         // 是否检查自身交集
        segment_intersector: &mut SegmentIntersector<F>, // 用于分段交集的对象
    ) {
        let edges = graph.edges(); // 获取几何图中的所有边
        for edge0 in edges.iter() {
            for edge1 in edges.iter() {
                // 若检查自身交集或边不同，则计算交集
                if check_for_self_intersecting_edges || edge0.as_ptr() != edge1.as_ptr() {
                    self.compute_intersects(edge0, edge1, segment_intersector); // 计算交集
                }
            }
        }
    }

    // 计算两个集合之间所有边的交集
    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>, // 第一个几何图
        graph_1: &GeometryGraph<'a, F>, // 第二个几何图
        segment_intersector: &mut SegmentIntersector<F>, // 用于分段交集的对象
    ) {
        let edges_0 = graph_0.edges(); // 获取第一个几何图中的边
        let edges_1 = graph_1.edges(); // 获取第二个几何图中的边

        for edge0 in edges_0 {
            for edge1 in edges_1 {
                // 迭代两个集合中的边，并计算交集
                self.compute_intersects(edge0, edge1, segment_intersector); // 计算交集
            }
        }
    }
}
