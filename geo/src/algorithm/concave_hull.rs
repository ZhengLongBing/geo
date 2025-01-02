use crate::convex_hull::qhull;
use crate::utils::partial_min;
use crate::{
    coord, Centroid, Coord, CoordNum, Distance, Euclidean, GeoFloat, Length, Line, LineString,
    MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use rstar::{RTree, RTreeNum};
use std::collections::VecDeque;

/// 返回覆盖几何体的多边形。与凸包不同的是，凹包不仅覆盖它们的几何体，
/// 而且通过构建边缘来尽量减少面积，使多边形的外部包含在凸包中会是内部点的点。
///
/// 该实现受到 <https://github.com/mapbox/concaveman> 的启发
/// 并使用了以下论文中的一些想法： www.iis.sinica.edu.tw/page/jise/2012/201205_10.pdf
///
/// # 示例
/// ```
/// use geo::{line_string, polygon};
/// use geo::ConcaveHull;
///
/// // 一个正方形
/// let poly = polygon![
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 4.0),
///     (x: 0.0, y: 4.0),
/// ];
///
/// // 正确的凹包坐标
/// let correct_hull = line_string![
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
/// ];
///
/// let res = poly.concave_hull(2.0);
/// assert_eq!(res.exterior(), &correct_hull);
/// ```
pub trait ConcaveHull {
    type Scalar: CoordNum;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar>;
}

impl<T> ConcaveHull for Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        let mut points: Vec<_> = self.exterior().0.clone();
        Polygon::new(concave_hull(&mut points, concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiPolygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        let mut aggregated: Vec<Coord<Self::Scalar>> = self
            .0
            .iter()
            .flat_map(|elem| elem.exterior().0.clone())
            .collect();
        Polygon::new(concave_hull(&mut aggregated, concavity), vec![])
    }
}

