use super::Intersects;
use crate::*;

// 从 Coord<T> 的 blanket 实现
impl<T, G> Intersects<G> for Point<T>
where
    T: CoordNum,
    Coord<T>: Intersects<G>,
{
    // 检查点是否与几何体 rhs 相交
    fn intersects(&self, rhs: &G) -> bool {
        self.0.intersects(rhs)
    }
}

// 从 Point<T> 的 blanket 实现
impl<T, G> Intersects<G> for MultiPoint<T>
where
    T: CoordNum,
    Point<T>: Intersects<G>,
{
    // 检查多点集合中的任意一个点是否与几何体 rhs 相交
    fn intersects(&self, rhs: &G) -> bool {
        self.iter().any(|p| p.intersects(rhs))
    }
}

// 对称实现：Coord<T> 与 MultiPoint<T> 的相交性
symmetric_intersects_impl!(Coord<T>, MultiPoint<T>);
// 对称实现：Line<T> 与 MultiPoint<T> 的相交性
symmetric_intersects_impl!(Line<T>, MultiPoint<T>);
// 对称实现：Triangle<T> 与 MultiPoint<T> 的相交性
symmetric_intersects_impl!(Triangle<T>, MultiPoint<T>);
// 对称实现：Polygon<T> 与 MultiPoint<T> 的相交性
symmetric_intersects_impl!(Polygon<T>, MultiPoint<T>);
