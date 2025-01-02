use super::{
    index::{
        EdgeSetIntersector, RStarEdgeSetIntersector, Segment, SegmentIntersector,
        SimpleEdgeSetIntersector,
    },
    CoordNode, CoordPos, Direction, Edge, Label, LineIntersector, PlanarGraph, TopologyPosition,
};

use crate::HasDimensions;
use crate::{Coord, GeoFloat, GeometryCow, Line, LineString, Point, Polygon};

use rstar::{RTree, RTreeNum};
use std::cell::RefCell;
use std::rc::Rc;

/// [`IntersectionMatrix`](crate::algorithm::relate::IntersectionMatrix) 的计算依赖于一种称为“拓扑图”的结构。拓扑图包含与 [`Geometry`](crate::Geometry) 的节点和线段对应的节点（CoordNode）和边（Edge）。图中的每个节点和边都标有其相对于源几何体的位置。
///
/// 注意，自相交点并不要求是顶点。因此，为了获得正确的拓扑图，必须在构建几何图之前对其进行自节点化。
///
/// 拓扑图支持两个基本操作：
/// - 计算单个图的所有边和节点之间的交集
/// - 计算两个不同图的边和节点之间的交集
///
/// GeometryGraph 基于 [JTS's `GeomGraph` as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/GeometryGraph.java)
#[derive(Clone)]
pub struct GeometryGraph<'a, F>
where
    F: GeoFloat,
{
    arg_index: usize,                      // 参数索引
    parent_geometry: GeometryCow<'a, F>,   // 父几何体
    tree: Option<Rc<RTree<Segment<F>>>>,   // 线段R树
    use_boundary_determination_rule: bool, // 使用边界判定规则
    has_computed_self_nodes: bool,         // 是否已计算自节点
    planar_graph: PlanarGraph<F>,          // 平面图
}

/// PlanarGraph 委托
///
/// 在用 Java 编写的 JTS 中，GeometryGraph 继承自 PlanarGraph。在 Rust 中我们使用组合和委托实现相同的效果。
impl<F> GeometryGraph<'_, F>
where
    F: GeoFloat,
{
    pub(crate) fn set_tree(&mut self, tree: Rc<RTree<Segment<F>>>) {
        self.tree = Some(tree);
    }

    pub(crate) fn get_or_build_tree(&self) -> Rc<RTree<Segment<F>>> {
        self.tree
            .clone()
            .unwrap_or_else(|| Rc::new(self.build_tree()))
    }

    pub(crate) fn build_tree(&self) -> RTree<Segment<F>> {
        let segments: Vec<Segment<F>> = self
            .edges()
            .iter()
            .enumerate()
            .flat_map(|(edge_idx, edge)| {
                let edge = RefCell::borrow(edge);
                let start_of_final_segment: usize = edge.coords().len() - 1; // 最后的线段起始点
                (0..start_of_final_segment).map(move |segment_idx| {
                    let p1 = edge.coords()[segment_idx];
                    let p2 = edge.coords()[segment_idx + 1];
                    Segment::new(edge_idx, segment_idx, p1, p2)
                })
            })
            .collect();
        RTree::bulk_load(segments)
    }

    pub(crate) fn assert_eq_graph(&self, other: &Self) {
        assert_eq!(self.arg_index, other.arg_index);
        assert_eq!(
            self.use_boundary_determination_rule,
            other.use_boundary_determination_rule
        );
        assert_eq!(self.parent_geometry, other.parent_geometry);
        self.planar_graph.assert_eq_graph(&other.planar_graph);
    }

    pub(crate) fn clone_for_arg_index(&self, arg_index: usize) -> Self {
        debug_assert!(self.has_computed_self_nodes, "应在计算自节点后调用");
        let planar_graph = self
            .planar_graph
            .clone_for_arg_index(self.arg_index, arg_index);
        Self {
            arg_index,
            parent_geometry: self.parent_geometry.clone(),
            tree: self.tree.clone(),
            use_boundary_determination_rule: self.use_boundary_determination_rule,
            has_computed_self_nodes: true,
            planar_graph,
        }
    }

    pub(crate) fn edges(&self) -> &[Rc<RefCell<Edge<F>>>] {
        self.planar_graph.edges()
    }

    pub(crate) fn insert_edge(&mut self, edge: Edge<F>) {
        self.planar_graph.insert_edge(edge)
    }

    pub(crate) fn is_boundary_node(&self, coord: Coord<F>) -> bool {
        self.planar_graph.is_boundary_node(self.arg_index, coord)
    }

    pub(crate) fn add_node_with_coordinate(&mut self, coord: Coord<F>) -> &mut CoordNode<F> {
        self.planar_graph.add_node_with_coordinate(coord)
    }

    pub(crate) fn nodes_iter(&self) -> impl Iterator<Item = &CoordNode<F>> {
        self.planar_graph.nodes.iter()
    }
}

