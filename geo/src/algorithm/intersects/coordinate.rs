use super::Intersects;
use crate::*;

impl<T> Intersects<Coord<T>> for Coord<T>
where
    T: CoordNum,
{
    // 判断两个坐标是否相交
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        self == rhs
    }
}

// 另一侧通过 blanket impl 来处理。
impl<T> Intersects<Point<T>> for Coord<T>
where
    T: CoordNum,
{
    // 判断坐标是否与点相交
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == &rhs.0
    }
}
