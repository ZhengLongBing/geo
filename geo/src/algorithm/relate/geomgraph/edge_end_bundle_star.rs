use super::{
    Dimensions, Direction, EdgeEnd, EdgeEndBundle, EdgeEndKey, GeometryGraph, IntersectionMatrix,
    LabeledEdgeEndBundle,
};
use crate::coordinate_position::{CoordPos, CoordinatePosition};
use crate::{Coord, GeoFloat, GeometryCow};

/// 一个有序的[`EdgeEndBundle`]列表，围绕一个[`RelateNodeFactory::Node`]。
///
/// 它们以逆时针顺序（从正x轴开始）维护在节点周围，以便于高效查找和拓扑构建。
///
/// 基于[JTS的`EdgeEndBundleStar`截至版本1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/EdgeEndBundleStar.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    edge_map: std::collections::BTreeMap<EdgeEndKey<F>, EdgeEndBundle<F>>,
    point_in_area_location: Option<[CoordPos; 2]>,
}

#[derive(Clone, Debug)]
pub(crate) struct LabeledEdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    edges: Vec<LabeledEdgeEndBundle<F>>,
}

impl<F: GeoFloat> LabeledEdgeEndBundleStar<F> {
    pub(crate) fn new(
        edges: Vec<LabeledEdgeEndBundle<F>>,
        graph_a: &GeometryGraph<F>,
        graph_b: &GeometryGraph<F>,
    ) -> Self {
        let mut labeled_bundle_star = Self { edges };
        labeled_bundle_star.compute_labeling(graph_a, graph_b);
        labeled_bundle_star
    }

    /// 基于其EdgeEndBundles的标签计算星的标签。
    fn compute_labeling(&mut self, graph_a: &GeometryGraph<F>, graph_b: &GeometryGraph<F>) {
        self.propagate_side_labels(0, graph_a);
        self.propagate_side_labels(1, graph_b);
        let mut has_dimensional_collapse_edge = [false, false];
        for edge_end in self.edges.iter() {
            let label = edge_end.label();
            for (geom_index, is_collapsed) in has_dimensional_collapse_edge.iter_mut().enumerate() {
                *is_collapsed = label.is_line(geom_index)
                    && label.on_position(geom_index) == Some(CoordPos::OnBoundary);
            }
        }
        for edge_end_bundle in &mut self.edges {
            let coord = *edge_end_bundle.coordinate();
            let label = edge_end_bundle.label_mut();
            for (geom_index, is_dimensionally_collapsed) in
                has_dimensional_collapse_edge.iter().enumerate()
            {
                if label.is_any_empty(geom_index) {
                    let position: CoordPos = if *is_dimensionally_collapsed {
                        CoordPos::Outside
                    } else {
                        // 性能：在JTS中这是缓存的，但在Rust的借用检查器中做到这一点有点棘手。
                        // 我们先观察是否成为热点。
                        let geometry = match geom_index {
                            0 => graph_a.geometry(),
                            1 => graph_b.geometry(),
                            _ => unreachable!("无效的geom_index"),
                        };
                        use crate::HasDimensions;
                        if geometry.dimensions() == Dimensions::TwoDimensional {
                            geometry.coordinate_position(&coord)
                        } else {
                            // 如果几何体不是一个面积，坐标始终为Outside
                            CoordPos::Outside
                        }
                    };
                    label.set_all_positions_if_empty(geom_index, position);
                }
            }
        }
        debug!("edge_end_bundle_star: {:?}", self);
    }

