use std::cmp::Ordering;

use crate::area::{get_linestring_area, Area};
use crate::dimensions::{Dimensions, Dimensions::*, HasDimensions};
use crate::geometry::*;
use crate::line_measures::{Euclidean, Length};
use crate::GeoFloat;

/// 计算质心。
/// 质心是形状中所有点的算术平均位置。
/// 非正式地说，它是可以使形状的剪影在针尖上完美平衡的点。
/// 凸对象的几何质心总是位于对象内部。
/// 非凸对象可能有一个质心，该质心在对象本身之外。
///
/// # 示例
///
/// ```
/// use geo::Centroid;
/// use geo::{point, polygon};
///
/// // 菱形的多边形
/// let polygon = polygon![
///     (x: -2., y: 1.),
///     (x: 1., y: 3.),
///     (x: 4., y: 1.),
///     (x: 1., y: -1.),
///     (x: -2., y: 1.),
/// ];
///
/// assert_eq!(
///     Some(point!(x: 1., y: 1.)),
///     polygon.centroid(),
/// );
/// ```
pub trait Centroid {
    type Output;

    /// 参见：<https://en.wikipedia.org/wiki/Centroid>
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{line_string, point};
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 40.02f64, y: 118.23),
    /// ];
    ///
    /// assert_eq!(
    ///     Some(point!(x: 40.02, y: 117.285)),
    ///     line_string.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output;
}

impl<T> Centroid for Line<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    /// [`Line`] 的质心是它的中间点
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{Line, point};
    ///
    /// let line = Line::new(
    ///     point!(x: 1.0f64, y: 3.0),
    ///     point!(x: 2.0f64, y: 4.0),
    /// );
    ///
    /// assert_eq!(
    ///     point!(x: 1.5, y: 3.5),
    ///     line.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        (self.start_point() + self.end_point()) / two
    }
}

impl<T> Centroid for LineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    // [`LineString`] 的质心是段中点的平均值，按段的长度加权。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{line_string, point};
    ///
    /// let line_string = line_string![
    ///   (x: 1.0f32, y: 1.0),
    ///   (x: 2.0, y: 2.0),
    ///   (x: 4.0, y: 4.0)
    ///   ];
    ///
    /// assert_eq!(
    ///     // (1.0 * (1.5, 1.5) + 2.0 * (3.0, 3.0)) / 3.0
    ///     Some(point!(x: 2.5, y: 2.5)),
    ///     line_string.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiLineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// [`MultiLineString`] 的质心是所有组成线字符串的质心的平均值，按每个线字符串的长度加权。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{MultiLineString, line_string, point};
    ///
    /// let multi_line_string = MultiLineString::new(vec![
    ///     // 质心: (2.5, 2.5)
    ///     line_string![(x: 1.0f32, y: 1.0), (x: 2.0, y: 2.0), (x: 4.0, y: 4.0)],
    ///     // 质心: (4.0, 4.0)
    ///     line_string![(x: 1.0, y: 1.0), (x: 3.0, y: 3.0), (x: 7.0, y: 7.0)],
    /// ]);
    ///
    /// assert_eq!(
    ///     // ( 3.0 * (2.5, 2.5) + 6.0 * (4.0, 4.0) ) / 9.0
    ///     Some(point!(x: 3.5, y: 3.5)),
    ///     multi_line_string.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for Polygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// [`Polygon`] 的质心是其所有点的平均值
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{polygon, point};
    ///
    /// let polygon = polygon![
    ///     (x: 0.0f32, y: 0.0),
    ///     (x: 2.0, y: 0.0),
    ///     (x: 2.0, y: 1.0),
    ///     (x: 0.0, y: 1.0),
    /// ];
    ///
    /// assert_eq!(
    ///     Some(point!(x: 1.0, y: 0.5)),
    ///     polygon.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiPolygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// [`MultiPolygon`] 的质心是其多边形质心的平均值，按多边形的面积加权。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{MultiPolygon, polygon, point};
    ///
    /// let multi_polygon = MultiPolygon::new(vec![
    ///   // 质心 (1.0, 0.5)
    ///   polygon![
    ///     (x: 0.0f32, y: 0.0),
    ///     (x: 2.0, y: 0.0),
    ///     (x: 2.0, y: 1.0),
    ///     (x: 0.0, y: 1.0),
    ///   ],
    ///   // 质心 (-0.5, 0.0)
    ///   polygon![
    ///     (x: 1.0, y: 1.0),
    ///     (x: -2.0, y: 1.0),
    ///     (x: -2.0, y: -1.0),
    ///     (x: 1.0, y: -1.0),
    ///   ]
    /// ]);
    ///
    /// assert_eq!(
    ///     // ( 2.0 * (1.0, 0.5) + 6.0 * (-0.5, 0.0) ) / 8.0
    ///     Some(point!(x: -0.125, y: 0.125)),
    ///     multi_polygon.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for Rect<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    /// [`Rect`] 的质心是其 [`Point`] 的平均值
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{Rect, point};
    ///
    /// let rect = Rect::new(
    ///   point!(x: 0.0f32, y: 0.0),
    ///   point!(x: 1.0, y: 1.0),
    /// );
    ///
    /// assert_eq!(
    ///     point!(x: 0.5, y: 0.5),
    ///     rect.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        self.center().into()
    }
}