impl<T> ConcaveHull for LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        Polygon::new(concave_hull(&mut self.0.clone(), concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiLineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let mut aggregated: Vec<Coord<T>> = self.iter().flat_map(|elem| elem.0.clone()).collect();
        Polygon::new(concave_hull(&mut aggregated, concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiPoint<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let mut coordinates: Vec<Coord<T>> = self.iter().map(|point| point.0).collect();
        Polygon::new(concave_hull(&mut coordinates, concavity), vec![])
    }
}

fn find_point_closest_to_line<T>(
    interior_coords_tree: &RTree<Coord<T>>,
    line: Line<T>,
    max_dist: T,
    edge_length: T,
    concavity: T,
    line_tree: &RTree<Line<T>>,
) -> Option<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    let h = max_dist + max_dist;
    let w = line.length::<Euclidean>() + h;
    let two = T::add(T::one(), T::one());
    let search_dist = T::div(T::sqrt(T::powi(w, 2) + T::powi(h, 2)), two);
    let centroid = line.centroid();
    let centroid_coord = coord! {
        x: centroid.x(),
        y: centroid.y(),
    };
    let mut candidates = interior_coords_tree
        .locate_within_distance(centroid_coord, search_dist)
        .peekable();
    let peek = candidates.peek();
    match peek {
        None => None,
        Some(&point) => {
            let closest_point =
                candidates.fold(Point::new(point.x, point.y), |acc_point, candidate| {
                    let candidate_point = Point::new(candidate.x, candidate.y);
                    if Euclidean::distance(&line, &acc_point)
                        > Euclidean::distance(&line, &candidate_point)
                    {
                        candidate_point
                    } else {
                        acc_point
                    }
                });
            let mut edges_nearby_point = line_tree
                .locate_within_distance(closest_point, search_dist)
                .peekable();
            let peeked_edge = edges_nearby_point.peek();

            // Clippy在这里有一个问题。这可能是一个有效的建议，
            // 但Clippy的自动修复会破坏代码，因此可能需要手动进行。
            // 参见 https://github.com/rust-lang/rust/issues/94241
            #[allow(clippy::manual_map)]
            let closest_edge_option = match peeked_edge {
                None => None,
                Some(&edge) => Some(edges_nearby_point.fold(*edge, |acc, candidate| {
                    if Euclidean::distance(&closest_point, &acc)
                        > Euclidean::distance(&closest_point, candidate)
                    {
                        *candidate
                    } else {
                        acc
                    }
                })),
            };
            let decision_distance = partial_min(
                Euclidean::distance(&closest_point, &line.start_point()),
                Euclidean::distance(&closest_point, &line.end_point()),
            );
            if let Some(closest_edge) = closest_edge_option {
                let far_enough = edge_length / decision_distance > concavity;
                let are_edges_equal = closest_edge == line;
                if far_enough && are_edges_equal {
                    Some(coord! {
                        x: closest_point.x(),
                        y: closest_point.y(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

// 这一函数在很大程度上得到了以下代码的启发:
// https://github.com/mapbox/concaveman/blob/54838e1/index.js#L11
fn concave_hull<T>(coords: &mut [Coord<T>], concavity: T) -> LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    let hull = qhull::quick_hull(coords);

    if coords.len() < 4 {
        return hull;
    }

    // 获取不在壳体外部线串上的整个数据集中点
    let hull_tree: RTree<Coord<T>> = RTree::bulk_load(hull.clone().0);

    let interior_coords: Vec<Coord<T>> = coords
        .iter()
        .filter(|coord| !hull_tree.contains(coord))
        .copied()
        .collect();
    let mut interior_points_tree: RTree<Coord<T>> = RTree::bulk_load(interior_coords);
    let mut line_tree: RTree<Line<T>> = RTree::new();

    let mut concave_list: Vec<Point<T>> = vec![];
    let lines = hull.lines();
    let mut line_queue: VecDeque<Line<T>> = VecDeque::new();

    for line in lines {
        line_queue.push_back(line);
        line_tree.insert(line);
    }
    while let Some(line) = line_queue.pop_front() {
        let edge_length = line.length::<Euclidean>();
        let dist = edge_length / concavity;
        let possible_closest_point = find_point_closest_to_line(
            &interior_points_tree,
            line,
            dist,
            edge_length,
            concavity,
            &line_tree,
        );

        if let Some(closest_point) = possible_closest_point {
            interior_points_tree.remove(&closest_point);
            line_tree.remove(&line);
            let point = Point::new(closest_point.x, closest_point.y);
            let start_line = Line::new(line.start_point(), point);
            let end_line = Line::new(point, line.end_point());
            line_tree.insert(start_line);
            line_tree.insert(end_line);
            line_queue.push_front(end_line);
            line_queue.push_front(start_line);
        } else {
            // 确保我们不添加重复项
            if concave_list.is_empty() || !concave_list.ends_with(&[line.start_point()]) {
                concave_list.push(line.start_point());
            }
            concave_list.push(line.end_point());
        }
    }

    concave_list.into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, polygon};
    use geo_types::Coord;

    #[test]
    fn triangle_test() {
        let mut triangle = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 2.0, y: 2.0 },
        ];

        let correct = line_string![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 2.0, y: 2.0),
            (x: 0.0, y: 0.0),
        ];

        let concavity = 2.0;
        let res = concave_hull(&mut triangle, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn square_test() {
        let mut square = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 0.0, y: 4.0 },
        ];

        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];

        let concavity = 2.0;
        let res = concave_hull(&mut square, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn one_flex_test() {
        let mut v = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 2.0, y: 1.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 0.0, y: 4.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 1.0;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn four_flex_test() {
        let mut v = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 2.0, y: 1.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 3.0, y: 2.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 2.0, y: 3.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: 1.0, y: 2.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 3.0, y: 2.0),
            (x: 4.0, y: 4.0),
            (x: 2.0, y: 3.0),
            (x: 0.0, y: 4.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 1.7;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn consecutive_flex_test() {
        let mut v = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 3.0, y: 1.0 },
            coord! { x: 3.0, y: 2.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 2.0),
            (x: 3.0, y: 1.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 2.0;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn concave_hull_norway_test() {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let norway_concave_hull: LineString = geo_test_fixtures::norway_concave_hull::<f64>();
        let res = norway.concave_hull(2.0);
        assert_eq!(res.exterior(), &norway_concave_hull);
    }

    #[test]
    fn concave_hull_linestring_test() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 1.0),
            (x: 3.0, y: 2.0)
        ];
        let concave = linestring.concave_hull(2.0);
        let correct = vec![
            Coord::from((4.0, 0.0)),
            Coord::from((4.0, 4.0)),
            Coord::from((3.0, 2.0)),
            Coord::from((3.0, 1.0)),
            Coord::from((0.0, 0.0)),
            Coord::from((4.0, 0.0)),
        ];
        assert_eq!(concave.exterior().0, correct);
    }

    #[test]
    fn concave_hull_multilinestring_test() {
        let v1 = line_string![
             (x: 0.0, y: 0.0),
             (x: 4.0, y: 0.0)
        ];
        let v2 = line_string![
             (x: 4.0, y: 4.0),
             (x: 3.0, y: 1.0),
             (x: 3.0, y: 2.0)
        ];
        let mls = MultiLineString::new(vec![v1, v2]);
        let correct = vec![
            Coord::from((4.0, 0.0)),
            Coord::from((4.0, 4.0)),
            Coord::from((3.0, 2.0)),
            Coord::from((3.0, 1.0)),
            Coord::from((0.0, 0.0)),
            Coord::from((4.0, 0.0)),
        ];
        let res = mls.concave_hull(2.0);
        assert_eq!(res.exterior().0, correct);
    }

    #[test]
    fn concave_hull_multipolygon_test() {
        let v1 = polygon![
             (x: 0.0, y: 0.0),
             (x: 4.0, y: 0.0)
        ];
        let v2 = polygon![
             (x: 4.0, y: 4.0),
             (x: 3.0, y: 1.0),
             (x: 3.0, y: 2.0)
        ];
        let multipolygon = MultiPolygon::new(vec![v1, v2]);
        let res = multipolygon.concave_hull(2.0);
        let correct = vec![
            Coord::from((4.0, 0.0)),
            Coord::from((4.0, 4.0)),
            Coord::from((3.0, 2.0)),
            Coord::from((3.0, 1.0)),
            Coord::from((0.0, 0.0)),
            Coord::from((4.0, 0.0)),
        ];
        assert_eq!(res.exterior().0, correct);
    }
}