    fn propagate_side_labels(&mut self, geom_index: usize, geometry_graph: &GeometryGraph<F>) {
        let mut start_position = None;

        for edge_ends in self.edge_end_bundles_iter() {
            let label = edge_ends.label();
            if label.is_geom_area(geom_index) {
                if let Some(position) = label.position(geom_index, Direction::Left) {
                    start_position = Some(position);
                }
            }
        }
        if start_position.is_none() {
            return;
        }
        let mut current_position = start_position.unwrap();

        for edge_ends in self.edge_end_bundles_iter_mut() {
            let label = edge_ends.label_mut();
            if label.position(geom_index, Direction::On).is_none() {
                label.set_position(geom_index, Direction::On, current_position);
            }
            if label.is_geom_area(geom_index) {
                let left_position = label.position(geom_index, Direction::Left);
                let right_position = label.position(geom_index, Direction::Right);

                if let Some(right_position) = right_position {
                    #[cfg(debug_assertions)]
                    if right_position != current_position {
                        use crate::algorithm::Validation;
                        if geometry_graph.geometry().is_valid() {
                            debug_assert!(false, "拓扑位置与坐标冲突——这可能发生在无效几何体上。坐标: {:?}, 右边位置: {:?}, 当前位置: {:?}", edge_ends.coordinate(), right_position, current_position);
                        } else {
                            warn!("拓扑位置与坐标冲突——这可能发生在无效几何体上。坐标: {:?}, 右边位置: {:?}, 当前位置: {:?}", edge_ends.coordinate(), right_position, current_position);
                        }
                    }
                    assert!(left_position.is_some(), "发现单个null侧");
                    current_position = left_position.unwrap();
                } else {
                    debug_assert!(label.position(geom_index, Direction::Left).is_none());
                    label.set_position(geom_index, Direction::Right, current_position);
                    label.set_position(geom_index, Direction::Left, current_position);
                }
            }
        }
    }

    fn edge_end_bundles_iter(&self) -> impl Iterator<Item = &LabeledEdgeEndBundle<F>> {
        self.edges.iter()
    }

    fn edge_end_bundles_iter_mut(&mut self) -> impl Iterator<Item = &mut LabeledEdgeEndBundle<F>> {
        self.edges.iter_mut()
    }

    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        for edge_end_bundle in self.edge_end_bundles_iter() {
            edge_end_bundle.update_intersection_matrix(intersection_matrix);
            debug!(
                "更新的intersection_matrix: {:?} 来自edge_end_bundle: {:?}",
                intersection_matrix, edge_end_bundle
            );
        }
    }
}

impl<F> EdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    pub(crate) fn new() -> Self {
        EdgeEndBundleStar {
            edge_map: std::collections::BTreeMap::new(),
            point_in_area_location: None,
        }
    }

    pub(crate) fn insert(&mut self, edge_end: EdgeEnd<F>) {
        let bundle = self
            .edge_map
            .entry(edge_end.key().clone())
            .or_insert_with(|| EdgeEndBundle::new(*edge_end.coordinate()));
        bundle.insert(edge_end);
    }

    fn edge_end_bundles_iter(&self) -> impl Iterator<Item = &EdgeEndBundle<F>> {
        self.edge_map.values()
    }

    fn edge_end_bundles_iter_mut(&mut self) -> impl Iterator<Item = &mut EdgeEndBundle<F>> {
        self.edge_map.values_mut()
    }

    /// 计算星的EdgeEndBundles的标签，并使用它们计算星的整体标签。
    ///
    /// 实现说明：这在两个方面与JTS有所不同。
    ///
    /// 首先，JTS不使用optionals，而是设置可为空的`Label`s，而这里我们转换为一个显式标记的类型，以避免稍后解包optionals。
    ///
    /// 其次，在JTS中，这个功能并不直接在EdgeEndBundleStar上，而是在其父类[EdgeEndStar](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeEndStar.java#L117)上。
    ///
    /// 由于我们只使用这个子类（EdgeEndBundleStar），我们跳过了将Java继承映射到Rust的复杂性，并直接在EdgeEndBundleStar上实现这个功能。
    ///
    /// 如果/当我们实现覆盖操作时，我们可以考虑提取父类行为。
    pub(crate) fn into_labeled(
        self,
        graph_a: &GeometryGraph<F>,
        graph_b: &GeometryGraph<F>,
    ) -> LabeledEdgeEndBundleStar<F> {
        debug!("edge_end_bundle_star: {:?}", self);
        let labeled_edges = self
            .edge_map
            .into_values()
            .map(|edge_end_bundle| edge_end_bundle.into_labeled())
            .collect();
        LabeledEdgeEndBundleStar::new(labeled_edges, graph_a, graph_b)
    }
}
