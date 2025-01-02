use crate::algorithm::{Densify, Length, LineInterpolatePoint, LinesIter};
use crate::geometry::{Coord, LineString, MultiLineString};
use crate::line_measures::{Euclidean, Haversine};

/// 将一个线串(LineString)分割成`segment_count`个等长的线串组成的多线串(MultiLineString)，
/// 使用欧几里得距离计算。 如果处理地理坐标(纬度/经度)，请参见`LineStringSegmentizeHaversine`。
///
/// 当`segment_count`等于0或无法在`Line`段上插入点时，将返回`None`。
///
/// # 例子
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentize};
/// // 创建一个简单的线串
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // 把它分割成6个线串，放入一个多线串中
/// let segmentized = lns.line_segmentize(6).unwrap();
/// // 比较元素的数量
/// assert_eq!(6, segmentized.0.len());
///```
pub trait LineStringSegmentize {
    fn line_segmentize(&self, segment_count: usize) -> Option<MultiLineString>;
}

/// 将一个线串(LineString)分割成`segment_count`个等长的线串组成的多线串(MultiLineString)，
/// 使用 Haversine 距离计算。使用于地理坐标系的数据时优先使用此方法而非`LineStringSegmentize`。
///
/// 当`segment_count`等于0或无法在`Line`段上插入点时，将返回`None`。
///
/// # 例子
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentizeHaversine};
/// // 创建一个简单的线串
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // 把它分割成6个线串，放入一个多线串中
/// let segmentized = lns.line_segmentize_haversine(6).unwrap();
/// // 比较元素的数量
/// assert_eq!(6, segmentized.0.len());
///```
pub trait LineStringSegmentizeHaversine {
    fn line_segmentize_haversine(&self, segment_count: usize) -> Option<MultiLineString>;
}

macro_rules! implement_segmentize {
    ($trait_name:ident, $method_name:ident, $metric_space:ty) => {
        impl $trait_name for LineString {
            fn $method_name(&self, n: usize) -> Option<MultiLineString> {
                if (n == usize::MIN) || (n == usize::MAX) {
                    return None;
                } else if n == 1 {
                    let mlns = MultiLineString::from(self.clone());
                    return Some(mlns);
                }

                let mut res_coords: Vec<Vec<Coord>> = Vec::with_capacity(n);
                let total_length = self.length::<$metric_space>();
                let mut cum_length = 0_f64;
                let segment_prop = (1_f64) / (n as f64);
                let segment_length = total_length * segment_prop;
                let densified = self.densify::<$metric_space>(segment_length - f64::EPSILON);

                if densified.lines().count() == n {
                    let linestrings = densified
                        .lines()
                        .map(LineString::from)
                        .collect::<Vec<LineString>>();
                    return Some(MultiLineString::new(linestrings));
                }

                let n_lines = densified.lines().count();
                let lns = densified.lines_iter();
                let mut ln_vec: Vec<Coord> = Vec::new();

                for (i, segment) in lns.enumerate() {
                    if i == 0 {
                        ln_vec.push(segment.start)
                    }

                    let length = segment.length::<$metric_space>();
                    cum_length += length;

                    if (cum_length >= segment_length) && (i != (n_lines - 1)) {
                        let remainder = cum_length - segment_length;
                        let endpoint =
                            segment.line_interpolate_point((length - remainder) / length)?;

                        ln_vec.push(endpoint.into());
                        let to_push = ln_vec.drain(..);
                        res_coords.push(to_push.collect::<Vec<Coord>>());

                        if i != n_lines {
                            ln_vec.push(endpoint.into());
                        }
                        cum_length = remainder;
                    }
                    ln_vec.push(segment.end);
                }

                res_coords.push(ln_vec);
                let res_lines = res_coords
                    .into_iter()
                    .map(LineString::new)
                    .collect::<Vec<LineString>>();
                Some(MultiLineString::new(res_lines))
            }
        }
    };
}

