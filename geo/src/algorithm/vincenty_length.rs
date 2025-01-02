use num_traits::FromPrimitive;

use crate::vincenty_distance::{FailedToConvergeError, VincentyDistance};
use crate::{CoordFloat, Line, LineString, MultiLineString};

/// 使用 [Vincenty 公式] 计算几何图形的长度。
///
/// [Vincenty 公式]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
pub trait VincentyLength<T, RHS = Self> {
    /// 使用 [Vincenty 公式] 计算几何图形的长度。
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
    /// let linestring = LineString::<f64>::from(vec![
    ///     // 纽约市
    ///     (-74.006, 40.7128),
    ///     // 伦敦
    ///     (-0.1278, 51.5074),
    ///     // 大阪
    ///     (135.5244559, 34.687455)
    /// ]);
    ///
    /// let length = linestring.vincenty_length().unwrap();
    ///
    /// assert_eq!(
    ///     15_109_158., // 米
    ///     length.round()
    /// );
    /// ```
    ///
    /// [Vincenty 公式]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError>;
}

impl<T> VincentyLength<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    /// 返回值的单位是米。
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let (start, end) = self.points();
        start.vincenty_distance(&end)
    }
}

impl<T> VincentyLength<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let mut length = T::zero(); // 初始化长度为零
        for line in self.lines() {
            length = length + line.vincenty_length()?; // 逐线累加长度
        }
        Ok(length) // 返回累加的总长
    }
}

impl<T> VincentyLength<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let mut length = T::zero(); // 初始化长度为零
        for line_string in &self.0 {
            length = length + line_string.vincenty_length()?; // 累加所有线串的长度
        }
        Ok(length) // 返回累加的总长
    }
}
