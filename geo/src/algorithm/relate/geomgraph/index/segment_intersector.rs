use super::super::{CoordNode, Edge, LineIntersection, LineIntersector};
use crate::{Coord, GeoFloat, Line};

use std::cell::{Ref, RefCell};

/// 计算线段的交集，并将交点添加到包含这些线段的 [`Edge`] 中。
pub(crate) struct SegmentIntersector<F>
where
    F: GeoFloat,
{
    // 虽然 JTS 将其作为抽象留出——我们可以考虑将其硬编码为 RobustLineIntersector
    line_intersector: Box<dyn LineIntersector<F>>, // 线段交集计算器
    edges_are_from_same_geometry: bool,            // 边是否来自同一几何对象
    proper_intersection_point: Option<Coord<F>>,   // 正确的交点
    has_proper_interior_intersection: bool,        // 是否具有适当的内部交集
    boundary_nodes: Option<[Vec<CoordNode<F>>; 2]>, // 边界节点
}

impl<F> SegmentIntersector<F>
where
    F: GeoFloat,
{
    fn is_adjacent_segments(i1: usize, i2: usize) -> bool {
        // 判断两个段是否相邻
        let difference = if i1 > i2 { i1 - i2 } else { i2 - i1 };
        difference == 1
    }

    pub fn new(
        line_intersector: Box<dyn LineIntersector<F>>,
        edges_are_from_same_geometry: bool,
    ) -> SegmentIntersector<F> {
        // 创建新的 SegmentIntersector 对象
        SegmentIntersector {
            line_intersector,
            edges_are_from_same_geometry,
            has_proper_interior_intersection: false,
            proper_intersection_point: None,
            boundary_nodes: None,
        }
    }
    pub fn set_boundary_nodes(
        &mut self,
        boundary_nodes_0: Vec<CoordNode<F>>,
        boundary_nodes_1: Vec<CoordNode<F>>,
    ) {
        // 设置边界节点
        debug_assert!(
            self.boundary_nodes.is_none(),
            "应该只在几何体之间设置边界一次"
        );
        self.boundary_nodes = Some([boundary_nodes_0, boundary_nodes_1]);
    }

    pub fn has_proper_intersection(&self) -> bool {
        // 检查是否有正确的交集
        self.proper_intersection_point.is_some()
    }

    pub fn has_proper_interior_intersection(&self) -> bool {
        // 检查是否有正确的内部交集
        self.has_proper_interior_intersection
    }

    /// 一个平凡交集是看似自交实际上只是由相邻线段共享的点。注意，闭合边缘需要对
    /// 由起始和结束段共享的点进行特殊检查。
    fn is_trivial_intersection(
        &self,
        intersection: LineIntersection<F>,
        edge0: &RefCell<Edge<F>>,
        segment_index_0: usize,
        edge1: &RefCell<Edge<F>>,
        segment_index_1: usize,
    ) -> bool {
        if edge0.as_ptr() != edge1.as_ptr() {
            return false; // 线段不来自同一边
        }

        if matches!(intersection, LineIntersection::Collinear { .. }) {
            return false; // 如果交点是共线的
        }

        if Self::is_adjacent_segments(segment_index_0, segment_index_1) {
            return true; // 如果段相邻
        }

        let edge0 = edge0.borrow();
        if edge0.is_closed() {
            // 首尾坐标在一个环中相邻
            let max_segment_index = edge0.coords().len() - 1;
            if (segment_index_0 == 0 && segment_index_1 == max_segment_index)
                || (segment_index_1 == 0 && segment_index_0 == max_segment_index)
            {
                return true; // 首尾部分相邻
            }
        }

        false
    }

    pub fn add_intersections(
        &mut self,
        edge0: &RefCell<Edge<F>>,
        segment_index_0: usize,
        edge1: &RefCell<Edge<F>>,
        segment_index_1: usize,
    ) {
        // 避免线段虚假地“与自身相交”
        if edge0.as_ptr() == edge1.as_ptr() && segment_index_0 == segment_index_1 {
            return;
        }

        let line_0 = Line::new(
            edge0.borrow().coords()[segment_index_0],
            edge0.borrow().coords()[segment_index_0 + 1],
        );
        let line_1 = Line::new(
            edge1.borrow().coords()[segment_index_1],
            edge1.borrow().coords()[segment_index_1 + 1],
        );

        let intersection = self.line_intersector.compute_intersection(line_0, line_1);

        if intersection.is_none() {
            return; // 如果没有交集
        }
        let intersection = intersection.unwrap();

        if !self.edges_are_from_same_geometry {
            edge0.borrow_mut().mark_as_unisolated();
            edge1.borrow_mut().mark_as_unisolated();
        }
        if !self.is_trivial_intersection(
            intersection,
            edge0,
            segment_index_0,
            edge1,
            segment_index_1,
        ) {
            if self.edges_are_from_same_geometry || !intersection.is_proper() {
                // 在自节点的情况下，`edge0` 可能别名为 `edge1`，因此务必保证可变借用短暂且不重叠。
                edge0
                    .borrow_mut()
                    .add_intersections(intersection, line_0, segment_index_0);

                edge1
                    .borrow_mut()
                    .add_intersections(intersection, line_1, segment_index_1);
            }
            if let LineIntersection::SinglePoint {
                is_proper: true,
                intersection: intersection_coord,
            } = intersection
            {
                self.proper_intersection_point = Some(intersection_coord);

                if !self.is_boundary_point(&intersection_coord, &self.boundary_nodes) {
                    self.has_proper_interior_intersection = true
                }
            }
        }
    }

    fn is_boundary_point(
        &self,
        intersection: &Coord<F>,
        boundary_nodes: &Option<[Vec<CoordNode<F>>; 2]>,
    ) -> bool {
        // 判断交点是否为边界点
        match &boundary_nodes {
            Some(boundary_nodes) => boundary_nodes
                .iter()
                .flatten()
                .any(|node| intersection == node.coordinate()),
            None => false,
        }
    }
}
