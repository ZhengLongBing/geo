use geo_types::{CoordFloat, Point};

/// 计算两个点之间的方位角。
pub trait Bearing<F: CoordFloat> {
    /// 计算从`origin`到`destination`的方位角，单位为度。
    ///
    /// 详见[具体实现](#implementors)。
    ///
    /// # 单位
    /// - `origin`, `destination`: 点，其中x/y的单位取决于[特性实现](#implementors)。
    /// - 返回值: 角度，其中：北方: 0°，东方: 90°，南方: 180°，西方: 270°
    fn bearing(origin: Point<F>, destination: Point<F>) -> F;
}
