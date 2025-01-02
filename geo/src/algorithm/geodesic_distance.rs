use crate::{Distance, Geodesic, Point};

#[deprecated(
    since = "0.29.0",
    note = "请使用 `Distance` 特征中的 `Geodesic::distance` 方法"
)]
/// 确定地球椭球模型上两个几何体之间的距离。
///
/// 这使用 [Karney (2013)] 提供的大地测量方法。与 Vincenty 等较旧的方法不同，此方法精确到几纳米并且总能收敛。
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicDistance<T, Rhs = Self> {
    /// 确定地球椭球模型上两个几何体之间的距离。
    ///
    /// 这使用 [Karney (2013)] 提供的大地测量方法。与 Vincenty 等较旧的方法不同，此方法精确到几纳米并且总能收敛。
    ///
    /// # 单位
    ///
    /// - 返回值：米
    ///
    /// # 示例
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::point;
    ///
    /// // 纽约
    /// let p1 = point!(x: -74.006, y: 40.7128);
    ///
    /// // 伦敦
    /// let p2 = point!(x: -0.1278, y: 51.5074);
    ///
    /// # #[allow(deprecated)]
    /// let distance = p1.geodesic_distance(&p2);
    ///
    /// assert_eq!(
    ///     5_585_234., // 米
    ///     distance.round()
    /// );
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_distance(&self, rhs: &Rhs) -> T;
}

#[allow(deprecated)]
impl GeodesicDistance<f64> for Point {
    fn geodesic_distance(&self, rhs: &Point) -> f64 {
        Geodesic::distance(*self, *rhs)
    }
}
