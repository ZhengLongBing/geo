use super::{has_disjoint_bboxes, Intersects};
use crate::BoundingRect;
use crate::*;

// 使用 self.lines().any() 的通用实现。
impl<T, G> Intersects<G> for LineString<T>
where
    T: CoordNum,
    Line<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    // 判断 LineString 是否与几何体 geom 相交
    fn intersects(&self, geom: &G) -> bool {
        if has_disjoint_bboxes(self, geom) {
            return false; // 如果 bounding boxes 不相交，直接返回 false
        }
        self.lines().any(|l| l.intersects(geom)) // 通过遍历每一条线段，判断是否存在相交
    }
}

symmetric_intersects_impl!(Coord<T>, LineString<T>);
symmetric_intersects_impl!(Line<T>, LineString<T>);
symmetric_intersects_impl!(Rect<T>, LineString<T>);
symmetric_intersects_impl!(Triangle<T>, LineString<T>);

// 从 LineString<T> 通用实现
impl<T, G> Intersects<G> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    // 判断 MultiLineString 是否与几何体 rhs 相交
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false; // 如果 bounding boxes 不相交，直接返回 false
        }
        self.iter().any(|p| p.intersects(rhs)) // 通过遍历每一个 LineString，判断是否存在相交
    }
}

symmetric_intersects_impl!(Point<T>, MultiLineString<T>);
symmetric_intersects_impl!(Line<T>, MultiLineString<T>);
symmetric_intersects_impl!(Rect<T>, MultiLineString<T>);
symmetric_intersects_impl!(Triangle<T>, MultiLineString<T>);
