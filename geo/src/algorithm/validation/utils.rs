use crate::{Coord, CoordFloat, GeoFloat, Intersects, LineString, RemoveRepeatedPoints};
use robust::{orient2d, Coord as RobustCoord};

pub(crate) fn check_coord_is_not_finite<T: CoordFloat>(geom: &Coord<T>) -> bool {
    // 检查坐标x和y是否有限
    if geom.x.is_finite() && geom.y.is_finite() {
        return false; // 如果都有限，返回false
    }
    true // 否则返回true
}

pub(crate) fn robust_check_points_are_collinear<T: CoordFloat>(
    p0: &Coord<T>,
    p1: &Coord<T>,
    p2: &Coord<T>,
) -> bool {
    // 使用鲁棒性算法检查三个点是否共线
    orient2d(
        RobustCoord {
            x: p0.x.to_f64().unwrap(),
            y: p0.y.to_f64().unwrap(),
        },
        RobustCoord {
            x: p1.x.to_f64().unwrap(),
            y: p1.y.to_f64().unwrap(),
        },
        RobustCoord {
            x: p2.x.to_f64().unwrap(),
            y: p2.y.to_f64().unwrap(),
        },
    ) == 0. // 如果结果为0，表示共线
}

pub(crate) fn check_too_few_points<T: CoordFloat>(geom: &LineString<T>, is_ring: bool) -> bool {
    // 根据是否为环判断有效的最少点数
    let n_pts = if is_ring { 4 } else { 2 };
    // 检查去除重复点后的点数是否小于有效点数
    if geom.remove_repeated_points().0.len() < n_pts {
        return true; // 如果小于，返回true
    }
    false // 否则返回false
}

pub(crate) fn linestring_has_self_intersection<F: GeoFloat>(geom: &LineString<F>) -> bool {
    // 需要更多测试以验证我们是否正确检测到“尖点”。
    // 也许我们可以使用 https://docs.rs/geo/latest/geo/algorithm/line_intersection/fn.line_intersection.html
    // 来计算交点，看看是否是单个点等。
    for (i, line) in geom.lines().enumerate() {
        for (j, other_line) in geom.lines().enumerate() {
            if i != j
                && line.intersects(&other_line)
                && line.start != other_line.end
                && line.end != other_line.start
            {
                return true; // 如果满足条件，表示有自相交，返回true
            }
        }
    }
    false // 如果没有检测到自相交，返回false
}
