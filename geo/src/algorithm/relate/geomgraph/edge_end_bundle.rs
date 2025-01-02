use super::{CoordPos, Direction, Edge, EdgeEnd, GeometryGraph, IntersectionMatrix, Label};
use crate::{Coord, GeoFloat};

/// 一个遵循以下不变量的 [`EdgeEnds`](EdgeEnd) 集合：
/// 它们来自同一节点并且具有相同的方向。
///
/// 基于 [JTS 的 `EdgeEndBundle` 版本 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/EdgeEndBundle.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEndBundle<F>
where
    F: GeoFloat,
{
    coordinate: Coord<F>,
    edge_ends: Vec<EdgeEnd<F>>,
}

impl<F> EdgeEndBundle<F>
where
    F: GeoFloat,
{
    pub(crate) fn new(coordinate: Coord<F>) -> Self {
        Self {
            coordinate,
            edge_ends: vec![],
        }
    }

    fn edge_ends_iter(&self) -> impl Iterator<Item = &EdgeEnd<F>> {
        self.edge_ends.iter()
    }

    fn edge_ends_iter_mut(&mut self) -> impl Iterator<Item = &mut EdgeEnd<F>> {
        self.edge_ends.iter_mut()
    }

    pub(crate) fn insert(&mut self, edge_end: EdgeEnd<F>) {
        self.edge_ends.push(edge_end);
    }

    pub(crate) fn into_labeled(mut self) -> LabeledEdgeEndBundle<F> {
        let is_area = self
            .edge_ends_iter()
            .any(|edge_end| edge_end.label().is_area());

        let mut label = if is_area {
            Label::empty_area()
        } else {
            Label::empty_line_or_point()
        };

        for i in 0..2 {
            self.compute_label_on(&mut label, i);
            if is_area {
                self.compute_label_side(&mut label, i, Direction::Left);
                self.compute_label_side(&mut label, i, Direction::Right);
            }
        }

        LabeledEdgeEndBundle {
            label,
            edge_end_bundle: self,
        }
    }

    /// 计算 EdgeEnds 列表的整体 ON 位置。
    /// （这本质上等同于计算单个几何体的自叠加）
    ///
    /// EdgeEnds 可以位于父几何体的边界（例如多边形边）或者内部（例如 LineString 的段）。
    ///
    /// 此外，GeometryCollections 使用边界节点规则来确定段是否在边界上。
    ///
    /// 最后，在 GeometryCollections 中可能会出现边同时在边界和内部的情况（例如，LineString 段位于多边形边之上）。在这种情况下，优先考虑边界。
    ///
    /// 这些观察结果导致了以下计算 ON 位置的规则：
    /// - 如果有奇数个边界边，则属性为边界
    /// - 如果有偶数个 >= 2 的边界边，则属性为内部
    /// - 如果有任何内部边，则属性为内部
    /// - 否则，属性为 None
    ///
    fn compute_label_on(&mut self, label: &mut Label, geom_index: usize) {
        let mut boundary_count = 0;
        let mut found_interior = false;

        for edge_end in self.edge_ends_iter() {
            match edge_end.label().on_position(geom_index) {
                Some(CoordPos::OnBoundary) => {
                    boundary_count += 1;
                }
                Some(CoordPos::Inside) => {
                    found_interior = true;
                }
                None | Some(CoordPos::Outside) => {}
            }
        }

        let mut position = None;
        if found_interior {
            position = Some(CoordPos::Inside);
        }

        if boundary_count > 0 {
            position = Some(GeometryGraph::<'_, F>::determine_boundary(boundary_count));
        }

        if let Some(location) = position {
            label.set_on_position(geom_index, location);
        } else {
            // 这在技术上与 JTS 有所不同，但我认为我们永远不会
            // 到达这里，除非 `l.on_location` 已经是 None，在这种情况下这是一个
            // 无操作，因此要断言该假设。
            // 如果此断言被正确触发，我们可能需要添加类似
            // `l.clear_on_location(geom_index)` 的方法
            debug_assert!(
                label.on_position(geom_index).is_none(),
                "与 JTS 不同，它会将现有位置替换为 None"
            );
        }
    }

    /// 为一侧计算汇总标签的算法是：
    ///     对于所有边
    ///       如果任何边的位置是 INTERIOR，则边位置 = INTERIOR
    ///       否则如果至少有一个 EXTERIOR 属性，则边位置 = EXTERIOR
    ///       否则 边位置 = NULL
    /// 请注意，两个边可能具有明显矛盾的信息
    /// 即一条边可能表明它在几何体的内部，而
    /// 另一条边可能表明同一几何体的外部。这不是不兼容性
    /// - GeometryCollections 可能包含沿边缘相接的两个多边形。
    /// 上述内部优先规则的原因是
    /// 使汇总标签在两侧都具有几何内部。
    fn compute_label_side(&mut self, label: &mut Label, geom_index: usize, side: Direction) {
        let mut position = None;
        for edge_end in self.edge_ends_iter_mut() {
            if edge_end.label().is_area() {
                match edge_end.label_mut().position(geom_index, side) {
                    Some(CoordPos::Inside) => {
                        position = Some(CoordPos::Inside);
                        break;
                    }
                    Some(CoordPos::Outside) => {
                        position = Some(CoordPos::Outside);
                    }
                    None | Some(CoordPos::OnBoundary) => {}
                }
            }
        }

        if let Some(position) = position {
            label.set_position(geom_index, side, position);
        }
    }
}

/// 一个 [`EdgeEndBundle`]，其拓扑关系已聚合为单个
/// [`Label`]。
///
/// `update_intersection_matrix` 将此聚合的拓扑应用于 `IntersectionMatrix`。
#[derive(Clone, Debug)]
pub(crate) struct LabeledEdgeEndBundle<F>
where
    F: GeoFloat,
{
    label: Label,
    edge_end_bundle: EdgeEndBundle<F>,
}

impl<F> LabeledEdgeEndBundle<F>
where
    F: GeoFloat,
{
    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        Edge::<F>::update_intersection_matrix(self.label(), intersection_matrix);
    }

    pub fn coordinate(&self) -> &Coord<F> {
        &self.edge_end_bundle.coordinate
    }
}
