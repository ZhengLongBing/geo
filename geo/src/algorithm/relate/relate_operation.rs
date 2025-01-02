use super::{EdgeEndBuilder, IntersectionMatrix};
use crate::dimensions::{Dimensions, HasDimensions};
use crate::relate::geomgraph::{
    index::SegmentIntersector,
    node_map::{NodeFactory, NodeMap},
    CoordNode, CoordPos, Edge, EdgeEnd, EdgeEndBundleStar, GeometryGraph, LabeledEdgeEndBundleStar,
    RobustLineIntersector,
};
use crate::CoordinatePosition;
use crate::{Coord, GeoFloat, GeometryCow};

use std::cell::RefCell;
use std::rc::Rc;

/// 计算描述两个几何体之间拓扑关系的 [`IntersectionMatrix`]。
///
/// `RelateOperation` 当前不支持包含重叠多边形的 [`GeometryCollection`]，并且在这种情况下可能会提供令人意外的结果。
///
/// 此实现严重依赖于 [`GeometryGraph`] 的功能。
///
/// 基于 [JTS 的 `RelateComputer` ，版本 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/RelateComputer.java)
pub(crate) struct RelateOperation<'a, F>
where
    F: GeoFloat,
{
    graph_a: GeometryGraph<'a, F>,
    graph_b: GeometryGraph<'a, F>,
    nodes: NodeMap<F, RelateNodeFactory>,
    line_intersector: RobustLineIntersector,
    isolated_edges: Vec<Rc<RefCell<Edge<F>>>>,
}

#[derive(PartialEq)]
pub(crate) struct RelateNodeFactory;
impl<F> NodeFactory<F> for RelateNodeFactory
where
    F: GeoFloat,
{
    type Node = (CoordNode<F>, EdgeEndBundleStar<F>);
    fn create_node(coordinate: Coord<F>) -> Self::Node {
        (CoordNode::new(coordinate), EdgeEndBundleStar::new())
    }
}

