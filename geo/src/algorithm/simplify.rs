use crate::algorithm::{CoordsIter, Distance, Euclidean};
use crate::geometry::{Coord, Line, LineString, MultiLineString, MultiPolygon, Polygon};
use crate::GeoFloat;

const LINE_STRING_INITIAL_MIN: usize = 2;
const POLYGON_INITIAL_MIN: usize = 4;

// 由于RDP算法是递归的，我们无法在循环内为一个点分配索引
// 因此，我们将索引和点包装在一个简单的结构体中，然后在一个包装函数中传递它，
// 在传递过程中返回点或索引
#[derive(Copy, Clone)]
struct RdpIndex<T>
where
    T: GeoFloat,
{
    index: usize,
    coord: Coord<T>,
}

// RDP算法的包装器，返回简化后的点
fn rdp<T, I: Iterator<Item = Coord<T>>, const INITIAL_MIN: usize>(
    coords: I,
    epsilon: &T,
) -> Vec<Coord<T>>
where
    T: GeoFloat,
{
    // Epsilon必须大于零才能进行有意义的简化
    if *epsilon <= T::zero() {
        return coords.collect::<Vec<Coord<T>>>();
    }
    let rdp_indices = &coords
        .enumerate()
        .map(|(idx, coord)| RdpIndex { index: idx, coord })
        .collect::<Vec<RdpIndex<T>>>();
    let mut simplified_len = rdp_indices.len();
    let simplified_coords: Vec<_> =
        compute_rdp::<T, INITIAL_MIN>(rdp_indices, &mut simplified_len, epsilon)
            .into_iter()
            .map(|rdpindex| rdpindex.coord)
            .collect();
    debug_assert_eq!(simplified_coords.len(), simplified_len);
    simplified_coords
}

// RDP算法的包装器，返回简化后的点索引
fn calculate_rdp_indices<T, const INITIAL_MIN: usize>(
    rdp_indices: &[RdpIndex<T>],
    epsilon: &T,
) -> Vec<usize>
where
    T: GeoFloat,
{
    if *epsilon <= T::zero() {
        return rdp_indices
            .iter()
            .map(|rdp_index| rdp_index.index)
            .collect();
    }

    let mut simplified_len = rdp_indices.len();
    let simplified_coords =
        compute_rdp::<T, INITIAL_MIN>(rdp_indices, &mut simplified_len, epsilon)
            .into_iter()
            .map(|rdpindex| rdpindex.index)
            .collect::<Vec<usize>>();
    debug_assert_eq!(simplified_len, simplified_coords.len());
    simplified_coords
}

// Ramer-Douglas-Peucker线简化算法
// 此函数返回保留的点及其在原始几何体中的索引，
// 以便FFI实现者更灵活地使用
fn compute_rdp<T, const INITIAL_MIN: usize>(
    rdp_indices: &[RdpIndex<T>],
    simplified_len: &mut usize,
    epsilon: &T,
) -> Vec<RdpIndex<T>>
where
    T: GeoFloat,
{
    if rdp_indices.is_empty() {
        return vec![];
    }

    let first = rdp_indices[0];
    let last = rdp_indices[rdp_indices.len() - 1];
    if rdp_indices.len() == 2 {
        return vec![first, last];
    }

    let first_last_line = Line::new(first.coord, last.coord);

    // 找到距离`first_last_line`最远的`RdpIndex`
    let (farthest_index, farthest_distance) = rdp_indices
        .iter()
        .enumerate()
        .take(rdp_indices.len() - 1) // 不包括最后一个索引
        .skip(1) // 不包括第一个索引
        .map(|(index, rdp_index)| {
            (
                index,
                Euclidean::distance(rdp_index.coord, &first_last_line),
            )
        })
        .fold(
            (0usize, T::zero()),
            |(farthest_index, farthest_distance), (index, distance)| {
                if distance >= farthest_distance {
                    (index, distance)
                } else {
                    (farthest_index, farthest_distance)
                }
            },
        );
    debug_assert_ne!(farthest_index, 0);

    if farthest_distance > *epsilon {
        // 最远的索引大于epsilon，因此我们将递归简化由最远索引分割的子段。
        let mut intermediate =
            compute_rdp::<T, INITIAL_MIN>(&rdp_indices[..=farthest_index], simplified_len, epsilon);

        intermediate.pop(); // 不要重复包括最远的索引

        intermediate.extend_from_slice(&compute_rdp::<T, INITIAL_MIN>(
            &rdp_indices[farthest_index..],
            simplified_len,
            epsilon,
        ));
        return intermediate;
    }

    // 最远的索引小于或等于epsilon，因此我们将只保留第一个和最后一个索引，导致中间的索引被剔除。

    // 更新`simplified_len`以反映新的索引数量，方法是减去我们要剔除的索引数量。
    let number_culled = rdp_indices.len() - 2;
    let new_length = *simplified_len - number_culled;

    // 如果`simplified_len`现在低于所需的最小索引数，则不进行剔除并返回原始输入。
    if new_length < INITIAL_MIN {
        return rdp_indices.to_owned();
    }
    *simplified_len = new_length;

    // 剔除`first`和`last`之间的索引。
    vec![first, last]
}