impl<T> Centroid for Triangle<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    /// [`Triangle`] 的质心是其 [`Point`] 的平均值
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{Triangle, coord, point};
    ///
    /// let triangle = Triangle::new(
    ///   coord!(x: 0.0f32, y: -1.0),
    ///   coord!(x: 3.0, y: 0.0),
    ///   coord!(x: 0.0, y: 1.0),
    /// );
    ///
    /// assert_eq!(
    ///     point!(x: 1.0, y: 0.0),
    ///     triangle.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_triangle(self);
        operation
            .centroid()
            .expect("triangle cannot have an empty centroid")
    }
}

impl<T> Centroid for Point<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    /// [`Point`] 的质心就是该点本身
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::point;
    ///
    /// let point = point!(x: 1.0f32, y: 2.0);
    ///
    /// assert_eq!(
    ///     point!(x: 1.0f32, y: 2.0),
    ///     point.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        *self
    }
}

impl<T> Centroid for MultiPoint<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// [`MultiPoint`] 的质心是所有 [`Point`] 的平均值
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{MultiPoint, Point};
    ///
    /// let empty: Vec<Point> = Vec::new();
    /// let empty_multi_points: MultiPoint<_> = empty.into();
    /// assert_eq!(empty_multi_points.centroid(), None);
    ///
    /// let points: MultiPoint<_> = vec![(5., 1.), (1., 3.), (3., 2.)].into();
    /// assert_eq!(points.centroid(), Some(Point::new(3., 2.)));
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_point(self);
        operation.centroid()
    }
}

impl<T> Centroid for Geometry<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    crate::geometry_delegate_impl! {
        /// [`Geometry`] 的质心是其枚举变体的质心
        ///
        /// # 示例
        ///
        /// ```
        /// use geo::Centroid;
        /// use geo::{Geometry, Rect, point};
        ///
        /// let rect = Rect::new(
        ///   point!(x: 0.0f32, y: 0.0),
        ///   point!(x: 1.0, y: 1.0),
        /// );
        /// let geometry = Geometry::from(rect.clone());
        ///
        /// assert_eq!(
        ///     Some(rect.centroid()),
        ///     geometry.centroid(),
        /// );
        ///
        /// assert_eq!(
        ///     Some(point!(x: 0.5, y: 0.5)),
        ///     geometry.centroid(),
        /// );
        /// ```
        fn centroid(&self) -> Self::Output;
    }
}

impl<T> Centroid for GeometryCollection<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// [`GeometryCollection`] 的质心是元素质心的平均值，按其元素的面积加权。
    ///
    /// 请注意，这意味着在计算质心时，不考虑没有面积的元素。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Centroid;
    /// use geo::{Geometry, GeometryCollection, Rect, Triangle, point, coord};
    ///
    /// let rect_geometry = Geometry::from(Rect::new(
    ///   point!(x: 0.0f32, y: 0.0),
    ///   point!(x: 1.0, y: 1.0),
    /// ));
    ///
    /// let triangle_geometry = Geometry::from(Triangle::new(
    ///     coord!(x: 0.0f32, y: -1.0),
    ///     coord!(x: 3.0, y: 0.0),
    ///     coord!(x: 0.0, y: 1.0),
    /// ));
    ///
    /// let point_geometry = Geometry::from(
    ///   point!(x: 12351.0, y: 129815.0)
    /// );
    ///
    /// let geometry_collection = GeometryCollection::new_from(
    ///   vec![
    ///     rect_geometry,
    ///     triangle_geometry,
    ///     point_geometry
    ///   ]
    /// );
    ///
    /// assert_eq!(
    ///     Some(point!(x: 0.875, y: 0.125)),
    ///     geometry_collection.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_geometry_collection(self);
        operation.centroid()
    }
}

