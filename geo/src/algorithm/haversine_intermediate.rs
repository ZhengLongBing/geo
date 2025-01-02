use crate::{CoordFloat, Haversine, InterpolatePoint, Point};
use num_traits::FromPrimitive;

#[deprecated(since = "0.29.0", note = "请使用 `InterpolatePoint` 特征代替")]
/// 返回在两个已知点之间的大圆路径上的一个新点
pub trait HaversineIntermediate<T: CoordFloat> {
    #[deprecated(
        since = "0.29.0",
        note = "请使用 `InterpolatePoint` 特征中的 `Haversine::point_at_ratio_between` 方法代替"
    )]
    /// 返回在`self`和`other`之间的大圆路径上的一个新`Point`。
    ///
    /// * `other` - 要插值到的另一点。
    /// * `ratio` - 新点应该在路线上距离的比例，以0.0表示在`self`上，
    ///             1.0表示在`other`上。
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// # #[allow(deprecated)]
    /// use geo::HaversineIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// # #[allow(deprecated)]
    /// let i20 = p1.haversine_intermediate(&p2, 0.2);
    /// assert_relative_eq!(i20, Point::new(29.8, 29.9), epsilon = 0.2);
    ///
    /// # #[allow(deprecated)]
    /// let i80 = p1.haversine_intermediate(&p2, 0.8);
    /// assert_relative_eq!(i80, Point::new(103.5, 33.5), epsilon = 0.2);
    /// ```
    fn haversine_intermediate(&self, other: &Point<T>, ratio: T) -> Point<T>;

    #[deprecated(
        since = "0.29.0",
        note = "请使用 `InterpolatePoint` 特征中的 `Haversine::points_along_line` 方法代替"
    )]
    /// 插值`Point`沿着`self`和`other`之间的大圆路径。
    ///
    /// 将添加尽可能多的点，以确保点之间的距离
    /// 永远不会超过`max_dist`。
    ///
    /// `include_ends`: 应该在输出中包括起点和终点吗？
    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

#[allow(deprecated)]
impl<T> HaversineIntermediate<T> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_intermediate(&self, other: &Point<T>, ratio: T) -> Point<T> {
        Haversine::point_at_ratio_between(*self, *other, ratio)
    }

    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>> {
        Haversine::points_along_line(*self, *other, max_dist, include_ends).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(deprecated)]
    use crate::HaversineIntermediate;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::new(10.0, 20.0); // 定义第一个点
        let p2 = Point::new(15.0, 25.0); // 定义第二个点
        #[allow(deprecated)]
        let i0 = p1.haversine_intermediate(&p2, 0.0); // 在路径比例为0时插值
        #[allow(deprecated)]
        let i100 = p1.haversine_intermediate(&p2, 1.0); // 在路径比例为1时插值
        assert_relative_eq!(i0.x(), p1.x(), epsilon = 1.0e-6); // 验证与第一个点 x 坐标接近
        assert_relative_eq!(i0.y(), p1.y(), epsilon = 1.0e-6); // 验证与第一个点 y 坐标接近
        assert_relative_eq!(i100.x(), p2.x(), epsilon = 1.0e-6); // 验证与第二个点 x 坐标接近
        assert_relative_eq!(i100.y(), p2.y(), epsilon = 1.0e-6); // 验证与第二个点 y 坐标接近
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::new(10.0, 20.0); // 定义起始点
        let p2 = Point::new(125.0, 25.0); // 定义终止点
        #[allow(deprecated)]
        let i20 = p1.haversine_intermediate(&p2, 0.2); // 在路径比例为0.2时插值
        #[allow(deprecated)]
        let i50 = p1.haversine_intermediate(&p2, 0.5); // 在路径比例为0.5时插值
        #[allow(deprecated)]
        let i80 = p1.haversine_intermediate(&p2, 0.8); // 在路径比例为0.8时插值
        let i20_should = Point::new(29.83519, 29.94841); // 预期点位置
        let i50_should = Point::new(65.87471, 37.72201); // 预期点位置
        let i80_should = Point::new(103.56036, 33.50518); // 预期点位置
        assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2); // 验证点 x 坐标接近
        assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2); // 验证点 y 坐标接近
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2); // 验证点 x 坐标接近
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2); // 验证点 y 坐标接近
        assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2); // 验证点 x 坐标接近
        assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2); // 验证点 y 坐标接近
    }

    #[test]
    fn should_be_north_pole_test() {
        let p1 = Point::new(0.0, 10.0); // 定义起始点
        let p2 = Point::new(180.0, 10.0); // 定义终止点
        #[allow(deprecated)]
        let i50 = p1.haversine_intermediate(&p2, 0.5); // 在路径比例为0.5时插值
        let i50_should = Point::new(90.0, 90.0); // 预期北极点位置
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 1.0e-6); // 验证点 x 坐标接近
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 1.0e-6); // 验证点 y 坐标接近
    }

    #[test]
    fn should_be_start_end_test() {
        let p1 = Point::new(30.0, 40.0); // 起始点
        let p2 = Point::new(40.0, 50.0); // 终止点
        let max_dist = 1500000.0; // 最大距离，单位为米
        #[allow(deprecated)]
        let route = p1.haversine_intermediate_fill(&p2, max_dist, true); // 插值
        assert_eq!(route, vec![p1, p2]); // 验证生成路线
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::new(30.0, 40.0); // 起始点
        let p2 = Point::new(40.0, 50.0); // 终止点
        let max_dist = 1000000.0; // 最大距离，单位为米
        #[allow(deprecated)]
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5); // 在路径比例为0.5时插值
        #[allow(deprecated)]
        let fill = p1.haversine_intermediate_fill(&p2, max_dist, true); // 插值并包括端点
        assert_eq!(fill, vec![p1, i50, p2]); // 验证生成的点
        #[allow(deprecated)]
        let fill = p1.haversine_intermediate_fill(&p2, max_dist, false); // 插值但不包括端点
        assert_eq!(fill, vec![i50]); // 验证生成的点
    }

    #[test]
    fn should_add_i25_i50_i75_test() {
        let p1 = Point::new(30.0, 40.0); // 起始点
        let p2 = Point::new(40.0, 50.0); // 终止点
        let max_dist = 400000.0; // 最大距离，单位为米
        #[allow(deprecated)]
        let i25 = p1.clone().haversine_intermediate(&p2, 0.25); // 在路径比例为0.25时插值
        #[allow(deprecated)]
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5); // 在路径比例为0.5时插值
        #[allow(deprecated)]
        let i75 = p1.clone().haversine_intermediate(&p2, 0.75); // 在路径比例为0.75时插值
        #[allow(deprecated)]
        let route = p1.haversine_intermediate_fill(&p2, max_dist, true); // 插值并包括端点
        assert_eq!(route, vec![p1, i25, i50, i75, p2]); // 验证生成的点顺序和数量
    }
}
