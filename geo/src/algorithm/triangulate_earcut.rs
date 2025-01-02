use crate::{coord, CoordFloat, CoordsIter, Polygon, Triangle};

/// 使用[ear-cutting算法](https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf)对多边形进行三角剖分。
///
/// 需要 `"earcutr"` 特性，默认启用。
pub trait TriangulateEarcut<T: CoordFloat> {
    /// # 示例
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // 西南
    ///     (x: 10., y: 0.), // 东南
    ///     (x: 10., y: 10.), // 东北
    ///     (x: 0., y: 10.), // 西北
    ///     (x: 0., y: 0.), // 西南
    /// ];
    ///
    /// let triangles = square_polygon.earcut_triangles();
    ///
    /// assert_eq!(
    ///     vec![
    ///         Triangle(
    ///             coord! { x: 0., y: 10. }, // 西北
    ///             coord! { x: 10., y: 10. }, // 东北
    ///             coord! { x: 10., y: 0. }, // 东南
    ///         ),
    ///         Triangle(
    ///             coord! { x: 10., y: 0. }, // 东南
    ///             coord! { x: 0., y: 0. }, // 西南
    ///             coord! { x: 0., y: 10. }, // 西北
    ///         ),
    ///     ],
    ///     triangles,
    /// );
    /// ```
    fn earcut_triangles(&self) -> Vec<Triangle<T>> {
        self.earcut_triangles_iter().collect()
    }

    /// # 示例
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // 西南
    ///     (x: 10., y: 0.), // 东南
    ///     (x: 10., y: 10.), // 东北
    ///     (x: 0., y: 10.), // 西北
    ///     (x: 0., y: 0.), // 西南
    /// ];
    ///
    /// let mut triangles_iter = square_polygon.earcut_triangles_iter();
    ///
    /// assert_eq!(
    ///     Some(Triangle(
    ///             coord! { x: 0., y: 10. }, // 西北
    ///             coord! { x: 10., y: 10. }, // 东北
    ///             coord! { x: 10., y: 0. }, // 东南
    ///     )),
    ///     triangles_iter.next(),
    /// );
    ///
    /// assert_eq!(
    ///     Some(Triangle(
    ///         coord! { x: 10., y: 0. }, // 东南
    ///         coord! { x: 0., y: 0. }, // 西南
    ///         coord! { x: 0., y: 10. }, // 西北
    ///     )),
    ///     triangles_iter.next(),
    /// );
    ///
    /// assert!(triangles_iter.next().is_none());
    /// ```
    fn earcut_triangles_iter(&self) -> Iter<T> {
        Iter(self.earcut_triangles_raw())
    }

    /// 返回 `earcutr` 库的原始结果：一个一维的多边形顶点向量（按XY顺序）和三角形在顶点向量中的索引。
    /// 此方法不如 `earcut_triangles` 和 `earcut_triangles_iter` 方法那么方便，但在图形上下文中使用
    /// 一维数据时可能会很有帮助。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    /// use geo::triangulate_earcut::RawTriangulation;
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // 西南
    ///     (x: 10., y: 0.), // 东南
    ///     (x: 10., y: 10.), // 东北
    ///     (x: 0., y: 10.), // 西北
    ///     (x: 0., y: 0.), // 西南
    /// ];
    ///
    /// let mut triangles_raw = square_polygon.earcut_triangles_raw();
    ///
    /// assert_eq!(
    ///     RawTriangulation {
    ///         vertices: vec![
    ///             0., 0., // 西南
    ///             10., 0., // 东南
    ///             10., 10., // 东北
    ///             0., 10., // 西北
    ///             0., 0., // 西南
    ///         ],
    ///         triangle_indices: vec![
    ///             3, 0, 1, // 西北-西南-东南
    ///             1, 2, 3, // 东南-东北-西北
    ///         ],
    ///     },
    ///     triangles_raw,
    /// );
    /// ```
    fn earcut_triangles_raw(&self) -> RawTriangulation<T>;
}

