use crate::geometry::*;
use crate::Orientation::Collinear;
use crate::{CoordNum, GeoNum, GeometryCow};

/// 几何体可以有0, 1或两个维度。或者，在几何体是[`empty`](#is_empty)的情况下，存在一个特殊的`Empty`维度。
///
/// # 示例
///
/// ```
/// use geo_types::{Point, Rect, line_string};
/// use geo::dimensions::{HasDimensions, Dimensions};
///
/// let point = Point::new(0.0, 5.0);
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 5.0, y: 5.0), (x: 0.0, y: 5.0)];
/// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
/// assert_eq!(Dimensions::ZeroDimensional, point.dimensions());
/// assert_eq!(Dimensions::OneDimensional, line_string.dimensions());
/// assert_eq!(Dimensions::TwoDimensional, rect.dimensions());
///
/// assert!(point.dimensions() < line_string.dimensions());
/// assert!(rect.dimensions() > line_string.dimensions());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum Dimensions {
    /// 某些几何体，如`MultiPoint`或`GeometryCollection`可能没有元素 - 因此没有维度。注意，这与像`Point`这样的`ZeroDimensional`是不同的。
    Empty,
    /// 点的维度
    ZeroDimensional,
    /// 线或曲线的维度
    OneDimensional,
    /// 表面的维度
    TwoDimensional,
}

/// 操作几何体的维度。
pub trait HasDimensions {
    /// 某些几何体，如`MultiPoint`，可以没有坐标 - 我们称这些为`empty`。
    ///
    /// 像`Point`和`Rect`这样的类型，由于构造上至少有一个坐标，因此永远不会被视为空。
    /// ```
    /// use geo_types::{Point, coord, LineString};
    /// use geo::HasDimensions;
    ///
    /// let line_string = LineString::new(vec![
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 10., y: 0. },
    /// ]);
    /// assert!(!line_string.is_empty());
    ///
    /// let empty_line_string: LineString = LineString::new(vec![]);
    /// assert!(empty_line_string.is_empty());
    ///
    /// let point = Point::new(0.0, 0.0);
    /// assert!(!point.is_empty());
    /// ```
    fn is_empty(&self) -> bool;

    /// 某些几何体的维度是固定的，例如一个点总是有0维。但是对于其他类型，维度取决于特定的几何体实例 - 例如，典型的`Rect`是2维的，但可以创建退化的`Rect`，它们可能具有1或0维。
    ///
    /// ## 示例
    ///
    /// ```
    /// use geo_types::{GeometryCollection, Rect, Point};
    /// use geo::dimensions::{Dimensions, HasDimensions};
    ///
    /// // 正常矩形
    /// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::TwoDimensional, rect.dimensions());
    ///
    /// // 高度为零的“矩形”退化为一条线
    /// let degenerate_line_rect = Rect::new((0.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::OneDimensional, degenerate_line_rect.dimensions());
    ///
    /// // 高度和宽度都为零的“矩形”退化为一点
    /// let degenerate_point_rect = Rect::new((10.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::ZeroDimensional, degenerate_point_rect.dimensions());
    ///
    /// // 集合继承其元素的最大维度
    /// let geometry_collection = GeometryCollection::new_from(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::OneDimensional, geometry_collection.dimensions());
    ///
    /// let point = Point::new(10.0, 10.0);
    /// assert_eq!(Dimensions::ZeroDimensional, point.dimensions());
    ///
    /// // `Empty`维度与0维是不同的，并且小于0维
    /// let empty_collection = GeometryCollection::<f32>::new_from(vec![]);
    /// assert_eq!(Dimensions::Empty, empty_collection.dimensions());
    /// assert!(empty_collection.dimensions() < point.dimensions());
    /// ```
    fn dimensions(&self) -> Dimensions;

    /// OGC-SFA中使用的`Geometry`边界的维度。
    ///
    /// ## 示例
    ///
    /// ```
    /// use geo_types::{GeometryCollection, Rect, Point};
    /// use geo::dimensions::{Dimensions, HasDimensions};
    ///
    /// // 点没有边界
    /// let point = Point::new(10.0, 10.0);
    /// assert_eq!(Dimensions::Empty, point.boundary_dimensions());
    ///
    /// // 一个典型的矩形有一个*线*（一维）的边界
    /// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::OneDimensional, rect.boundary_dimensions());
    ///
    /// // 高度为零的“矩形”退化为一条线，其边界为两个点
    /// let degenerate_line_rect = Rect::new((0.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::ZeroDimensional, degenerate_line_rect.boundary_dimensions());
    ///
    /// // 高度和宽度都为零的“矩形”退化为一点，点没有边界
    /// let degenerate_point_rect = Rect::new((10.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::Empty, degenerate_point_rect.boundary_dimensions());
    ///
    /// // 集合继承其元素的最大维度
    /// let geometry_collection = GeometryCollection::new_from(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::ZeroDimensional, geometry_collection.boundary_dimensions());
    ///
    /// let geometry_collection = GeometryCollection::<f32>::new_from(vec![]);
    /// assert_eq!(Dimensions::Empty, geometry_collection.boundary_dimensions());
    /// ```
    fn boundary_dimensions(&self) -> Dimensions;
}

