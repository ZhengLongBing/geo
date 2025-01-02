use super::Distance;
use crate::{CoordFloat, Line, LineString, MultiLineString, Point};

/// 计算给定[度量空间](crate::algorithm::line_measures::metric_spaces)中的`Line`、`LineString`或`MultiLineString`的长度。
///
/// # 示例
/// ```
/// use geo::algorithm::line_measures::{Length, Euclidean, Haversine};
///
/// let line_string = geo::wkt!(LINESTRING(
///     0.0 0.0,
///     3.0 4.0,
///     3.0 5.0
/// ));
/// assert_eq!(line_string.length::<Euclidean>(), 6.0);
///
/// let line_string_lon_lat = geo::wkt!(LINESTRING (
///     -47.9292 -15.7801f64,
///     -58.4173 -34.6118,
///     -70.6483 -33.4489
/// ));
/// assert_eq!(line_string_lon_lat.length::<Haversine>().round(), 3_474_956.0);
/// ```
pub trait Length<F: CoordFloat> {
    fn length<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self) -> F;
}

impl<F: CoordFloat> Length<F> for Line<F> {
    fn length<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self) -> F {
        MetricSpace::distance(self.start_point(), self.end_point())
    }
}

impl<F: CoordFloat> Length<F> for LineString<F> {
    fn length<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self) -> F {
        let mut length = F::zero();
        for line in self.lines() {
            length = length + line.length::<MetricSpace>();
        }
        length
    }
}

impl<F: CoordFloat> Length<F> for MultiLineString<F> {
    fn length<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self) -> F {
        let mut length = F::zero();
        for line in self {
            length = length + line.length::<MetricSpace>();
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, Euclidean, Geodesic, Haversine, Rhumb};

    #[test]
    fn lines() {
        // 从伦敦到巴黎
        let line = Line::new(
            coord!(x: -0.1278f64, y: 51.5074),
            coord!(x: 2.3522, y: 48.8566),
        );

        assert_eq!(
            343_923., // 米
            line.length::<Geodesic>().round()
        );
        assert_eq!(
            341_088., // 米
            line.length::<Rhumb>().round()
        );
        assert_eq!(
            343_557., // 米
            line.length::<Haversine>().round()
        );

        // 计算未投影（经度/纬度）线的欧氏长度会得出无意义的答案
        assert_eq!(
            4., // 无意义！
            line.length::<Euclidean>().round()
        );
        // EPSG:3035 下的伦敦到巴黎
        let projected_line = Line::new(
            coord!(x: 3620451.74f64, y: 3203901.44),
            coord!(x: 3760771.86, y: 2889484.80),
        );
        assert_eq!(344_307., projected_line.length::<Euclidean>().round());
    }

    #[test]
    fn line_strings() {
        let line_string = LineString::new(vec![
            coord!(x: -58.3816f64, y: -34.6037), // 布宜诺斯艾利斯，阿根廷
            coord!(x: -77.0428, y: -12.0464),    // 利马，秘鲁
            coord!(x: -47.9292, y: -15.7801),    // 巴西利亚，巴西
        ]);

        assert_eq!(
            6_302_220., // 米
            line_string.length::<Geodesic>().round()
        );
        assert_eq!(
            6_332_790., // 米
            line_string.length::<Rhumb>().round()
        );
        assert_eq!(
            6_304_387., // 米
            line_string.length::<Haversine>().round()
        );

        // 计算未投影（经度/纬度）的欧氏长度会得出无意义的答案
        assert_eq!(
            59., // 无意义！
            line_string.length::<Euclidean>().round()
        );
        // EPSG:102033
        let projected_line_string = LineString::from(vec![
            coord!(x: 143042.46f64, y: -1932485.45), // 布宜诺斯艾利斯，阿根廷
            coord!(x: -1797084.08, y: 583528.84),    // 利马，秘鲁
            coord!(x: 1240052.27, y: 207169.12),     // 巴西利亚，巴西
        ]);
        assert_eq!(
            6_237_538.,
            projected_line_string.length::<Euclidean>().round()
        );
    }
}
