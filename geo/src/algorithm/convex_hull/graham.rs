use super::{swap_with_first_and_remove, trivial_hull};
use crate::kernels::*;
use crate::{Coord, GeoNum, LineString};

/// [Graham's scan] 算法用于计算
/// 一组点的凸包。此算法的性能不如快速凸包，但允许
/// 计算凸包上的所有点，而不是不包括共线点的严格凸包。
///
/// # 参考资料
///
/// Graham, R.L.（1972）。["An Efficient Algorithm for
/// Determining the Convex Hull of a Finite Planar
/// Set"](http://www.math.ucsd.edu/~ronspubs/72_10_convex_hull.pdf)
/// (PDF). \
/// Information Processing Letters. 1 (4): 132–133.
/// [doi:10.1016/0020-0190(72)90045-2](https://doi.org/10.1016%2F0020-0190%2872%2990045-2)
///
/// [Graham's scan]: //en.wikipedia.org/wiki/Graham_scan
pub fn graham_hull<T>(mut points: &mut [Coord<T>], include_on_hull: bool) -> LineString<T>
where
    T: GeoNum,
{
    if points.len() < 4 {
        // 如果点数量少于四个，就无法构建。
        return trivial_hull(points, include_on_hull);
    }

    // 分配输出向量
    let mut output = Vec::with_capacity(points.len());

    // 找到字典序最小的点并添加到凸包中
    use crate::utils::least_index;
    use std::cmp::Ordering;
    let min_idx = least_index(points);
    let head = swap_with_first_and_remove(&mut points, min_idx);
    output.push(*head);

    // 按与头点的角度对其余点进行排序。如果两个点与头点共线，
    // 则按距离排序。在这里我们使用核谓词。
    let cmp = |q: &Coord<T>, r: &Coord<T>| match T::Ker::orient2d(*q, *head, *r) {
        Orientation::CounterClockwise => Ordering::Greater,
        Orientation::Clockwise => Ordering::Less,
        Orientation::Collinear => {
            let dist1 = T::Ker::square_euclidean_distance(*head, *q);
            let dist2 = T::Ker::square_euclidean_distance(*head, *r);
            dist1.partial_cmp(&dist2).unwrap()
        }
    };
    points.sort_unstable_by(cmp);

    for pt in points.iter() {
        while output.len() > 1 {
            let len = output.len();
            match T::Ker::orient2d(output[len - 2], output[len - 1], *pt) {
                Orientation::CounterClockwise => {
                    break;
                }
                Orientation::Clockwise => {
                    output.pop();
                }
                Orientation::Collinear => {
                    if include_on_hull {
                        break;
                    } else {
                        output.pop();
                    }
                }
            }
        }
        // 角落情况：如果之前在这个循环前添加的字典序最小点被重复，
        // 那么我们不应该在这里再次添加它（因为在第一次迭代中 output.len() == 1）
        if include_on_hull || pt != output.last().unwrap() {
            output.push(*pt);
        }
    }

    // 关闭并输出线串
    let mut output = LineString::new(output);
    output.close();
    output
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IsConvex;
    fn test_convexity<T: GeoNum>(mut initial: Vec<Coord<T>>) {
        let hull = graham_hull(&mut initial, false);
        assert!(hull.is_strictly_ccw_convex());
        let hull = graham_hull(&mut initial, true);
        assert!(hull.is_ccw_convex());
    }

    #[test]
    fn test_graham_hull_ccw() {
        let initial = [
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let initial = initial.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_hull_test1() {
        let v: Vec<_> = vec![(0, 0), (4, 0), (4, 1), (1, 1), (1, 4), (0, 4), (0, 0)];
        let initial = v.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_hull_test2() {
        let v = [
            (0, 10),
            (1, 1),
            (10, 0),
            (1, -1),
            (0, -10),
            (-1, -1),
            (-10, 0),
            (-1, 1),
            (0, 10),
        ];
        let initial = v.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_test_complex() {
        test_convexity(geo_test_fixtures::poly1::<f64>().0);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        test_convexity(geo_test_fixtures::poly2::<f64>().0);
    }
}
