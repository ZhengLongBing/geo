use num_traits::FromPrimitive;

use crate::{CoordFloat, Length, Line, LineString, MultiLineString, Rhumb};

#[deprecated(
    since = "0.29.0",
    note = "请改用 `Length` 特征中的 `line.length::<Rhumb>()` 方法。"
)]
/// 确定几何体的长度，假设每段是一个[恒向线]。
///
/// [恒向线]: https://en.wikipedia.org/wiki/Rhumb_line
///
/// *注意*：此实现使用平均地球半径6371.088公里，基于[IUGG的建议](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait RhumbLength<T, RHS = Self> {
    /// 确定几何体的长度，假设每段是一个[恒向线]。
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
    /// ]);
    ///
    /// let length = linestring.rhumb_length();
    ///
    /// assert_eq!(
    ///     5_794_129., // 米
    ///     length.round()
    /// );
    /// ```
    ///
    /// [恒向线]: https://en.wikipedia.org/wiki/Rhumb_line
    fn rhumb_length(&self) -> T;
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        self.length::<Rhumb>()
    }
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        self.length::<Rhumb>()
    }
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        self.length::<Rhumb>()
    }
}
