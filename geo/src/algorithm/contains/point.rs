use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::{CoordsIter, HasDimensions};
use crate::geometry::*;
use crate::{CoordNum, GeoFloat};

// ┌──────────────────────────────────┐
// │ Point 的实现                      │
// └──────────────────────────────────┘

impl<T> Contains<Coord<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        &self.0 == coord
    }
}

impl<T> Contains<Point<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            // 退化的线作为一个点
            line.start == self.0
        } else {
            false
        }
    }
}

impl<T> Contains<LineString<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, line_string: &LineString<T>) -> bool {
        if line_string.is_empty() {
            return false;
        }
        // 只有退化的 LineString 可以包含在一个点中
        line_string.coords().all(|c| c == &self.0)
    }
}

impl<T> Contains<Polygon<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, polygon: &Polygon<T>) -> bool {
        if polygon.is_empty() {
            return false;
        }
        // 只有退化的 Polygon 可以包含在一个点中
        polygon.coords_iter().all(|coord| coord == self.0)
    }
}

impl<T> Contains<MultiPoint<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_point: &MultiPoint<T>) -> bool {
        if multi_point.is_empty() {
            return false;
        }
        multi_point.iter().all(|point| self.contains(point))
    }
}

impl<T> Contains<MultiLineString<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_line_string: &MultiLineString<T>) -> bool {
        if multi_line_string.is_empty() {
            return false;
        }
        // 只有退化的 MultiLineString 可以包含在一个点中
        multi_line_string
            .iter()
            .all(|line_string| self.contains(line_string))
    }
}

impl<T> Contains<MultiPolygon<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_polygon: &MultiPolygon<T>) -> bool {
        if multi_polygon.is_empty() {
            return false;
        }
        // 只有退化的 MultiPolygon 可以包含在一个点中
        multi_polygon.iter().all(|polygon| self.contains(polygon))
    }
}

impl<T> Contains<GeometryCollection<T>> for Point<T>
where
    T: GeoFloat,
{
    fn contains(&self, geometry_collection: &GeometryCollection<T>) -> bool {
        if geometry_collection.is_empty() {
            return false;
        }
        geometry_collection
            .iter()
            .all(|geometry| self.contains(geometry))
    }
}

impl<T> Contains<Rect<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, rect: &Rect<T>) -> bool {
        // 只有退化的 Rect 可以包含在一个点中
        rect.min() == rect.max() && rect.min() == self.0
    }
}

impl<T> Contains<Triangle<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, triangle: &Triangle<T>) -> bool {
        // 只有退化的 Triangle 可以包含在一个点中
        triangle.0 == triangle.1 && triangle.0 == triangle.2 && triangle.0 == self.0
    }
}

impl_contains_geometry_for!(Point<T>);

// ┌──────────────────────────────────┐
// │ MultiPoint  的实现               │
// └──────────────────────────────────┘

impl_contains_from_relate!(MultiPoint<T>, [Line<T>, LineString<T>, Polygon<T>, MultiLineString<T>, MultiPolygon<T>, MultiPoint<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);

impl<T> Contains<Coord<T>> for MultiPoint<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|c| &c.0 == coord)
    }
}

impl<T> Contains<Point<T>> for MultiPoint<T>
where
    T: CoordNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.iter().any(|c| c == point)
    }
}