struct CentroidOperation<T: GeoFloat>(Option<WeightedCentroid<T>>);
impl<T: GeoFloat> CentroidOperation<T> {
    fn new() -> Self {
        CentroidOperation(None)
    }

    fn centroid(&self) -> Option<Point<T>> {
        self.0.as_ref().map(|weighted_centroid| {
            Point::from(weighted_centroid.accumulated / weighted_centroid.weight)
        })
    }

    fn centroid_dimensions(&self) -> Dimensions {
        self.0
            .as_ref()
            .map(|weighted_centroid| weighted_centroid.dimensions)
            .unwrap_or(Empty)
    }

    fn add_coord(&mut self, coord: Coord<T>) {
        self.add_centroid(ZeroDimensional, coord, T::one());
    }

    fn add_line(&mut self, line: &Line<T>) {
        match line.dimensions() {
            ZeroDimensional => self.add_coord(line.start),
            OneDimensional => self.add_centroid(
                OneDimensional,
                line.centroid().0,
                line.length::<Euclidean>(),
            ),
            _ => unreachable!("Line must be zero or one dimensional"),
        }
    }

    fn add_line_string(&mut self, line_string: &LineString<T>) {
        if self.centroid_dimensions() > OneDimensional {
            return;
        }

        if line_string.0.len() == 1 {
            self.add_coord(line_string.0[0]);
            return;
        }

        for line in line_string.lines() {
            self.add_line(&line);
        }
    }

    fn add_multi_line_string(&mut self, multi_line_string: &MultiLineString<T>) {
        if self.centroid_dimensions() > OneDimensional {
            return;
        }

        for element in &multi_line_string.0 {
            self.add_line_string(element);
        }
    }

    fn add_polygon(&mut self, polygon: &Polygon<T>) {
        // 完全被其内部环覆盖的多边形，面积为零，并且表示一种独特的退化线字符串，无法通过对 `self` 的累加来处理。
        // 相反，我们执行一个子操作，检查结果，然后仅将结果合并到 `self` 中。

        let mut exterior_operation = CentroidOperation::new();
        exterior_operation.add_ring(polygon.exterior());

        let mut interior_operation = CentroidOperation::new();
        for interior in polygon.interiors() {
            interior_operation.add_ring(interior);
        }

        if let Some(exterior_weighted_centroid) = exterior_operation.0 {
            let mut poly_weighted_centroid = exterior_weighted_centroid;
            if let Some(interior_weighted_centroid) = interior_operation.0 {
                poly_weighted_centroid.sub_assign(interior_weighted_centroid);
                if poly_weighted_centroid.weight.is_zero() {
                    // 一个面积为零的多边形，其内部完全覆盖其外部，退化为线字符串
                    self.add_line_string(polygon.exterior());
                    return;
                }
            }
            self.add_weighted_centroid(poly_weighted_centroid);
        }
    }

    fn add_multi_point(&mut self, multi_point: &MultiPoint<T>) {
        if self.centroid_dimensions() > ZeroDimensional {
            return;
        }

        for element in &multi_point.0 {
            self.add_coord(element.0);
        }
    }

    fn add_multi_polygon(&mut self, multi_polygon: &MultiPolygon<T>) {
        for element in &multi_polygon.0 {
            self.add_polygon(element);
        }
    }

    fn add_geometry_collection(&mut self, geometry_collection: &GeometryCollection<T>) {
        for element in &geometry_collection.0 {
            self.add_geometry(element);
        }
    }