/// 简化几何对象。
///
/// [Ramer-Douglas-Peucker
/// 算法](https://en.wikipedia.org/wiki/Ramer-Douglas-Peucker_algorithm)简化了一条线。
/// 多边形通过在其所有组成环上运行RDP算法来简化。
/// 这可能导致无效的多边形，且不保证保持拓扑。
///
/// Multi*对象通过分别简化其所有组成几何体来简化。
///
/// 较大的`epsilon`意味着更积极地移除与保持现有形状的关注度较少的点。
///
/// 具体来说，与简化输出距离比`epsilon`更近的点可能会被丢弃。
///
/// 小于或等于零的`epsilon`将返回未更改的几何体版本。
pub trait Simplify<T, Epsilon = T> {
    /// 使用[Ramer-Douglas-Peucker](https://en.wikipedia.org/wiki/Ramer-Douglas-Peucker_algorithm)
    /// 算法返回几何体的简化表示
    ///
    /// # 例子
    ///
    /// ```
    /// use geo::Simplify;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 17.3, y: 3.2),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// let simplified = line_string.simplify(&1.0);
    ///
    /// let expected = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// assert_eq!(expected, simplified)
    /// ```
    fn simplify(&self, epsilon: &T) -> Self
    where
        T: GeoFloat;
}

/// 简化几何体，返回保留的输入索引。
///
/// 此操作使用[Ramer-Douglas-Peucker
/// 算法](https://en.wikipedia.org/wiki/Ramer-Douglas-Peucker_algorithm)
/// 并不保证返回的几何体是有效的。
///
/// 较大的`epsilon`意味着更积极地移除与保持现有形状的关注度较少的点。
///
/// 具体来说，与简化输出距离比`epsilon`更近的点可能会被丢弃。
///
/// 小于或等于零的`epsilon`将返回未更改的几何体版本。
pub trait SimplifyIdx<T, Epsilon = T> {
    /// 使用[Ramer-Douglas-Peucker](https://en.wikipedia.org/wiki/Ramer-Douglas-Peucker_algorithm)
    /// 算法返回几何体的简化索引
    ///
    /// # 例子
    ///
    /// ```
    /// use geo::SimplifyIdx;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 17.3, y: 3.2),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// let simplified = line_string.simplify_idx(&1.0);
    ///
    /// let expected = vec![
    ///     0_usize,
    ///     1_usize,
    ///     2_usize,
    ///     4_usize,
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_idx(&self, epsilon: &T) -> Vec<usize>
    where
        T: GeoFloat;
}

impl<T> Simplify<T> for LineString<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: &T) -> Self {
        LineString::from(rdp::<_, _, LINE_STRING_INITIAL_MIN>(
            self.coords_iter(),
            epsilon,
        ))
    }
}

impl<T> SimplifyIdx<T> for LineString<T>
where
    T: GeoFloat,
{
    fn simplify_idx(&self, epsilon: &T) -> Vec<usize> {
        calculate_rdp_indices::<_, LINE_STRING_INITIAL_MIN>(
            &self
                .0
                .iter()
                .enumerate()
                .map(|(idx, coord)| RdpIndex {
                    index: idx,
                    coord: *coord,
                })
                .collect::<Vec<RdpIndex<T>>>(),
            epsilon,
        )
    }
}

impl<T> Simplify<T> for MultiLineString<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: &T) -> Self {
        MultiLineString::new(self.iter().map(|l| l.simplify(epsilon)).collect())
    }
}

