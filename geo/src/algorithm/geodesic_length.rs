use crate::{Geodesic, Length, Line, LineString, MultiLineString};

#[deprecated(
    since = "0.29.0",
    note = "请使用`Length`特征中的`line.length::<Geodesic>()`方法。"
)]
/// 在地球椭球模型上确定几何体的长度。
///
/// 这使用了[Karney (2013)]提供的测地线测量方法。与Vincenty等较旧的方法相比，该方法精确到几纳米且总是收敛。
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicLength<T, RHS = Self> {
    /// 在地球椭球模型上确定几何体的长度。
    ///
    /// 这使用了[Karney (2013)]提供的测地线测量方法。与Vincenty等较旧的方法相比，该方法精确到几纳米且总是收敛。
    ///
    /// # 单位
    ///
    /// - 返回值：米
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::prelude::*;
    /// use geo::LineString;
    ///
    /// let linestring = LineString::from(vec![
    ///     // 纽约
    ///     (-74.006, 40.7128),
    ///     // 伦敦
    ///     (-0.1278, 51.5074),
    ///     // 大阪
    ///     (135.5244559, 34.687455)
    /// ]);
    ///
    /// let length = linestring.geodesic_length();
    ///
    /// assert_eq!(
    ///     15_109_158., // 米
    ///     length.round()
    /// );
    /// ```
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_length(&self) -> T;
}

#[allow(deprecated)]
impl GeodesicLength<f64> for Line {
    /// 返回值的单位是米。
    fn geodesic_length(&self) -> f64 {
        self.length::<Geodesic>()
    }
}

#[allow(deprecated)]
impl GeodesicLength<f64> for LineString {
    fn geodesic_length(&self) -> f64 {
        self.length::<Geodesic>()
    }
}

#[allow(deprecated)]
impl GeodesicLength<f64> for MultiLineString {
    fn geodesic_length(&self) -> f64 {
        self.length::<Geodesic>()
    }
}