impl<T: CoordFloat> TriangulateEarcut<T> for Polygon<T> {
    fn earcut_triangles_raw(&self) -> RawTriangulation<T> {
        let input = polygon_to_earcutr_input(self);
        let triangle_indices =
            earcutr::earcut(&input.vertices, &input.interior_indexes, 2).unwrap();
        RawTriangulation {
            vertices: input.vertices,
            triangle_indices,
        }
    }
}

/// 来自 `earcutr` 的多边形三角剖分原始结果。
#[derive(Debug, PartialEq, Clone)]
pub struct RawTriangulation<T: CoordFloat> {
    /// 一维多边形顶点向量（按XY顺序）。
    pub vertices: Vec<T>,

    /// 顶点向量中三角形的索引。
    pub triangle_indices: Vec<usize>,
}

#[derive(Debug)]
pub struct Iter<T: CoordFloat>(RawTriangulation<T>);

impl<T: CoordFloat> Iterator for Iter<T> {
    type Item = Triangle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let triangle_index_1 = self.0.triangle_indices.pop()?;
        let triangle_index_2 = self.0.triangle_indices.pop()?;
        let triangle_index_3 = self.0.triangle_indices.pop()?;
        Some(Triangle(
            self.triangle_index_to_coord(triangle_index_1),
            self.triangle_index_to_coord(triangle_index_2),
            self.triangle_index_to_coord(triangle_index_3),
        ))
    }
}

impl<T: CoordFloat> Iter<T> {
    fn triangle_index_to_coord(&self, triangle_index: usize) -> crate::Coord<T> {
        coord! {
            x: self.0.vertices[triangle_index * 2],
            y: self.0.vertices[triangle_index * 2 + 1],
        }
    }
}

struct EarcutrInput<T: CoordFloat> {
    pub vertices: Vec<T>,
    pub interior_indexes: Vec<usize>,
}

fn polygon_to_earcutr_input<T: CoordFloat>(polygon: &crate::Polygon<T>) -> EarcutrInput<T> {
    let mut vertices = Vec::with_capacity(polygon.coords_count() * 2);
    let mut interior_indexes = Vec::with_capacity(polygon.interiors().len());
    debug_assert!(polygon.exterior().0.len() >= 4);

    flat_line_string_coords_2(polygon.exterior(), &mut vertices);

    for interior in polygon.interiors() {
        debug_assert!(interior.0.len() >= 4); // 内部的线必须至少有四个点
        interior_indexes.push(vertices.len() / 2);
        flat_line_string_coords_2(interior, &mut vertices);
    }

    EarcutrInput {
        vertices,
        interior_indexes,
    }
}

fn flat_line_string_coords_2<T: CoordFloat>(
    line_string: &crate::LineString<T>,
    vertices: &mut Vec<T>,
) {
    for coord in &line_string.0 {
        vertices.push(coord.x); // 添加x坐标
        vertices.push(coord.y); // 添加y坐标
    }
}

#[cfg(test)]
mod test {
    use super::TriangulateEarcut;
    use crate::{coord, polygon, Triangle};

    #[test]
    fn test_triangle() {
        let triangle_polygon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 0.),
        ];

        let triangles = triangle_polygon.earcut_triangles();

        assert_eq!(
            &[Triangle(
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 10.0, y: 10.0 },
            ),][..],
            triangles,
        );
    }

    #[test]
    fn test_square() {
        let square_polygon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.),
        ];

        let mut triangles = square_polygon.earcut_triangles();
        triangles.sort_by(|t1, t2| t1.1.x.partial_cmp(&t2.2.x).unwrap());

        assert_eq!(
            &[
                Triangle(
                    coord! { x: 10.0, y: 0.0 },
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 0.0, y: 10.0 },
                ),
                Triangle(
                    coord! { x: 0.0, y: 10.0 },
                    coord! { x: 10.0, y: 10.0 },
                    coord! { x: 10.0, y: 0.0 },
                ),
            ][..],
            triangles,
        );
    }
}
