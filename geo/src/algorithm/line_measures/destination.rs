use geo_types::{CoordFloat, Point};

/// 计算从起始点出发，给定航向和距离后的目的地坐标。
pub trait Destination<F: CoordFloat> {
    /// 返回一个新的点，该点是沿着给定航向从起始点移动指定距离后的目的地。
    ///
    /// 详见[具体实现](#implementors)。
    ///
    /// # 单位
    ///
    /// - `origin`: 起始点，x/y 单位取决于[特性实现](#implementors)。
    /// - `bearing`: 角度制，北：0°，东：90°，南：180°，西：270°。
    /// - `distance`: 单位取决于[特性实现](#implementors)。
    /// - 返回值: 目的地坐标，x/y 单位取决于[特性实现](#implementors)。
    ///
    /// [`metric_spaces`]: super::metric_spaces
    fn destination(origin: Point<F>, bearing: F, distance: F) -> Point<F>;
}