// 为平面距离(Euclidean)实现线段化(LineStringSegmentize)特性
implement_segmentize!(LineStringSegmentize, line_segmentize, Euclidean);
// 为大圆距离(Haversine)实现线段化(LineStringSegmentizeHaversine)特性
implement_segmentize!(
    LineStringSegmentizeHaversine,
    line_segmentize_haversine,
    Haversine
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::LineString;
    use approx::RelativeEq;

    #[test]
    fn n_elems_bug() {
        // 测试一个似乎失败的边缘案例：
        // https://github.com/georust/geo/issues/1075
        // https://github.com/JosiahParry/rsgeo/issues/28

        let linestring: LineString = vec![
            [324957.69921197, 673670.123131518],
            [324957.873557727, 673680.139281405],
            [324959.863123514, 673686.784106964],
            [324961.852683597, 673693.428933452],
            [324963.822867622, 673698.960855279],
            [324969.636546456, 673709.992098018],
            [324976.718443977, 673722.114520549],
            [324996.443964294, 673742.922904206],
        ]
        .into();
        let segments = linestring.line_segmentize(2).unwrap();
        assert_eq!(segments.0.len(), 2);
        let segments = linestring.line_segmentize(3).unwrap();
        assert_eq!(segments.0.len(), 3);
        let segments = linestring.line_segmentize(4).unwrap();
        assert_eq!(segments.0.len(), 4);

        assert_eq!(
            segments.length::<Euclidean>(),
            linestring.length::<Euclidean>()
        );
    }

    #[test]
    fn long_end_segment() {
        let linestring: LineString = vec![
            [325581.792390628, 674398.495901267],
            [325585.576868499, 674400.657039341],
            [325589.966469742, 674401.694493658],
            [325593.750940609, 674403.855638851],
            [325599.389217394, 674404.871546368],
            [325604.422360924, 674407.011146146],
            [325665.309662534, 674424.885671739],
        ]
        .into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);
        assert_relative_eq!(
            linestring.length::<Euclidean>(),
            segments.length::<Euclidean>(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn two_coords() {
        let linestring: LineString = vec![[0.0, 0.0], [0.0, 1.0]].into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);
        assert_relative_eq!(
            linestring.length::<Euclidean>(),
            segments.length::<Euclidean>(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn long_middle_segments() {
        let linestring: LineString = vec![
            [325403.816883668, 673966.295402012],
            [325410.280933752, 673942.805501254],
            [325410.280933752, 673942.805501254],
            [325439.782082601, 673951.201057316],
            [325439.782082601, 673951.201057316],
            [325446.064640793, 673953.318876004],
            [325446.064640793, 673953.318876004],
            [325466.14184472, 673958.537886844],
            [325466.14184472, 673958.537886844],
            [325471.799973648, 673960.666539074],
            [325471.799973648, 673960.666539074],
            [325518.255916084, 673974.335722824],
            [325518.255916084, 673974.335722824],
            [325517.669972133, 673976.572326305],
            [325517.669972133, 673976.572326305],
            [325517.084028835, 673978.808929878],
            [325517.084028835, 673978.808929878],
            [325515.306972763, 673984.405833764],
            [325515.306972763, 673984.405833764],
            [325513.549152184, 673991.115645844],
            [325513.549152184, 673991.115645844],
            [325511.772106396, 673996.712551354],
        ]
        .into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);

        assert_relative_eq!(
            linestring.length::<Euclidean>(),
            segments.length::<Euclidean>(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    // 测试 n 为 0 返回 None，usize::MAX 返回 None
    fn n_is_zero() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(0);
        assert!(segments.is_none())
    }

    #[test]
    fn n_is_max() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(usize::MAX);
        assert!(segments.is_none())
    }

    #[test]
    fn n_greater_than_lines() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(5).unwrap();

        // 确认有 n 个线段
        assert_eq!(segments.0.len(), 5);

        // 确认线段是等长的
        let lens = segments
            .into_iter()
            .map(|x| x.length::<Euclidean>())
            .collect::<Vec<f64>>();

        let first = lens[0];

        assert!(lens
            .iter()
            .all(|x| first.relative_eq(x, f64::EPSILON, 1e-10)))
    }

    #[test]
    // 测试累计长度是相同的
    fn cumul_length() {
        let linestring: LineString = vec![[0.0, 0.0], [1.0, 1.0], [1.0, 2.0], [3.0, 3.0]].into();
        let segments = linestring.line_segmentize(2).unwrap();

        assert_relative_eq!(
            linestring.length::<Euclidean>(),
            segments.length::<Euclidean>(),
            epsilon = f64::EPSILON
        )
    }

    #[test]
    fn n_elems() {
        let linestring: LineString = vec![[0.0, 0.0], [1.0, 1.0], [1.0, 2.0], [3.0, 3.0]].into();
        let segments = linestring.line_segmentize(2).unwrap();
        assert_eq!(segments.0.len(), 2)
    }

    #[test]
    fn tiny_distances() {
        // 这个测试是为了确保在超小距离下，单位的数量仍然是指定的。
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        let n = 8;
        let segments = linestring.line_segmentize(n).unwrap();
        assert_eq!(segments.0.len(), n)
    }

    #[test]
    fn haversine_n_elems() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        let n = 8;

        let segments = linestring.line_segmentize_haversine(n).unwrap();
        assert_eq!(n, segments.0.len());
    }

    #[test]
    fn haversine_segment_length() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        let n = 8;

        let segments = linestring.line_segmentize_haversine(n).unwrap();
        let lens = segments
            .0
            .iter()
            .map(|li| li.length::<Haversine>())
            .collect::<Vec<_>>();

        let epsilon = 1e-6; // 小数点后第6位，相当于微米
        assert!(lens.iter().all(|&x| (x - lens[0]).abs() < epsilon));
    }

    #[test]
    fn haversine_total_length() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        assert_relative_eq!(linestring.length::<Haversine>(), 83.3523000093029);

        let n = 8;

        let segments = linestring.line_segmentize_haversine(n).unwrap();

        // 在第12位小数处不同，相当于皮米
        assert_relative_eq!(
            linestring.length::<Haversine>(),
            segments.length::<Haversine>(),
            epsilon = 1e-11
        );
    }
}
