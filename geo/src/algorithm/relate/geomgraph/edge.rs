use super::{Dimensions, Direction, EdgeIntersection, IntersectionMatrix, Label};
use super::{LineIntersection, LineIntersector, RobustLineIntersector};
use crate::{Coord, GeoFloat, Line};

use std::collections::BTreeSet;

/// `Edge` 表示几何体中的一维线。
///
/// 这是基于[JTS的`Edge`截至版本1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/Edge.java)
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Edge<F: GeoFloat> {
    /// 线的几何坐标
    coords: Vec<Coord<F>>,

    /// 如果没有其他边与之接触，则边是"隔离的"
    is_isolated: bool,

    /// 与此边相交的其他边
    edge_intersections: BTreeSet<EdgeIntersection<F>>,

    /// 记录该线相对于两个几何体的拓扑分类
    label: Label,
}

impl<F: GeoFloat> Edge<F> {
    /// 创建一个新的Edge。
    ///
    /// - `coords` 一个非空的`Vec`类型的坐标
    /// - `label` Edge的适当维度的拓扑标签。详情参阅[`TopologyPosition`]
    pub(crate) fn new(mut coords: Vec<Coord<F>>, label: Label) -> Edge<F> {
        assert!(!coords.is_empty(), "不能添加空边");
        // 一旦设定，`edge.coords`的长度不再改变。
        coords.shrink_to_fit();
        Edge {
            coords,
            label,
            is_isolated: true,
            edge_intersections: BTreeSet::new(),
        }
    }

    pub(crate) fn label(&self) -> &Label {
        &self.label
    }

    pub(crate) fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    /// 比较两个准备好的几何体时，我们缓存每个几何体的拓扑图。
    /// 根据操作的顺序 - `a.relate(b)` vs `b.relate(a)` - 可能需要交换标签。
    pub fn swap_label_args(&mut self) {
        self.label.swap_args()
    }

    pub fn coords(&self) -> &[Coord<F>] {
        &self.coords
    }

    pub fn is_isolated(&self) -> bool {
        self.is_isolated
    }

    pub fn mark_as_unisolated(&mut self) {
        self.is_isolated = false;
    }

    pub fn edge_intersections(&self) -> &BTreeSet<EdgeIntersection<F>> {
        &self.edge_intersections
    }

    pub fn edge_intersections_mut(&mut self) -> &mut BTreeSet<EdgeIntersection<F>> {
        &mut self.edge_intersections
    }

    pub fn add_edge_intersection_list_endpoints(&mut self) {
        let max_segment_index = self.coords().len() - 1;
        let first_coord = self.coords()[0];
        let max_coord = self.coords()[max_segment_index];
        self.edge_intersections_mut()
            .insert(EdgeIntersection::new(first_coord, 0, F::zero()));
        self.edge_intersections_mut().insert(EdgeIntersection::new(
            max_coord,
            max_segment_index,
            F::zero(),
        ));
    }

    pub fn is_closed(&self) -> bool {
        self.coords().first() == self.coords().last()
    }

    /// 为边的一个段中找到的一个或两个相交点添加EdgeIntersections到边的相交列表中。
    pub fn add_intersections(
        &mut self,
        intersection: LineIntersection<F>,
        line: Line<F>,
        segment_index: usize,
    ) {
        match intersection {
            LineIntersection::SinglePoint { intersection, .. } => {
                self.add_intersection(intersection, line, segment_index);
            }
            LineIntersection::Collinear { intersection } => {
                self.add_intersection(intersection.start, line, segment_index);
                self.add_intersection(intersection.end, line, segment_index);
            }
        }
    }

    /// 为`intersection`添加一个EdgeIntersection。
    ///
    /// 落在边的顶点上的相交点被归一化为使用可能的`segment_index`中较大的一个
    pub fn add_intersection(
        &mut self,
        intersection_coord: Coord<F>,
        line: Line<F>,
        segment_index: usize,
    ) {
        let mut normalized_segment_index = segment_index;
        let mut distance = RobustLineIntersector::compute_edge_distance(intersection_coord, line);

        let next_segment_index = normalized_segment_index + 1;

        if next_segment_index < self.coords.len() {
            let next_coord = self.coords[next_segment_index];
            if intersection_coord == next_coord {
                normalized_segment_index = next_segment_index;
                distance = F::zero();
            }
        }
        self.edge_intersections.insert(EdgeIntersection::new(
            intersection_coord,
            normalized_segment_index,
            distance,
        ));
    }

    /// 用此组件的贡献更新相交矩阵(IM)。
    ///
    /// 仅当组件为两个父几何体都有标签时，才贡献。
    pub fn update_intersection_matrix(label: &Label, intersection_matrix: &mut IntersectionMatrix) {
        intersection_matrix.set_at_least_if_in_both(
            label.position(0, Direction::On),
            label.position(1, Direction::On),
            Dimensions::OneDimensional,
        );

        if label.is_area() {
            intersection_matrix.set_at_least_if_in_both(
                label.position(0, Direction::Left),
                label.position(1, Direction::Left),
                Dimensions::TwoDimensional,
            );
            intersection_matrix.set_at_least_if_in_both(
                label.position(0, Direction::Right),
                label.position(1, Direction::Right),
                Dimensions::TwoDimensional,
            );
        }
    }
}