impl<T> Simplify<T> for Polygon<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: &T) -> Self {
        Polygon::new(
            LineString::from(rdp::<_, _, POLYGON_INITIAL_MIN>(
                self.exterior().coords_iter(),
                epsilon,
            )),
            self.interiors()
                .iter()
                .map(|l| {
                    LineString::from(rdp::<_, _, POLYGON_INITIAL_MIN>(l.coords_iter(), epsilon))
                })
                .collect(),
        )
    }
}

impl<T> Simplify<T> for MultiPolygon<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: &T) -> Self {
        MultiPolygon::new(self.iter().map(|p| p.simplify(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, line_string, polygon};

    #[test]
    fn recursion_test() {
        let input = [
            coord! { x: 8.0, y: 100.0 },
            coord! { x: 9.0, y: 100.0 },
            coord! { x: 12.0, y: 100.0 },
        ];
        let actual = rdp::<_, _, 2>(input.into_iter(), &1.0);
        let expected = [coord! { x: 8.0, y: 100.0 }, coord! { x: 12.0, y: 100.0 }];
        assert_eq!(actual, expected);
    }

    #[test]
    fn rdp_test() {
        let vec = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 5.0, y: 4.0 },
            coord! { x: 11.0, y: 5.5 },
            coord! { x: 17.3, y: 3.2 },
            coord! { x: 27.8, y: 0.1 },
        ];
        let compare = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 5.0, y: 4.0 },
            coord! { x: 11.0, y: 5.5 },
            coord! { x: 27.8, y: 0.1 },
        ];
        let simplified = rdp::<_, _, 2>(vec.into_iter(), &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_empty_linestring() {
        let vec = Vec::new();
        let compare = Vec::new();
        let simplified = rdp::<_, _, 2>(vec.into_iter(), &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_two_point_linestring() {
        let vec = vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 27.8, y: 0.1 }];
        let compare = vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 27.8, y: 0.1 }];
        let simplified = rdp::<_, _, 2>(vec.into_iter(), &1.0);
        assert_eq!(simplified, compare);
    }

    #[test]
    fn multilinestring() {
        let mline = MultiLineString::new(vec![LineString::from(vec![
            (0.0, 0.0),
            (5.0, 4.0),
            (11.0, 5.5),
            (17.3, 3.2),
            (27.8, 0.1),
        ])]);

        let mline2 = mline.simplify(&1.0);

        assert_eq!(
            mline2,
            MultiLineString::new(vec![LineString::from(vec![
                (0.0, 0.0),
                (5.0, 4.0),
                (11.0, 5.5),
                (27.8, 0.1),
            ])])
        );
    }

    #[test]
    fn polygon() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ];

        let poly2 = poly.simplify(&2.);

        assert_eq!(
            poly2,
            polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.),
            ],
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon::new(vec![polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ]]);

        let mpoly2 = mpoly.simplify(&2.);

        assert_eq!(
            mpoly2,
            MultiPolygon::new(vec![polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.)
            ]]),
        );
    }

    #[test]
    fn simplify_negative_epsilon() {
        let ls = line_string![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
        ];
        let simplified = ls.simplify(&-1.0);
        assert_eq!(ls, simplified);
    }

    #[test]
    fn simplify_idx_negative_epsilon() {
        let ls = line_string![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
        ];
        let indices = ls.simplify_idx(&-1.0);
        assert_eq!(vec![0usize, 1, 2, 3, 4], indices);
    }

    // https://github.com/georust/geo/issues/142
    #[test]
    fn simplify_line_string_polygon_initial_min() {
        let ls = line_string![
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ( x: -5.9730447e26, y: 1.5590374e-27 ),
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
        ];
        let epsilon: f64 = 3.46e-43;

        // 线串结果应该是三个坐标
        let result = ls.simplify(&epsilon);
        assert_eq!(
            line_string![
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: -5.9730447e26, y: 1.5590374e-27 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ],
            result
        );

        // 多边形结果应该是五个坐标
        let result = Polygon::new(ls, vec![]).simplify(&epsilon);
        assert_eq!(
            polygon![
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: -5.9730447e26, y: 1.5590374e-27 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ],
            result,
        );
    }

    // https://github.com/georust/geo/issues/995
    #[test]
    fn dont_oversimplify() {
        let unsimplified = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 17.3, y: 3.2),
            (x: 27.8, y: 0.1)
        ];
        let actual = unsimplified.simplify(&30.0);
        let expected = line_string![
            (x: 0.0, y: 0.0),
            (x: 27.8, y: 0.1)
        ];
        assert_eq!(actual, expected);
    }
}
