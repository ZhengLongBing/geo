use std::{cmp::Ordering, ops::Deref};

use geo_types::Coord;

use crate::GeoNum;

/// 词典序排序的点。
///
/// [`Coord`] 的包裹，用于按 `x` 排序点，然后是 `y`。
/// 实现了 `Ord` 和 `Eq`，允许在有序集合中使用，
/// 例如 `BinaryHeap`。
///
/// 注意，标量类型 `T` 只需要实现 `PartialOrd`。
/// 因此，除非坐标保证可排序，否则构建这个结构体是逻辑错误的。
#[derive(PartialEq, Clone, Copy)]
pub struct SweepPoint<T: GeoNum>(Coord<T>);

impl<T: GeoNum> std::fmt::Debug for SweepPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SPt")
            .field(&self.0.x)
            .field(&self.0.y)
            .finish()
    }
}

/// 实现按 `x` 然后按 `y` 坐标的词典序排序。
impl<T: GeoNum> PartialOrd for SweepPoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 从 `PartialOrd` 派生 `Ord` 并期望不会失败。
impl<T: GeoNum> Ord for SweepPoint<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.x.total_cmp(&other.0.x) {
            Ordering::Equal => self.0.y.total_cmp(&other.0.y),
            o => o,
        }
    }
}

/// 我们手动派生 `Eq` 以不要求 `T: Eq`。
impl<T: GeoNum> Eq for SweepPoint<T> {}

/// 从可以转换为 `Coord` 的类型进行转换。
impl<T: GeoNum, X: Into<Coord<T>>> From<X> for SweepPoint<T> {
    fn from(pt: X) -> Self {
        SweepPoint(pt.into())
    }
}

impl<T: GeoNum> Deref for SweepPoint<T> {
    type Target = Coord<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 注意：为了更好的洁净性，我们目前保持它为不可变的。
// impl<T: GeoNum> DerefMut for SweepPoint<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_point_ordering() {
        let p1 = SweepPoint::from(Coord { x: 0., y: 0. });
        let p2 = SweepPoint::from(Coord { x: 1., y: 0. });
        let p3 = SweepPoint::from(Coord { x: 1., y: 1. });
        let p4 = SweepPoint::from(Coord { x: 1., y: 1. });

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
        assert!(p3 <= p4);
    }
}
