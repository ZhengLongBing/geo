// 引入必要的模块和trait
use super::{has_disjoint_bboxes, Intersects};
use crate::geometry::*;
use crate::geometry_delegate_impl;
use crate::BoundingRect;
use crate::CoordNum;

// 为 Geometry<T> 实现 Intersects<G> trait
impl<T, G> Intersects<G> for Geometry<T>
where
    T: CoordNum,
    Point<T>: Intersects<G>,
    MultiPoint<T>: Intersects<G>,
    Line<T>: Intersects<G>,
    LineString<T>: Intersects<G>,
    MultiLineString<T>: Intersects<G>,
    Triangle<T>: Intersects<G>,
    Rect<T>: Intersects<G>,
    Polygon<T>: Intersects<G>,
    MultiPolygon<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    // 使用宏实现intersects方法
    geometry_delegate_impl! {
        fn intersects(&self, rhs: &G) -> bool;
    }
}

// 实现几何体之间的对称相交性
symmetric_intersects_impl!(Coord<T>, Geometry<T>);
symmetric_intersects_impl!(Line<T>, Geometry<T>);
symmetric_intersects_impl!(Rect<T>, Geometry<T>);
symmetric_intersects_impl!(Triangle<T>, Geometry<T>);
symmetric_intersects_impl!(Polygon<T>, Geometry<T>);

// 为 GeometryCollection<T> 实现 Intersects<G> trait
impl<T, G> Intersects<G> for GeometryCollection<T>
where
    T: CoordNum,
    Geometry<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        // 如果边界盒不相交，则返回false
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // 检查集合中是否有任何几何体与rhs相交
        self.iter().any(|geom| geom.intersects(rhs))
    }
}

// 为几何体集合实现对称相交性
symmetric_intersects_impl!(Coord<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Line<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Rect<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Triangle<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Polygon<T>, GeometryCollection<T>);