    fn add_rect(&mut self, rect: &Rect<T>) {
        match rect.dimensions() {
            ZeroDimensional => self.add_coord(rect.min()),
            OneDimensional => {
                // 退化矩形是线条，将其与处理平面多边形的方式相同
                self.add_line(&Line::new(rect.min(), rect.min()));
                self.add_line(&Line::new(rect.min(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.min()));
            }
            TwoDimensional => {
                self.add_centroid(TwoDimensional, rect.centroid().0, rect.unsigned_area())
            }
            Empty => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_triangle(&mut self, triangle: &Triangle<T>) {
        match triangle.dimensions() {
            ZeroDimensional => self.add_coord(triangle.0),
            OneDimensional => {
                // 退化三角形是线，将其与处理平面多边形的方式相同
                let l0_1 = Line::new(triangle.0, triangle.1);
                let l1_2 = Line::new(triangle.1, triangle.2);
                let l2_0 = Line::new(triangle.2, triangle.0);
                self.add_line(&l0_1);
                self.add_line(&l1_2);
                self.add_line(&l2_0);
            }
            TwoDimensional => {
                let centroid = (triangle.0 + triangle.1 + triangle.2) / T::from(3).unwrap();
                self.add_centroid(TwoDimensional, centroid, triangle.unsigned_area());
            }
            Empty => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_geometry(&mut self, geometry: &Geometry<T>) {
        match geometry {
            Geometry::Point(g) => self.add_coord(g.0),
            Geometry::Line(g) => self.add_line(g),
            Geometry::LineString(g) => self.add_line_string(g),
            Geometry::Polygon(g) => self.add_polygon(g),
            Geometry::MultiPoint(g) => self.add_multi_point(g),
            Geometry::MultiLineString(g) => self.add_multi_line_string(g),
            Geometry::MultiPolygon(g) => self.add_multi_polygon(g),
            Geometry::GeometryCollection(g) => self.add_geometry_collection(g),
            Geometry::Rect(g) => self.add_rect(g),
            Geometry::Triangle(g) => self.add_triangle(g),
        }
    }

    fn add_ring(&mut self, ring: &LineString<T>) {
        debug_assert!(ring.is_closed());

        let area = get_linestring_area(ring);
        if area == T::zero() {
            match ring.dimensions() {
                // 空环不对质心有贡献
                Empty => {}
                // 退化环是点
                ZeroDimensional => self.add_coord(ring[0]),
                // 零面积环是线字符串
                _ => self.add_line_string(ring),
            }
            return;
        }

        // 由于面积非零，我们知道环至少有一个点
        let shift = ring.0[0];

        let accumulated_coord = ring.lines().fold(Coord::zero(), |accum, line| {
            use crate::MapCoords;
            let line = line.map_coords(|c| c - shift);
            let tmp = line.determinant();
            accum + (line.end + line.start) * tmp
        });
        let six = T::from(6).unwrap();
        let centroid = accumulated_coord / (six * area) + shift;
        let weight = area.abs();
        self.add_centroid(TwoDimensional, centroid, weight);
    }

    fn add_centroid(&mut self, dimensions: Dimensions, centroid: Coord<T>, weight: T) {
        let weighted_centroid = WeightedCentroid {
            dimensions,
            weight,
            accumulated: centroid * weight,
        };
        self.add_weighted_centroid(weighted_centroid);
    }

    fn add_weighted_centroid(&mut self, other: WeightedCentroid<T>) {
        match self.0.as_mut() {
            Some(centroid) => centroid.add_assign(other),
            None => self.0 = Some(other),
        }
    }
}

// 用于累加几何体质心或集合几何体质心的聚合状态。
struct WeightedCentroid<T: GeoFloat> {
    weight: T,
    accumulated: Coord<T>,
    /// 几何体集合可以有不同的维度。质心必须根据维度单独考虑。
    ///
    /// 例如，如果我有几个点，添加一个新的 `Point` 将影响它们的质心。
    ///
    /// 然而，由于一个点是零维的，与任何 2D 多边形相比，它是无限小的。因此，点不会影响包含 2D 多边形的任何 GeometryCollection 的质心。
    ///
    /// 因此，在累积一个质心时，我们必须跟踪质心的维度。
    dimensions: Dimensions,
}

impl<T: GeoFloat> WeightedCentroid<T> {
    fn add_assign(&mut self, b: WeightedCentroid<T>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated + b.accumulated;
                self.weight = self.weight + b.weight;
            }
        }
    }

    fn sub_assign(&mut self, b: WeightedCentroid<T>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated - b.accumulated;
                self.weight = self.weight - b.weight;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, line_string, point, polygon, wkt};

    /// 用于创建坐标的小帮助函数
    fn c<T: GeoFloat>(x: T, y: T) -> Coord<T> {
        coord! { x: x, y: y }
    }

    /// 用于创建点的小帮助函数
    fn p<T: GeoFloat>(x: T, y: T) -> Point<T> {
        point! { x: x, y: y }
    }

    // 测试：LineString 的质心
    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = line_string![];
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let coord = coord! {
            x: 40.02f64,
            y: 116.34,
        };
        let linestring = line_string![coord];
        let centroid = linestring.centroid();
        assert_eq!(centroid, Some(Point::from(coord)));
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_eq!(linestring.centroid(), Some(point!(x: 6., y: 1. )));
    }
    #[test]
    fn linestring_with_repeated_point_test() {
        let l1 = LineString::from(vec![p(1., 1.), p(1., 1.), p(1., 1.)]);
        assert_eq!(l1.centroid(), Some(p(1., 1.)));

        let l2 = LineString::from(vec![p(2., 2.), p(2., 2.), p(2., 2.)]);
        let mls = MultiLineString::new(vec![l1, l2]);
        assert_eq!(mls.centroid(), Some(p(1.5, 1.5)));
    }
    // 测试：MultiLineString 的质心
    #[test]
    fn empty_multilinestring_test() {
        let mls: MultiLineString = MultiLineString::new(vec![]);
        let centroid = mls.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn multilinestring_with_empty_line_test() {
        let mls: MultiLineString = MultiLineString::new(vec![line_string![]]);
        let centroid = mls.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn multilinestring_length_0_test() {
        let coord = coord! {
            x: 40.02f64,
            y: 116.34,
        };
        let mls: MultiLineString = MultiLineString::new(vec![
            line_string![coord],
            line_string![coord],
            line_string![coord],
        ]);
        assert_relative_eq!(mls.centroid().unwrap(), Point::from(coord));
    }
    #[test]
    fn multilinestring_one_line_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        let mls: MultiLineString = MultiLineString::new(vec![linestring]);
        assert_relative_eq!(mls.centroid().unwrap(), point! { x: 6., y: 1. });
    }
    #[test]
    fn multilinestring_test() {
        let mls = wkt! {
            MULTILINESTRING(
                (0.0 0.0,1.0 10.0),
                (1.0 10.0,2.0 0.0,3.0 1.0),
                (-12.0 -100.0,7.0 8.0)
            )
        };
        assert_relative_eq!(
            mls.centroid().unwrap(),
            point![x: -1.9097834383655845, y: -37.683866439745714]
        );
    }
    // 测试：多边形的质心
    #[test]
    fn empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert!(poly.centroid().is_none());
    }
    #[test]
    fn polygon_one_point_test() {
        let p = point![ x: 2., y: 1. ];
        let poly = polygon![p.0];
        assert_relative_eq!(poly.centroid().unwrap(), p);
    }

    #[test]
    fn centroid_polygon_numerical_stability() {
        let polygon = {
            use std::f64::consts::PI;
            const NUM_VERTICES: usize = 10;
            const ANGLE_INC: f64 = 2. * PI / NUM_VERTICES as f64;

            Polygon::new(
                (0..NUM_VERTICES)
                    .map(|i| {
                        let angle = i as f64 * ANGLE_INC;
                        coord! {
                            x: angle.cos(),
                            y: angle.sin(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .into(),
                vec![],
            )
        };

        let centroid = polygon.centroid().unwrap();

        let shift = coord! { x: 1.5e8, y: 1.5e8 };

        use crate::map_coords::MapCoords;
        let polygon = polygon.map_coords(|c| c + shift);

        let new_centroid = polygon.centroid().unwrap().map_coords(|c| c - shift);
        debug!("centroid {:?}", centroid.0);
        debug!("new_centroid {:?}", new_centroid.0);
        assert_relative_eq!(centroid.0.x, new_centroid.0.x, max_relative = 0.0001);
        assert_relative_eq!(centroid.0.y, new_centroid.0.y, max_relative = 0.0001);
    }

    #[test]
    fn polygon_test() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 2., y: 0.),
            (x: 2., y: 2.),
            (x: 0., y: 2.),
            (x: 0., y: 0.)
        ];
        assert_relative_eq!(poly.centroid().unwrap(), point![x:1., y:1.]);
    }
    #[test]
    fn polygon_hole_test() {
        // 六边形
        let p1 = wkt! { POLYGON(
            (5.0 1.0,4.0 2.0,4.0 3.0,5.0 4.0,6.0 4.0,7.0 3.0,7.0 2.0,6.0 1.0,5.0 1.0),
            (5.0 1.3,5.5 2.0,6.0 1.3,5.0 1.3),
            (5.0 2.3,5.5 3.0,6.0 2.3,5.0 2.3)
        ) };
        let centroid = p1.centroid().unwrap();
        assert_relative_eq!(centroid, point!(x: 5.5, y: 2.5518518518518523));
    }
    #[test]
    fn flat_polygon_test() {
        let poly = wkt! { POLYGON((0. 1.,1. 1.,0. 1.)) };
        assert_eq!(poly.centroid(), Some(p(0.5, 1.)));
    }
    #[test]
    fn multi_poly_with_flat_polygon_test() {
        let multipoly = wkt! { MULTIPOLYGON(((0. 0.,1. 0.,0. 0.))) };
        assert_eq!(multipoly.centroid(), Some(p(0.5, 0.)));
    }
    #[test]
    fn multi_poly_with_multiple_flat_polygon_test() {
        let multipoly = wkt! { MULTIPOLYGON(
            ((1. 1.,1. 3.,1. 1.)),
            ((2. 2.,6. 2.,2. 2.))
        )};

        assert_eq!(multipoly.centroid(), Some(p(3., 2.)));
    }
    #[test]
    fn multi_poly_with_only_points_test() {
        let p1 = wkt! { POLYGON((1. 1.,1. 1.,1. 1.)) };
        assert_eq!(p1.centroid(), Some(p(1., 1.)));

        let multipoly = wkt! { MULTIPOLYGON(
            ((1. 1.,1. 1.,1. 1.)),
            ((2. 2., 2. 2.,2. 2.))
        ) };
        assert_eq!(multipoly.centroid(), Some(p(1.5, 1.5)));
    }
    #[test]
    fn multi_poly_with_one_ring_and_one_real_poly() {
        // 如果多边形是由一个“正常”多边形（面积不为空）和一个环（面积为空的多边形）组成的
        // 多边形的质心是“正常”多边形的质心
        let normal = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 3.), p(3., 1.), p(1., 1.)]),
            vec![],
        );
        let flat = Polygon::new(
            LineString::from(vec![p(2., 2.), p(6., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon::new(vec![normal.clone(), flat]);
        assert_eq!(multipoly.centroid(), normal.centroid());
    }
    #[test]
    fn polygon_flat_interior_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString::from(vec![p(0., 0.), p(0., 1.), p(0., 0.)])],
        );
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn empty_interior_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString::new(vec![])],
        );
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn polygon_ring_test() {
        let square = LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]);
        let poly = Polygon::new(square.clone(), vec![square]);
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn polygon_cell_test() {
        // 测试面积为零的多边形的质心
        // 这是一个包含两个内部多边形，使外部划分的多边形
        let square = LineString::from(vec![p(0., 0.), p(0., 2.), p(2., 2.), p(2., 0.), p(0., 0.)]);
        let bottom = LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 1.), p(0., 1.), p(0., 0.)]);
        let top = LineString::from(vec![p(0., 1.), p(2., 1.), p(2., 2.), p(0., 2.), p(0., 1.)]);
        let poly = Polygon::new(square, vec![top, bottom]);
        assert_eq!(poly.centroid(), Some(p(1., 1.)));
    }
    // 测试：多重多边形的质心
    #[test]
    fn empty_multipolygon_polygon_test() {
        assert!(MultiPolygon::<f64>::new(Vec::new()).centroid().is_none());
    }

    #[test]
    fn multipolygon_one_polygon_test() {
        let linestring =
            LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(MultiPolygon::new(vec![poly]).centroid(), Some(p(1., 1.)));
    }
    #[test]
    fn multipolygon_two_polygons_test() {
        let linestring =
            LineString::from(vec![p(2., 1.), p(5., 1.), p(5., 3.), p(2., 3.), p(2., 1.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring =
            LineString::from(vec![p(7., 1.), p(8., 1.), p(8., 2.), p(7., 2.), p(7., 1.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        let centroid = MultiPolygon::new(vec![poly1, poly2]).centroid().unwrap();
        assert_relative_eq!(
            centroid,
            point![x: 4.071428571428571, y: 1.9285714285714286]
        );
    }
    #[test]
    fn multipolygon_two_polygons_of_opposite_clockwise_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString::from(vec![(0., 0.), (-2., 0.), (-2., 2.), (0., 2.), (0., 0.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        assert_relative_eq!(
            MultiPolygon::new(vec![poly1, poly2]).centroid().unwrap(),
            point![x: 0., y: 1.]
        );
    }
    #[test]
    fn bounding_rect_test() {
        let bounding_rect = Rect::new(coord! { x: 0., y: 50. }, coord! { x: 4., y: 100. });
        let point = point![x: 2., y: 75.];
        assert_eq!(point, bounding_rect.centroid());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(c(0., 1.), c(1., 3.));
        assert_eq!(line1.centroid(), point![x: 0.5, y: 2.]);
    }
    #[test]
    fn collection_weighting() {
        let p0 = point!(x: 0.0, y: 0.0);
        let p1 = point!(x: 2.0, y: 0.0);
        let p2 = point!(x: 2.0, y: 2.0);
        let p3 = point!(x: 0.0, y: 2.0);

        let multi_point = MultiPoint::new(vec![p0, p1, p2, p3]);
        assert_eq!(multi_point.centroid().unwrap(), point!(x: 1.0, y: 1.0));

        let collection =
            GeometryCollection::new_from(vec![MultiPoint::new(vec![p1, p2, p3]).into(), p0.into()]);

        assert_eq!(collection.centroid().unwrap(), point!(x: 1.0, y: 1.0));
    }
    #[test]
    fn triangles() {
        // 普通三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 0.), c(1.5, 3.)).centroid(),
            point!(x: 1.5, y: 1.0)
        );

        // 平面三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 0.), c(1., 0.)).centroid(),
            point!(x: 1.5, y: 0.0)
        );

        // 非轴对齐的平面三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 3.), c(1., 1.)).centroid(),
            point!(x: 1.5, y: 1.5)
        );

        // 带有一些重复点的三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(0., 0.), c(1., 0.)).centroid(),
            point!(x: 0.5, y: 0.0)
        );

        // 所有点重复的三角形
        assert_eq!(
            Triangle::new(c(0., 0.5), c(0., 0.5), c(0., 0.5)).centroid(),
            point!(x: 0., y: 0.5)
        )
    }

    #[test]
    fn degenerate_triangle_like_ring() {
        let triangle = Triangle::new(c(0., 0.), c(1., 1.), c(2., 2.));
        let poly: Polygon<_> = triangle.into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection::new_from(vec![triangle.into(), line.into()]);
        let g2 = GeometryCollection::new_from(vec![poly.into(), line.into()]);
        assert_eq!(g1.centroid(), g2.centroid());
    }

    #[test]
    fn degenerate_rect_like_ring() {
        let rect = Rect::new(c(0., 0.), c(0., 4.));
        let poly: Polygon<_> = rect.into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection::new_from(vec![rect.into(), line.into()]);
        let g2 = GeometryCollection::new_from(vec![poly.into(), line.into()]);
        assert_eq!(g1.centroid(), g2.centroid());
    }

    #[test]
    fn rectangles() {
        // 普通矩形
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 4.)).centroid(),
            point!(x: 2.0, y: 2.0)
        );

        // 平面矩形
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 0.)).centroid(),
            point!(x: 2.0, y: 0.0)
        );

        // 所有点都重复的矩形
        assert_eq!(
            Rect::new(c(4., 4.), c(4., 4.)).centroid(),
            point!(x: 4., y: 4.)
        );

        // 带矩形的集合
        let mut collection = GeometryCollection::new_from(vec![
            p(0., 0.).into(),
            p(6., 0.).into(),
            p(6., 6.).into(),
        ]);
        // 合理性检查
        assert_eq!(collection.centroid().unwrap(), point!(x: 4., y: 2.));

        // 将 0 维矩形视作点
        collection.0.push(Rect::new(c(0., 6.), c(0., 6.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 3., y: 3.));

        // 将 1 维矩形视作线。因为线在整个集合中有更高的维度，
        // 它的质心会覆盖集合中的其他元素。
        collection.0.push(Rect::new(c(0., 0.), c(0., 2.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 0., y: 1.));

        // 2 维具有比集合中其他内容更高的维度，因此其质心覆盖
        // 集合中的其他元素。
        collection
            .0
            .push(Rect::new(c(10., 10.), c(11., 11.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 10.5, y: 10.5));
    }
}
