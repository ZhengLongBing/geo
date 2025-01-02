use crate::{CoordFloat, Distance, Point, Rhumb};
use num_traits::FromPrimitive;

#[deprecated(
    since = "0.29.0",
    note = "请使用 `Distance` 特征中的 `Rhumb::distance` 方法代替"
)]
/// 确定沿着[恒向线]的两个几何体之间的距离。
///
/// [恒向线]: https://en.wikipedia.org/wiki/Rhumb_line
///
/// *注意*: 此实现使用平均地球半径6371.088公里，基于[IUGG的建议](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait RhumbDistance<T, Rhs = Self> {
    /// 确定两个几何体之间沿着[恒向线]的距离。
    ///
    /// # 单位
    ///
    /// - 返回值：米
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::point;
    ///
    /// // 纽约市
    /// let p1 = point!(x: -74.006f64, y: 40.7128f64);
    ///
    /// // 伦敦
    /// let p2 = point!(x: -0.1278f64, y: 51.5074f64);
    ///
    /// # #[allow(deprecated)]
    /// let distance = p1.rhumb_distance(&p2);
    ///
    /// assert_eq!(
    ///     5_794_129., // 米
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [恒向线]: https://en.wikipedia.org/wiki/Rhumb_line
    fn rhumb_distance(&self, rhs: &Rhs) -> T;
}

#[allow(deprecated)]
impl<T> RhumbDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_distance(&self, rhs: &Point<T>) -> T {
        Rhumb::distance(*self, *rhs)
    }
}

#[cfg(test)]
mod test {
    use crate::Point;
    #[allow(deprecated)]
    use crate::RhumbDistance;

    #[test]
    fn distance1_test() {
        let a = Point::new(0., 0.);
        let b = Point::new(1., 0.);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 111195.0802335329_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 8903668.508603323_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test() {
        // 此输入来自问题 #100
        let a = Point::new(-77.036585, 38.897448);
        let b = Point::new(-77.009080, 38.889825);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 2526.7031699343006_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test_f32() {
        // 此输入来自问题 #100
        let a = Point::<f32>::new(-77.03658, 38.89745);
        let b = Point::<f32>::new(-77.00908, 38.889825);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 2526.7273_f32, epsilon = 1.0e-6);
    }
}
