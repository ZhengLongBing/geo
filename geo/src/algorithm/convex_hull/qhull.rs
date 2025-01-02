use super::{swap_with_first_and_remove, trivial_hull};
use crate::kernels::{Kernel, Orientation};
use crate::utils::partition_slice;
use crate::{coord, Coord, GeoNum, LineString};

// 判断 `p_c` 是否位于线段 `p_a` 到  `p_b` 的正侧。
// 换句话说，是否存在线段 `p_a` 到 `p_c` 是对线段的逆时针旋转。
// 我们使用内核来确保这一谓词是精确的。
#[inline]
fn is_ccw<T>(p_a: Coord<T>, p_b: Coord<T>, p_c: Coord<T>) -> bool
where
    T: GeoNum,
{
    let o = T::Ker::orient2d(p_a, p_b, p_c);
    o == Orientation::CounterClockwise
}

// 改编自 https://web.archive.org/web/20180409175413/http://www.ahristov.com/tutorial/geometry-games/convex-hull.html
pub fn quick_hull<T>(mut points: &mut [Coord<T>]) -> LineString<T>
where
    T: GeoNum,
{
    // 如果点少于四个，则无法构建外壳
    if points.len() < 4 {
        return trivial_hull(points, false);
    }
    let mut hull = vec![];

    use crate::utils::least_and_greatest_index;
    let (min, max) = {
        let (min_idx, mut max_idx) = least_and_greatest_index(points);
        let min = swap_with_first_and_remove(&mut points, min_idx);

        // 需要考虑的两个特例：
        // (1) max_idx = 0，并且进行了交换
        if max_idx == 0 {
            max_idx = min_idx;
        }

        // (2) max_idx = min_idx: 那么可以选择任意点作为最大值。
        // 但在情况 (1) 中，它可能是 0，我们不应该减少它。
        max_idx = max_idx.saturating_sub(1);

        let max = swap_with_first_and_remove(&mut points, max_idx);
        (min, max)
    };

    {
        let (points, _) = partition_slice(points, |p| is_ccw(*max, *min, *p));
        hull_set(*max, *min, points, &mut hull);
    }
    hull.push(*max);
    let (points, _) = partition_slice(points, |p| is_ccw(*min, *max, *p));
    hull_set(*min, *max, points, &mut hull);
    hull.push(*min);
    // 闭合多边形
    let mut hull: LineString<_> = hull.into();
    hull.close();
    hull
}

/// 递归计算点子集的凸包
fn hull_set<T>(p_a: Coord<T>, p_b: Coord<T>, mut set: &mut [Coord<T>], hull: &mut Vec<Coord<T>>)
where
    T: GeoNum,
{
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return;
    }

    // 构造 `p_b` - `p_a` 的正交向量。
    // 我们计算其与 `v` - `p_a` 的内积，以找到离线段 a-b 最远的点。
    let p_orth = coord! {
        x: p_a.y - p_b.y,
        y: p_b.x - p_a.x,
    };

    let furthest_idx = set
        .iter()
        .map(|pt| {
            let p_diff = coord! {
                x: pt.x - p_a.x,
                y: pt.y - p_a.y,
            };
            p_orth.x * p_diff.x + p_orth.y * p_diff.y
        })
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;

    // 将 set 中的触及点移动到 hull
    let furthest_point = swap_with_first_and_remove(&mut set, furthest_idx);
    // 点位于 PB 上的情况
    {
        let (points, _) = partition_slice(set, |p| is_ccw(*furthest_point, p_b, *p));
        hull_set(*furthest_point, p_b, points, hull);
    }
    hull.push(*furthest_point);
    // 点位于 AP 上的情况
    let (points, _) = partition_slice(set, |p| is_ccw(p_a, *furthest_point, *p));
    hull_set(p_a, *furthest_point, points, hull);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IsConvex;

    #[test]
    fn quick_hull_test1() {
        let mut v = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 4.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: 0.0, y: 0.0 },
        ];
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            coord! { x: 0, y: 10 },
            coord! { x: 1, y: 1 },
            coord! { x: 10, y: 0 },
            coord! { x: 1, y: -1 },
            coord! { x: 0, y: -10 },
            coord! { x: -1, y: -1 },
            coord! { x: -10, y: 0 },
            coord! { x: -1, y: 1 },
            coord! { x: 0, y: 10 },
        ];
        let correct = vec![
            coord! { x: 0, y: -10 },
            coord! { x: 10, y: 0 },
            coord! { x: 0, y: 10 },
            coord! { x: -10, y: 0 },
            coord! { x: 0, y: -10 },
        ];
        let res = quick_hull(&mut v);
        assert_eq!(res.0, correct);
    }

    #[test]
    // 测试输出是否为逆时针
    fn quick_hull_test_ccw() {
        let initial = [
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let correct = [(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let v_correct: Vec<_> = correct.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res.0, v_correct);
    }

    #[test]
    fn quick_hull_test_ccw_maintain() {
        // 初始输入从最小的 y 值开始，是逆时针方向
        let initial = [
            (0., 0.),
            (2., 0.),
            (2.5, 1.75),
            (2.3, 1.7),
            (1.75, 2.5),
            (1.3, 2.),
            (0., 2.),
            (0., 0.),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test_complex() {
        let mut coords = geo_test_fixtures::poly1::<f64>().0;
        let correct = geo_test_fixtures::poly1_hull::<f64>().0;
        let res = quick_hull(&mut coords);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        let mut coords = geo_test_fixtures::poly2::<f64>().0;
        let correct = geo_test_fixtures::poly2_hull::<f64>().0;
        let res = quick_hull(&mut coords);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn quick_hull_test_collinear() {
        // 初始输入从最小的 x 开始，但不是最小的 y
        // 有三点具有相同的 x。
        // 输出不应包含中间点。
        let initial = [
            (-1., 0.),
            (-1., -1.),
            (-1., 1.),
            (0., 0.),
            (0., -1.),
            (0., 1.),
            (1., 0.),
            (1., -1.),
            (1., 1.),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }
}