impl<C: GeoNum> HasDimensions for Geometry<C> {
    crate::geometry_delegate_impl! {
        fn is_empty(&self) -> bool;
        fn dimensions(&self) -> Dimensions;
        fn boundary_dimensions(&self) -> Dimensions;
    }
}

impl<C: GeoNum> HasDimensions for GeometryCow<'_, C> {
    crate::geometry_cow_delegate_impl! {
        fn is_empty(&self) -> bool;
        fn dimensions(&self) -> Dimensions;
        fn boundary_dimensions(&self) -> Dimensions;
    }
}

impl<C: CoordNum> HasDimensions for Point<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordNum> HasDimensions for Line<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // 退化线是一个点
            Dimensions::ZeroDimensional
        } else {
            Dimensions::OneDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // 退化线是一个点，点没有边界
            Dimensions::Empty
        } else {
            Dimensions::ZeroDimensional
        }
    }
}

impl<C: CoordNum> HasDimensions for LineString<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        let first = self.0[0];
        if self.0.iter().any(|&coord| first != coord) {
            Dimensions::OneDimensional
        } else {
            // 所有坐标都相同 - 即一个点
            Dimensions::ZeroDimensional
        }
    }

    /// ```
    /// use geo_types::line_string;
    /// use geo::dimensions::{HasDimensions, Dimensions};
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.)];
    /// assert_eq!(Dimensions::ZeroDimensional, ls.boundary_dimensions());
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.), (x: 0., y: 0.)];
    /// assert_eq!(Dimensions::Empty, ls.boundary_dimensions());
    ///```
    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string不能是二维的"),
        }
    }
}

impl<C: CoordNum> HasDimensions for Polygon<C> {
    fn is_empty(&self) -> bool {
        self.exterior().is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        use crate::CoordsIter;
        let mut coords = self.exterior_coords_iter();

        let Some(first) = coords.next() else {
            // 没有坐标 - 多边形为空
            return Dimensions::Empty;
        };

        let Some(second) = coords.find(|next| *next != first) else {
            // 多边形内所有坐标都是相同的点
            return Dimensions::ZeroDimensional;
        };

        let Some(_third) = coords.find(|next| *next != first && *next != second) else {
            // 多边形中只有两个不同的坐标 - 它已退化为一条线
            return Dimensions::OneDimensional;
        };

        Dimensions::TwoDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: CoordNum> HasDimensions for MultiPoint<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordNum> HasDimensions for MultiLineString<C> {
    fn is_empty(&self) -> bool {
        self.iter().all(LineString::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for line in &self.0 {
            match line.dimensions() {
                Dimensions::Empty => {}
                Dimensions::ZeroDimensional => max = Dimensions::ZeroDimensional,
                Dimensions::OneDimensional => {
                    // 提前返回因为我们知道多线字符串的维度不能超过1维
                    return Dimensions::OneDimensional;
                }
                Dimensions::TwoDimensional => unreachable!("MultiLineString不能是二维"),
            }
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string不能是二维"),
        }
    }
}

impl<C: CoordNum> HasDimensions for MultiPolygon<C> {
    fn is_empty(&self) -> bool {
        self.iter().all(Polygon::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let dimensions = geom.dimensions();
            if dimensions == Dimensions::TwoDimensional {
                // 短路，因为我们知道没有更大的可能性
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: GeoNum> HasDimensions for GeometryCollection<C> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.iter().all(Geometry::is_empty)
        }
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let dimensions = geom.dimensions();
            if dimensions == Dimensions::TwoDimensional {
                // 短路，因为我们知道没有更大的可能性
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let d = geom.boundary_dimensions();

            if d == Dimensions::OneDimensional {
                return Dimensions::OneDimensional;
            }

            max = max.max(d);
        }
        max
    }
}

impl<C: CoordNum> HasDimensions for Rect<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.min() == self.max() {
            // 退化矩形是一个点
            Dimensions::ZeroDimensional
        } else if self.min().x == self.max().x || self.min().y == self.max().y {
            // 退化矩形是一条线
            Dimensions::OneDimensional
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("即使是退化的矩形也应该至少是0维")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: GeoNum> HasDimensions for Triangle<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        use crate::Kernel;
        if Collinear == C::Ker::orient2d(self.0, self.1, self.2) {
            if self.0 == self.1 && self.1 == self.2 {
                // 退化三角形是一个点
                Dimensions::ZeroDimensional
            } else {
                // 退化三角形是一条线
                Dimensions::OneDimensional
            }
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("即使是退化的三角形也应该至少是0维")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: Coord = crate::coord!(x: 1.0, y: 1.0);
    use crate::wkt;

    #[test]
    fn point() {
        assert_eq!(
            Dimensions::ZeroDimensional,
            wkt!(POINT(1.0 1.0)).dimensions()
        );
    }

    #[test]
    fn line_string() {
        assert_eq!(
            Dimensions::OneDimensional,
            wkt!(LINESTRING(1.0 1.0,2.0 2.0,3.0 3.0)).dimensions()
        );
    }

    #[test]
    fn polygon() {
        assert_eq!(
            Dimensions::TwoDimensional,
            wkt!(POLYGON((1.0 1.0,2.0 2.0,3.0 3.0,1.0 1.0))).dimensions()
        );
    }

    #[test]
    fn multi_point() {
        assert_eq!(
            Dimensions::ZeroDimensional,
            wkt!(MULTIPOINT(1.0 1.0)).dimensions()
        );
    }

    #[test]
    fn multi_line_string() {
        assert_eq!(
            Dimensions::OneDimensional,
            wkt!(MULTILINESTRING((1.0 1.0,2.0 2.0,3.0 3.0))).dimensions()
        );
    }

    #[test]
    fn multi_polygon() {
        assert_eq!(
            Dimensions::TwoDimensional,
            wkt!(MULTIPOLYGON(((1.0 1.0,2.0 2.0,3.0 3.0,1.0 1.0)))).dimensions()
        );
    }

    mod empty {
        use super::*;
        #[test]
        fn empty_line_string() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(LINESTRING EMPTY) as LineString<f64>).dimensions()
            );
        }

        #[test]
        fn empty_polygon() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(POLYGON EMPTY) as Polygon<f64>).dimensions()
            );
        }