impl<'a, F> RelateOperation<'a, F>
where
    F: GeoFloat,
{
    pub(crate) fn new(graph_a: GeometryGraph<'a, F>, graph_b: GeometryGraph<'a, F>) -> Self {
        Self {
            graph_a,
            graph_b,
            nodes: NodeMap::new(),
            isolated_edges: vec![],
            line_intersector: RobustLineIntersector::new(),
        }
    }

    pub(crate) fn compute_intersection_matrix(&mut self) -> IntersectionMatrix {
        let mut intersection_matrix = IntersectionMatrix::empty_disjoint();

        use crate::BoundingRect;
        use crate::Intersects;
        match (
            self.graph_a.geometry().bounding_rect(),
            self.graph_b.geometry().bounding_rect(),
        ) {
            (Some(bounding_rect_a), Some(bounding_rect_b))
                if bounding_rect_a.intersects(&bounding_rect_b) => {}
            _ => {
                // 由于几何体不重叠，我们可以跳过大部分工作
                intersection_matrix
                    .compute_disjoint(self.graph_a.geometry(), self.graph_b.geometry());
                return intersection_matrix;
            }
        }

        // 由于拓扑改变是在节点处检查的，我们必须为每个交点创建一个节点。
        self.graph_a
            .compute_self_nodes(Box::new(self.line_intersector.clone()));
        self.graph_b
            .compute_self_nodes(Box::new(self.line_intersector.clone()));

        // 计算两个输入几何体的边之间的交点
        let segment_intersector = self
            .graph_a
            .compute_edge_intersections(&self.graph_b, Box::new(self.line_intersector.clone()));

        self.compute_intersection_nodes(0);
        self.compute_intersection_nodes(1);
        // 复制父几何体中节点的标记。这些将覆盖通过几何体之间的交点所确定的任何标记。
        self.copy_nodes_and_labels(0);
        self.copy_nodes_and_labels(1);
        // 完成仅为单一几何体提供标签的任何节点的标记
        self.label_isolated_nodes();
        // 如果找到适当的交点，我们可以在 IM 上设置一个下界。
        self.compute_proper_intersection_im(&segment_intersector, &mut intersection_matrix);
        // 现在处理不当的交点
        // （例如，在交点处一个或另一个几何体有一个顶点的情况）
        // 我们需要计算所有节点处的边图以确定 IM。
        let edge_end_builder = EdgeEndBuilder::new();
        let edge_ends_a: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_a.edges());
        self.insert_edge_ends(edge_ends_a);
        let edge_ends_b: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_b.edges());
        self.insert_edge_ends(edge_ends_b);

        let mut nodes = NodeMap::new();
        std::mem::swap(&mut self.nodes, &mut nodes);
        let labeled_node_edges = nodes
            .into_iter()
            .map(|(node, edges)| (node, edges.into_labeled(&self.graph_a, &self.graph_b)))
            .collect();

        // 计算“孤立”组件的标记
        //
        // 孤立组件是指在图中不接触任何其他组件的组件。
        //
        // 它们可以通过两个几何体的标签中只有一个非空元素来识别。
        //
        // 根据定义，我们只需要检查输入图中包含的组件，因为孤立组件不会被
        // 交点所形成的新组件所取代。
        self.label_isolated_edges(0, 1);
        self.label_isolated_edges(1, 0);

        debug!(
            "before update_intersection_matrix: {:?}",
            &intersection_matrix
        );
        self.update_intersection_matrix(labeled_node_edges, &mut intersection_matrix);

        intersection_matrix
    }

    fn insert_edge_ends(&mut self, edge_ends: Vec<EdgeEnd<F>>) {
        for edge_end in edge_ends {
            let (_node, edges) = self
                .nodes
                .insert_node_with_coordinate(*edge_end.coordinate());
            edges.insert(edge_end);
        }
    }

    fn compute_proper_intersection_im(
        &mut self,
        segment_intersector: &SegmentIntersector<F>,
        intersection_matrix: &mut IntersectionMatrix,
    ) {
        // 如果找到适当的交点，我们可以在 IM 上设置一个下界。
        let dim_a = self.graph_a.geometry().dimensions();
        let dim_b = self.graph_b.geometry().dimensions();

        let has_proper = segment_intersector.has_proper_intersection();
        let has_proper_interior = segment_intersector.has_proper_interior_intersection();

        debug_assert!(
            (dim_a != Dimensions::ZeroDimensional && dim_b != Dimensions::ZeroDimensional)
                || (!has_proper && !has_proper_interior)
        );

        match (dim_a, dim_b) {
            // 如果区域的边段正确相交，则区域必须正确重叠。
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("212101212")
                        .expect("error in hardcoded dimensions");
                }
            }

            // 如果线段正确地与区域的边段相交，则线段的内部与区域的边界相交。
            // 如果交点是一个正确的“内部”交点，则在内部也有一个内-内交点。
            // 请注意，这并不意味着线段的内部与区域的外部相交，因为可能有另一个区域组件包含线段的其余部分。
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("FFF0FFFF2")
                        .expect("error in hardcoded dimensions");
                }

                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("1FFFFF1FF")
                        .expect("error in hardcoded dimensions");
                }
            }

            (Dimensions::OneDimensional, Dimensions::TwoDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("F0FFFFFF2")
                        .expect("error in hardcoded dimensions");
                }

                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("1F1FFFFFF")
                        .expect("error in hardcoded dimensions");
                }
            }

            // 如果线字符串的边在内部点正确相交，我们能推断出的是内-内交点。
            // （我们不能推断出外部相交，因为几何体中的其他一些段可能覆盖交点附近的点）。
            // 确保该点是两个几何体的内部点很重要，因为在自相交几何体中，可能一个段上的正确交点也是另一个段的边界点。
            (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("0FFFFFFFF")
                        .expect("error in hardcoded dimensions");
                }
            }
            _ => {}
        }
    }

    /// 将所有节点从参数几何体复制到此图中。
    ///
    /// 参数几何体中的节点标签会覆盖先前计算的该参数索引的任何标签。
    /// （例如，一个节点可能是一个带有计算标签的交点节点，其标签为边界，但在原始参数几何体中，由于边界确定规则，该节点实际上在内部。
    fn copy_nodes_and_labels(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert_eq!(geom_index, 1);
            &self.graph_b
        };
        for graph_node in graph.nodes_iter() {
            let new_node = self
                .nodes
                .insert_node_with_coordinate(*graph_node.coordinate());

            let on_position = graph_node
                .label()
                .on_position(geom_index)
                .expect("node should have been labeled by now");

            new_node.0.set_label_on_position(geom_index, on_position);
        }
    }

    /// 为几何体边上的所有交点插入节点。
    ///
    /// 如果创建的节点没有标签，标记节点为相同的边标签。
    /// 这允许由自交或互交创建的节点被标记。
    ///
    /// 端点节点在插入时已标记。
    fn compute_intersection_nodes(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert_eq!(geom_index, 1);
            &self.graph_b
        };

        for edge in graph.edges() {
            let edge = edge.borrow();

            let edge_position = edge.label().on_position(geom_index);
            for edge_intersection in edge.edge_intersections() {
                let (new_node, _edges) = self
                    .nodes
                    .insert_node_with_coordinate(edge_intersection.coordinate());

                if edge_position == Some(CoordPos::OnBoundary) {
                    new_node.set_label_boundary(geom_index);
                } else if new_node.label().is_empty(geom_index) {
                    new_node.set_label_on_position(geom_index, CoordPos::Inside);
                }
            }
        }
    }

    fn update_intersection_matrix(
        &self,
        labeled_node_edges: Vec<(CoordNode<F>, LabeledEdgeEndBundleStar<F>)>,
        intersection_matrix: &mut IntersectionMatrix,
    ) {
        debug!(
            "before updated_intersection_matrix(isolated_edges): {:?}",
            intersection_matrix
        );
        for isolated_edge in &self.isolated_edges {
            let edge = isolated_edge.borrow();
            Edge::<F>::update_intersection_matrix(edge.label(), intersection_matrix);
            debug!(
                "after isolated_edge update_intersection_matrix: {:?}, (isolated_edge: {:?}, label: {:?})",
                intersection_matrix,
                edge,
                edge.label()
            );
        }

        for (node, edges) in labeled_node_edges.iter() {
            node.update_intersection_matrix(intersection_matrix);
            edges.update_intersection_matrix(intersection_matrix);
        }
    }

    /// 通过计算其标签并将其添加到孤立边列表中来处理孤立边。
    ///
    /// 根据定义，“孤立”边保证不触摸目标的边界（因为如果它们触碰了边界，它们将导致计算一个交点，因此不会保持孤立）。
    fn label_isolated_edges(&mut self, this_index: usize, target_index: usize) {
        let (this_graph, target_graph) = if this_index == 0 {
            (&self.graph_a, &self.graph_b)
        } else {
            (&self.graph_b, &self.graph_a)
        };

        for edge in this_graph.edges() {
            let mut mut_edge = edge.borrow_mut();
            if mut_edge.is_isolated() {
                Self::label_isolated_edge(&mut mut_edge, target_index, target_graph.geometry());
                self.isolated_edges.push(edge.clone());
            }
        }
    }

    /// 标记图中孤立边与目标几何体的关系。
    /// 如果目标的维度是 2 或 1，边可以在内部或外部。
    /// 如果目标的维度是 0，边必须在外部
    fn label_isolated_edge(edge: &mut Edge<F>, target_index: usize, target: &GeometryCow<F>) {
        if target.dimensions() > Dimensions::ZeroDimensional {
            // 一个孤立边不跨越任何边界，因此它要么完全在几何体内部，要么完全在外部。 因此，我们可以使用边上的任何一点来推断整个边的位置。
            let coord = edge.coords().first().expect("can't create empty edge");
            let position = target.coordinate_position(coord);
            edge.label_mut().set_all_positions(target_index, position);
        } else {
            edge.label_mut()
                .set_all_positions(target_index, CoordPos::Outside);
        }
    }

    /// 孤立节点是标签不完整的节点（例如，一个几何体的位置为空）。
    /// 这是因为一个图中的节点在添加到节点列表的初始过程中没有与另一个图中的节点相交，因此没有完全被标记。
    /// 为了完成标记，我们需要检查位于边内部和区域内部的节点。
    fn label_isolated_nodes(&mut self) {
        let geometry_a = self.graph_a.geometry();
        let geometry_b = self.graph_b.geometry();
        for (node, _edges) in self.nodes.iter_mut() {
            let label = node.label();
            // 孤立节点的标签中应始终至少包含一个几何体
            debug_assert!(label.geometry_count() > 0, "node with empty label found");
            if node.is_isolated() {
                if label.is_empty(0) {
                    Self::label_isolated_node(node, 0, geometry_a)
                } else {
                    Self::label_isolated_node(node, 1, geometry_b)
                }
            }
        }
    }

    fn label_isolated_node(
        node: &mut CoordNode<F>,
        target_index: usize,
        geometry: &GeometryCow<F>,
    ) {
        let position = geometry.coordinate_position(node.coordinate());
        node.label_mut().set_all_positions(target_index, position);
    }
}