impl<'a, F> GeometryGraph<'a, F>
where
    F: GeoFloat + RTreeNum,
{
    pub(crate) fn new(arg_index: usize, parent_geometry: GeometryCow<'a, F>) -> Self {
        let mut graph = GeometryGraph {
            arg_index,
            parent_geometry,
            use_boundary_determination_rule: true,
            tree: None,
            has_computed_self_nodes: false,
            planar_graph: PlanarGraph::new(),
        };
        graph.add_geometry(&graph.parent_geometry.clone());
        graph
    }

    pub(crate) fn geometry(&self) -> &GeometryCow<F> {
        &self.parent_geometry
    }

    /// 确定多几何体中多次出现的组件（节点或边）是在几何体的边界还是内部
    pub fn determine_boundary(boundary_count: usize) -> CoordPos {
        // 目前，我们只支持 SFS 的 "Mod-2 Rule"
        // 如果我们想支持其他边界规则，可以使其配置化。
        if boundary_count % 2 == 1 {
            CoordPos::OnBoundary
        } else {
            CoordPos::Inside
        }
    }

    fn boundary_nodes(&self) -> impl Iterator<Item = &CoordNode<F>> {
        self.planar_graph.boundary_nodes(self.arg_index)
    }

    pub(crate) fn add_geometry(&mut self, geometry: &GeometryCow<F>) {
        if geometry.is_empty() {
            return;
        }
        match geometry {
            GeometryCow::Line(line) => self.add_line(line),
            GeometryCow::Rect(rect) => {
                // PERF: 避免这种转换/克隆？
                self.add_polygon(&rect.to_polygon());
            }
            GeometryCow::Point(point) => {
                self.add_point(point);
            }
            GeometryCow::Polygon(polygon) => self.add_polygon(polygon),
            GeometryCow::Triangle(triangle) => {
                // PERF: 避免这种转换/克隆？
                self.add_polygon(&triangle.to_polygon());
            }
            GeometryCow::LineString(line_string) => self.add_line_string(line_string),
            GeometryCow::MultiPoint(multi_point) => {
                for point in &multi_point.0 {
                    self.add_point(point);
                }
            }
            GeometryCow::MultiPolygon(multi_polygon) => {
                // 检查此几何体是否应遵循边界判定规则
                // 除多边形之外，所有集合都遵循规则
                self.use_boundary_determination_rule = false;
                for polygon in &multi_polygon.0 {
                    self.add_polygon(polygon);
                }
            }
            GeometryCow::MultiLineString(multi_line_string) => {
                for line_string in &multi_line_string.0 {
                    self.add_line_string(line_string);
                }
            }
            GeometryCow::GeometryCollection(geometry_collection) => {
                for geometry in geometry_collection.iter() {
                    self.add_geometry(&GeometryCow::from(geometry));
                }
            }
        }
    }

    fn add_polygon_ring(
        &mut self,
        linear_ring: &LineString<F>,
        cw_left: CoordPos,
        cw_right: CoordPos,
    ) {
        debug_assert!(linear_ring.is_closed());
        if linear_ring.is_empty() {
            return;
        }

        let mut coords: Vec<Coord<F>> = Vec::with_capacity(linear_ring.0.len());
        // 移除重复的坐标
        for coord in &linear_ring.0 {
            if coords.last() != Some(coord) {
                coords.push(*coord)
            }
        }

        if coords.len() < 4 {
            // TODO: 我们可以在此返回Err，但这会对如何在其他操作中使用此代码产生影响 - 我们希望所有方法，比如`contains`返回Result吗？
            warn!("遇到无效环，结果未定义");
        }
        let first_point = coords[0];

        use crate::winding_order::{Winding, WindingOrder};
        let (left, right) = match linear_ring.winding_order() {
            Some(WindingOrder::Clockwise) => (cw_left, cw_right),
            Some(WindingOrder::CounterClockwise) => (cw_right, cw_left),
            None => {
                warn!("多边形没有绕圈顺序。结果未定义。");
                (cw_left, cw_right)
            }
        };

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::area(CoordPos::OnBoundary, left, right),
            ),
        );
        self.insert_edge(edge);

        // 插入端点为节点，以标记它在边界上
        self.insert_point(self.arg_index, first_point, CoordPos::OnBoundary);
    }

    fn add_polygon(&mut self, polygon: &Polygon<F>) {
        self.add_polygon_ring(polygon.exterior(), CoordPos::Outside, CoordPos::Inside);
        // 孔的拓扑标签与壳体相对，因为多边形的内部位于它们的对侧
        // （如果孔是顺时针定向的，则在左侧）
        for hole in polygon.interiors() {
            self.add_polygon_ring(hole, CoordPos::Inside, CoordPos::Outside)
        }
    }

    fn add_line_string(&mut self, line_string: &LineString<F>) {
        if line_string.is_empty() {
            return;
        }

        let mut coords: Vec<Coord<F>> = Vec::with_capacity(line_string.0.len());
        for coord in &line_string.0 {
            if coords.last() != Some(coord) {
                coords.push(*coord)
            }
        }

        if coords.len() < 2 {
            warn!("将无效线字符串处理为点，结果未定义");
            self.add_point(&coords[0].into());
            return;
        }

        self.insert_boundary_point(*coords.first().unwrap());
        self.insert_boundary_point(*coords.last().unwrap());

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::line_or_point(CoordPos::Inside),
            ),
        );
        self.insert_edge(edge);
    }

    fn add_line(&mut self, line: &Line<F>) {
        self.insert_boundary_point(line.start);
        self.insert_boundary_point(line.end);

        let edge = Edge::new(
            vec![line.start, line.end],
            Label::new(
                self.arg_index,
                TopologyPosition::line_or_point(CoordPos::Inside),
            ),
        );

        self.insert_edge(edge);
    }

    /// 添加外部计算的点。假设该点是一个几何体的点部分，位置在内部。
    fn add_point(&mut self, point: &Point<F>) {
        self.insert_point(self.arg_index, (*point).into(), CoordPos::Inside);
    }

    /// 计算自节点，利用几何体类型来最小化相交测试的数量。（例如，圆环不进行自相交测试，因为假设它们是有效的）。
    ///
    /// `line_intersector` 用于确定相交的 [`LineIntersector`]
    pub(crate) fn compute_self_nodes(&mut self, line_intersector: Box<dyn LineIntersector<F>>) {
        if self.has_computed_self_nodes {
            return;
        }
        self.has_computed_self_nodes = true;

        let mut segment_intersector = SegmentIntersector::new(line_intersector, true);

        // 优化有效多边形和线性环的相交搜索
        let is_rings = match self.geometry() {
            GeometryCow::LineString(ls) => ls.is_closed(),
            GeometryCow::MultiLineString(ls) => ls.is_closed(),
            GeometryCow::Polygon(_) | GeometryCow::MultiPolygon(_) => true,
            _ => false,
        };
        let check_for_self_intersecting_edges = !is_rings;

        let edge_set_intersector = RStarEdgeSetIntersector;
        edge_set_intersector.compute_intersections_within_set(
            self,
            check_for_self_intersecting_edges,
            &mut segment_intersector,
        );
        self.add_self_intersection_nodes();
    }

    pub(crate) fn compute_edge_intersections(
        &self,
        other: &GeometryGraph<F>,
        line_intersector: Box<dyn LineIntersector<F>>,
    ) -> SegmentIntersector<F> {
        let mut segment_intersector = SegmentIntersector::new(line_intersector, false);
        segment_intersector.set_boundary_nodes(
            self.boundary_nodes().cloned().collect(),
            other.boundary_nodes().cloned().collect(),
        );

        let edge_set_intersector = RStarEdgeSetIntersector;
        edge_set_intersector.compute_intersections_between_sets(
            self,
            other,
            &mut segment_intersector,
        );

        segment_intersector
    }

    fn insert_point(&mut self, arg_index: usize, coord: Coord<F>, position: CoordPos) {
        let node: &mut CoordNode<F> = self.add_node_with_coordinate(coord);
        node.label_mut().set_on_position(arg_index, position);
    }

    /// 添加一维（线）几何体的边界点。
    fn insert_boundary_point(&mut self, coord: Coord<F>) {
        let arg_index = self.arg_index;
        let node: &mut CoordNode<F> = self.add_node_with_coordinate(coord);

        let label: &mut Label = node.label_mut();

        // 确定该点的当前位置（如果有）
        let boundary_count = {
            #[allow(clippy::bool_to_int_with_if)]
            let prev_boundary_count =
                if Some(CoordPos::OnBoundary) == label.position(arg_index, Direction::On) {
                    1
                } else {
                    0
                };
            prev_boundary_count + 1
        };

        let new_position = Self::determine_boundary(boundary_count);
        label.set_on_position(arg_index, new_position);
    }

    fn add_self_intersection_nodes(&mut self) {
        let positions_and_intersections: Vec<(CoordPos, Vec<Coord<F>>)> = self
            .edges()
            .iter()
            .map(|cell| cell.borrow())
            .map(|edge| {
                let position = edge
                    .label()
                    .on_position(self.arg_index)
                    .expect("所有边标签现在应该有一个`on`位置");
                let coordinates = edge
                    .edge_intersections()
                    .iter()
                    .map(|edge_intersection| edge_intersection.coordinate());

                (position, coordinates.collect())
            })
            .collect();

        for (position, edge_intersection_coordinates) in positions_and_intersections {
            for coordinate in edge_intersection_coordinates {
                self.add_self_intersection_node(coordinate, position)
            }
        }
    }

    /// 为自交添加一个节点。
    ///
    /// 如果节点是潜在的边界节点（例如来自边界的边），则将其作为潜在的边界节点插入。否则，只需作为普通节点添加即可。
    fn add_self_intersection_node(&mut self, coord: Coord<F>, position: CoordPos) {
        // 如果此节点已经是边界节点，则不更改
        if self.is_boundary_node(coord) {
            return;
        }

        if position == CoordPos::OnBoundary && self.use_boundary_determination_rule {
            self.insert_boundary_point(coord)
        } else {
            self.insert_point(self.arg_index, coord, position)
        }
    }
}