        #[test]
        fn empty_multi_point() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTIPOINT EMPTY) as MultiPoint<f64>).dimensions()
            );
        }

        #[test]
        fn empty_multi_line_string() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTILINESTRING EMPTY) as MultiLineString<f64>).dimensions()
            );
        }

        #[test]
        fn multi_line_string_with_empty_line_string() {
            let empty_line_string = wkt!(LINESTRING EMPTY) as LineString<f64>;
            let multi_line_string = MultiLineString::new(vec![empty_line_string]);
            assert_eq!(Dimensions::Empty, multi_line_string.dimensions());
        }

        #[test]
        fn empty_multi_polygon() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTIPOLYGON EMPTY) as MultiPolygon<f64>).dimensions()
            );
        }

        #[test]
        fn multi_polygon_with_empty_polygon() {
            let empty_polygon = (wkt!(POLYGON EMPTY) as Polygon<f64>);
            let multi_polygon = MultiPolygon::new(vec![empty_polygon]);
            assert_eq!(Dimensions::Empty, multi_polygon.dimensions());
        }
    }

    mod dimensional_collapse {
        use super::*;

        #[test]
        fn line_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                Line::new(ONE, ONE).dimensions()
            );
        }

        #[test]
        fn line_string_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(LINESTRING(1.0 1.0)).dimensions()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(LINESTRING(1.0 1.0,1.0 1.0)).dimensions()
            );
        }

        #[test]
        fn polygon_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(POLYGON((1.0 1.0))).dimensions()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(POLYGON((1.0 1.0,1.0 1.0))).dimensions()
            );
        }

        #[test]
        fn polygon_collapsed_to_line() {
            assert_eq!(
                Dimensions::OneDimensional,
                wkt!(POLYGON((1.0 1.0,2.0 2.0))).dimensions()
            );
        }

        #[test]
        fn multi_line_string_with_line_string_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0))).dimensions()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0,1.0 1.0))).dimensions()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0),(1.0 1.0))).dimensions()
            );
        }

        #[test]
        fn multi_polygon_with_polygon_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0)))).dimensions()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0,1.0 1.0)))).dimensions()
            );
        }

        #[test]
        fn multi_polygon_with_polygon_collapsed_to_line() {
            assert_eq!(
                Dimensions::OneDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0,2.0 2.0)))).dimensions()
            );
        }
    }
}
