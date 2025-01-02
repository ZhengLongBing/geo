use num_traits::FromPrimitive;

use crate::{CoordFloat, Line, LineString, MultiLineString};
use crate::{Haversine, Length};

#[deprecated(
    since = "0.29.0",
    note = "请通过 `Length` 特征使用 `line.length::<Haversine>()` 方法"
)]
/// 使用[半正矢公式]计算几何图形的长度。
///
/// [半正矢公式]: https://en.wikipedia.org/wiki/Haversine_formula
///
/// *注意*: 此实现使用基于[IUGG 的推荐值](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
/// 的平均地球半径 6371.0088 公里
pub trait HaversineLength<T, RHS = Self> {
    /// 使用[半正矢公式]计算几何图形的长度。
    ///
    /// # 单位
    ///
    /// - 返回值: 米
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::prelude::*;
    /// use geo::LineString;
    ///
    /// let linestring = LineString::<f64>::from(vec![
    ///     // 纽约市
    ///     (-74.006, 40.7128),
    ///     // 伦敦
    ///     (-0.1278, 51.5074),
    /// ]);
    ///
    /// let length = linestring.haversine_length();
    ///
    /// assert_eq!(
    ///     5_570_230., // 米
    ///     length.round()
    /// );
    /// ```
    ///
    /// [半正矢公式]: https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_length(&self) -> T;
}

#[allow(deprecated)]
impl<T> HaversineLength<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        self.length::<Haversine>()
    }
}

#[allow(deprecated)]
impl<T> HaversineLength<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        self.length::<Haversine>()
    }
}

#[allow(deprecated)]
impl<T> HaversineLength<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        self.length::<Haversine>()
    }
}