#[cfg(test)]
mod test {
    use crate::Relate;

    use super::*;
    use geo_types::{line_string, polygon, Geometry};
    use std::str::FromStr;

    #[test]
    fn test_disjoint() {
        let square_a: Geometry = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry = polygon![
            (x: 55., y: 55.),
            (x: 50., y: 60.),
            (x: 60., y: 60.),
            (x: 60., y: 55.),
            (x: 55., y: 55.),
        ]
        .into();

        assert_eq!(
            square_a.relate(&square_b),
            IntersectionMatrix::from_str("FF2FF1212").unwrap()
        );
    }

    #[test]
    fn test_a_contains_b() {
        let square_a: Geometry = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 10.),
            (x: 10., y: 10.),
            (x: 10., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        assert_eq!(
            square_a.relate(&square_b),
            IntersectionMatrix::from_str("212FF1FF2").unwrap()
        );
    }

    #[test]
    fn test_a_overlaps_b() {
        let square_a: Geometry = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 30.),
            (x: 30., y: 30.),
            (x: 30., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        assert_eq!(
            square_a.relate(&square_b),
            IntersectionMatrix::from_str("212101212").unwrap()
        );
    }
    #[test]
    fn equals() {
        let square_a = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ];
        let square_b = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ];
        let polyrelation = square_a.relate(&square_b);

        // 相同，但坐标顺序不同
        let ls1 = line_string![(x: 1.0, y: 1.0), (x: 2.0, y: 2.0)];
        let ls2 = line_string![(x: 2.0, y: 2.0), (x: 1.0, y: 1.0)];
        let lsrelation = ls1.relate(&ls2);

        let de9im_eq = "T*F**FFF*";
        assert!(polyrelation.matches(de9im_eq).unwrap());
        assert!(lsrelation.matches(de9im_eq).unwrap());
    }
}
