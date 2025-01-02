pub(crate) use edge_end_builder::EdgeEndBuilder;
pub use geomgraph::intersection_matrix::IntersectionMatrix;
use relate_operation::RelateOperation;

use crate::geometry::*;
pub use crate::relate::geomgraph::index::PreparedGeometry;
pub use crate::relate::geomgraph::GeometryGraph;
use crate::{GeoFloat, GeometryCow};

mod edge_end_builder;
mod geomgraph;
mod relate_operation;

/// 基于[DE-9IM](https://en.wikipedia.org/wiki/DE-9IM)语义拓扑地关联两个几何体。
///
/// 参见 [`IntersectionMatrix`] 以获取详细信息。所有谓词都可在计算出的矩阵上使用。
///
/// # 示例
///
/// ```
/// use geo::{coord, Line, Rect, line_string};
/// use crate::geo::relate::Relate;
///
/// let line = Line::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let rect = Rect::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let intersection_matrix = rect.relate(&line);
///
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// assert!(intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
///
/// let line = Line::new(coord! { x: 1.0, y: 1.0}, coord! { x: 5.0, y: 5.0 });
/// let rect = Rect::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let intersection_matrix = rect.relate(&line);
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// assert!(!intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
///
/// let rect_boundary = line_string![
///     (x: 2.0, y: 2.0),
///     (x: 4.0, y: 2.0),
///     (x: 4.0, y: 4.0),
///     (x: 2.0, y: 4.0),
///     (x: 2.0, y: 2.0)
/// ];
/// let intersection_matrix = rect.relate(&rect_boundary);
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// // 根据 DE-9IM, 多边形不包含其自身的边界
/// assert!(!intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
/// ```
///
/// 注意: `Relate` 不应在包含 `NaN` 坐标的几何体上调用。
pub trait Relate<F: GeoFloat> {
    /// 构造一个 [`GeometryGraph`]
    fn geometry_graph(&self, arg_index: usize) -> GeometryGraph<F>;

    fn relate(&self, other: &impl Relate<F>) -> IntersectionMatrix {
        RelateOperation::new(self.geometry_graph(0), other.geometry_graph(1))
            .compute_intersection_matrix()
    }
}

macro_rules! relate_impl {
    ($($t:ty ,)*) => {
        $(
            impl<F: GeoFloat> Relate<F> for $t {
                fn geometry_graph(&self, arg_index: usize) -> GeometryGraph<F> {
                    GeometryGraph::new(arg_index, GeometryCow::from(self))
                }
            }
        )*
    };
}

relate_impl![
    Point<F>,
    Line<F>,
    LineString<F>,
    Polygon<F>,
    MultiPoint<F>,
    MultiLineString<F>,
    MultiPolygon<F>,
    Rect<F>,
    Triangle<F>,
    GeometryCollection<F>,
    Geometry<F>,
];

#[cfg(test)]
mod tests {
    #[test]
    fn run_jts_relate_tests() {
        jts_test_runner::assert_jts_tests_succeed("*Relate*.xml");
    }
}
