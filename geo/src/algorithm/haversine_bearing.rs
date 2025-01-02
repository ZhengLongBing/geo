use crate::{CoordFloat, Point};

#[deprecated(
    since = "0.29.0",
    note = "请改用 `Bearing` 特征中的 `Haversine::bearing` 方法"
)]
/// 返回到另一个点的方位角，单位为度。
///
/// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
/// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
pub trait HaversineBearing<T: CoordFloat> {
    /// 返回到另一个点的方位角，单位为度，其中北为0°，东为90°。
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// # #[allow(deprecated)]
    /// use geo::HaversineBearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.274410083250379, 48.84033282787534);
    /// # #[allow(deprecated)]
    /// let bearing = p_1.haversine_bearing(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    fn haversine_bearing(&self, point: Point<T>) -> T;
}

#[allow(deprecated)]
impl<T> HaversineBearing<T> for Point<T>
where
    T: CoordFloat,
{
    fn haversine_bearing(&self, point: Point<T>) -> T {
        let (lng_a, lat_a) = (self.x().to_radians(), self.y().to_radians()); // 当前点的经纬度转换为弧度
        let (lng_b, lat_b) = (point.x().to_radians(), point.y().to_radians()); // 目标点的经纬度转换为弧度
        let delta_lng = lng_b - lng_a; // 经度差异
        let s = lat_b.cos() * delta_lng.sin(); // 正弦计算
        let c = lat_a.cos() * lat_b.sin() - lat_a.sin() * lat_b.cos() * delta_lng.cos(); // 余弦计算

        T::atan2(s, c).to_degrees() // 返回方位角，转换为度
    }
}

#[cfg(test)]
mod test {
    use crate::point;
    #[allow(deprecated)]
    use crate::HaversineBearing;
    #[allow(deprecated)]
    use crate::HaversineDestination;

    #[test]
    fn north_bearing() {
        let p_1 = point!(x: 9., y: 47.);
        let p_2 = point!(x: 9., y: 48.);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 0.); // 北向方位角测试
    }

    #[test]
    fn equatorial_east_bearing() {
        let p_1 = point!(x: 9., y: 0.);
        let p_2 = point!(x: 10., y: 0.);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 90.); // 赤道东向方位角测试
    }

    #[test]
    fn east_bearing() {
        let p_1 = point!(x: 9., y: 10.);
        let p_2 = point!(x: 18.12961917258341, y: 9.875828894123304);

        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 90.); // 东向方位角测试
    }

    #[test]
    fn northeast_bearing() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = point!(x: 9.274409949623548, y: 48.84033274015048);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 45., epsilon = 1.0e-6); // 东北向方位角测试
    }

    #[test]
    fn consistent_with_destination() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        #[allow(deprecated)]
        let p_2 = p_1.haversine_destination(45., 10000.);

        #[allow(deprecated)]
        let b_1 = p_1.haversine_bearing(p_2);
        assert_relative_eq!(b_1, 45., epsilon = 1.0e-6); // 与目的地一致性的测试
    }
}
