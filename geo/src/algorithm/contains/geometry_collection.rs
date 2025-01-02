// 导入相关模块和trait
use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

// 为 GeometryCollection<T> 实现 Contains<Coord<T>> trait
impl<T> Contains<Coord<T>> for GeometryCollection<T>
where
    T: GeoNum,
{
    // 检查 GeometryCollection 中是否有几何体包含给定坐标
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|geometry| geometry.contains(coord))
    }
}

// 为 GeometryCollection<T> 实现 Contains<Point<T>> trait
impl<T> Contains<Point<T>> for GeometryCollection<T>
where
    T: GeoNum,
{
    // 检查 GeometryCollection 中是否包含给定点
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

// 使用宏实现针对多个几何类型的包含操作
impl_contains_from_relate!(GeometryCollection<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(GeometryCollection<T>);
