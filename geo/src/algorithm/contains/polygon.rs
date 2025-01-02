use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};
use crate::{HasDimensions, Relate};

// ┌───────────────────────────┐
// │ Polygon 的实现部分        │
// └───────────────────────────┘
impl<T> Contains<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        use crate::coordinate_position::{CoordPos, CoordinatePosition};

        // 检查坐标位置是否在多边形内部
        self.coordinate_position(coord) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        // 通过坐标检查点是否在多边形内
        self.contains(&p.0)
    }
}

// 使用宏定义包含关系
impl_contains_from_relate!(Polygon<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Polygon<T>);

// ┌─────────────────────────────┐
// │ MultiPolygon 的实现部分    │
// └─────────────────────────────┘

impl<T> Contains<Coord<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        // 任何一个多边形包含坐标则返回 true
        self.iter().any(|poly| poly.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        // 使用坐标的实现来检查点
        self.contains(&p.0)
    }
}

impl<T: GeoNum> Contains<MultiPoint<T>> for MultiPolygon<T> {
    fn contains(&self, rhs: &MultiPoint<T>) -> bool {
        // 如果 MultiPolygon 或 MultiPoint 为空就返回 false
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        // 所有的点都必须包含在 MultiPolygon 中
        rhs.iter().all(|point| self.contains(point))
    }
}

impl<F> Contains<Line<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Line<F>) -> bool {
        // 使用相对关系检查线是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<LineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &LineString<F>) -> bool {
        // 使用相对关系检查线串是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiLineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiLineString<F>) -> bool {
        // 使用相对关系检查多线串是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Polygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Polygon<F>) -> bool {
        // 使用相对关系检查多边形是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiPolygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiPolygon<F>) -> bool {
        // 使用相对关系检查多多边形是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<GeometryCollection<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &GeometryCollection<F>) -> bool {
        // 使用相对关系检查几何集合是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Rect<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Rect<F>) -> bool {
        // 使用相对关系检查矩形是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Triangle<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Triangle<F>) -> bool {
        // 使用相对关系检查三角形是否在 MultiPolygon 内
        rhs.relate(self).is_within()
    }
}
