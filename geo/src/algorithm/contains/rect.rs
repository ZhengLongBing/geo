use geo_types::CoordFloat;

use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::{geometry::*, Area, CoordsIter, HasDimensions, Intersects};
use crate::{CoordNum, GeoFloat};

// ┌──────────────────────────┐
// │ Rect 的实现部分           │
// └──────────────────────────┘

impl<T> Contains<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        coord.x > self.min().x
            && coord.x < self.max().x
            && coord.y > self.min().y
            && coord.y < self.max().y
    }
}

impl<T> Contains<Point<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        // TODO: 检查退化矩形（即线或点）
        // LineString 的所有点必须在多边形内？
        self.min().x <= other.min().x
            && self.max().x >= other.max().x
            && self.min().y <= other.min().y
            && self.max().y >= other.max().y
    }
}

impl<T> Contains<Polygon<T>> for Rect<T>
where
    T: CoordFloat,
{
    fn contains(&self, rhs: &Polygon<T>) -> bool {
        // 多边形不能是空的
        if rhs.is_empty() {
            return false;
        }

        // 多边形的点不能在矩形之外
        let mut points_inside = 0;
        for c in rhs.exterior_coords_iter() {
            if !self.intersects(&c) {
                return false;
            }
            if self.contains(&c) {
                points_inside += 1;
            }
        }

        // 多边形不能完全位于矩形的边界内。
        // 换句话说：多边形内部至少有一个点位于矩形的内部。
        // 由于我们知道矩形是凸的，我们只需确保
        // 多边形内部至少有一个点位于矩形内部，
        // 或者多边形内部不为空，这种情况下它肯定会与
        // 矩形的内部相交。
        if points_inside == 0 && rhs.signed_area().is_zero() {
            return false;
        }

        true
    }
}

impl_contains_from_relate!(Rect<T>, [Line<T>, LineString<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Triangle<T>]);
impl_contains_geometry_for!(Rect<T>);
