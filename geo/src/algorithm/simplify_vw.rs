use crate::prelude::*;
use crate::{
    Coord, CoordFloat, GeoFloat, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    Triangle,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use rstar::primitives::CachedEnvelope;
use rstar::{RTree, RTreeNum};

/// 存储三角形的信息。面积用于优先队列中的排序以及确定移除
#[derive(Debug)]
struct VScore<T>
where
    T: CoordFloat,
{
    left: usize,
    /// 原始[LineString]中当前[Point]的索引：待移除的候选点
    current: usize,
    right: usize,
    area: T,
    // `visvalingam_preserve`使用`intersector`，而`visvalingam`不使用，所以它始终为假
    intersector: bool,
}

// 这些impls为我们提供了一个最小堆
impl<T> Ord for VScore<T>
where
    T: CoordFloat,
{
    fn cmp(&self, other: &VScore<T>) -> Ordering {
        other.area.partial_cmp(&self.area).unwrap()
    }
}

impl<T> PartialOrd for VScore<T>
where
    T: CoordFloat,
{
    fn partial_cmp(&self, other: &VScore<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for VScore<T> where T: CoordFloat {}

impl<T> PartialEq for VScore<T>
where
    T: CoordFloat,
{
    fn eq(&self, other: &VScore<T>) -> bool
    where
        T: CoordFloat,
    {
        self.area == other.area
    }
}

/// 使用[Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263)算法简化线
//
// 此方法返回简化线的**索引**
// epsilon是最小三角形面积
// 论文指出：
// 如果[新三角形]的计算面积小于最后一个被消除点的面积，使用后者的面积。
// （这样可以确保当前点不能在不消除之前消除的点的情况下被消除）
// （Visvalingam和Whyatt 2013，第47页）
// 但是，如果你使用用户定义的epsilon，则不适用；
// 可以删除面积低于epsilon的三角形，
// 然后重新计算新三角形面积并将其推入堆
// 基于胡戈·威尔逊的原始实现:
// https://github.com/huonw/isrustfastyet/blob/25e7a68ff26673a8556b170d3c9af52e1c818288/mem/line_simplify.rs
fn visvalingam_indices<T>(orig: &LineString<T>, epsilon: &T) -> Vec<usize>
where
    T: CoordFloat,
{
    // 没有至少三个点则无需继续
    if orig.0.len() < 3 {
        return orig.0.iter().enumerate().map(|(idx, _)| idx).collect();
    }

    let max = orig.0.len();

    // 相邻的保留点。用`orig`中的索引模拟链表中的点。大数（大于等于`max`）表示没有下一个元素，（0，0）表示已删除的元素。
    let mut adjacent: Vec<_> = (0..orig.0.len())
        .map(|i| {
            if i == 0 {
                (-1_i32, 1_i32)
            } else {
                ((i - 1) as i32, (i + 1) as i32)
            }
        })
        .collect();

    // 基于三角形的面积，将所有三角形存储在最小优先队列中。
    //
    // 如果/当点被移除时，*不*删除无效的三角形；它们通过在主循环中检查相邻对应的（0，0）值来实现跳过。

    // 计算初始三角形
    let mut pq = orig
        .triangles()
        .enumerate()
        .map(|(i, triangle)| VScore {
            area: triangle.unsigned_area(),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: false,
        })
        .collect::<BinaryHeap<VScore<T>>>();
    // 当仍有其相关三角形的面积低于epsilon的点时
    while let Some(smallest) = pq.pop() {
        if smallest.area > *epsilon {
            // 无需继续尝试：最小堆确保我们按顺序处理三角形
            // 所以，如果我们看到一个超出容差的，我们就完成了：其他都太大
            break;
        }
        // 该三角形的面积低于epsilon：相关点是待移除的候选
        let (left, right) = adjacent[smallest.current];
        // 自从创建此VScore后，此三角形中的一个点已被删除，因此跳过
        if left != smallest.left as i32 || right != smallest.right as i32 {
            continue;
        }
        // 我们有一个有效的三角形，其面积小于epsilon，因此从模拟的“链表”中移除它
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        adjacent[smallest.current] = (0, 0);

        // 使用左和右相邻点重新计算相邻的三角形，这可能会将新三角形添加到堆中
        recompute_triangles(&smallest, orig, &mut pq, ll, left, right, rr, max, epsilon);
    }
    // 过滤掉已删除的点，返回剩下的点索引
    orig.0
        .iter()
        .enumerate()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(tup.0) } else { None })
        .collect::<Vec<usize>>()
}

/// 使用左右相邻点重新计算相邻的三角形，并推入堆中
///
/// 这用于标准和拓扑保护变体。
#[allow(clippy::too_many_arguments)]
fn recompute_triangles<T>(
    smallest: &VScore<T>,
    orig: &LineString<T>,
    pq: &mut BinaryHeap<VScore<T>>,
    ll: i32,
    left: i32,
    right: i32,
    rr: i32,
    max: usize,
    epsilon: &T,
) where
    T: CoordFloat,
{
    let choices = [(ll, left, right), (left, right, rr)];
    for &(ai, current_point, bi) in &choices {
        if ai as usize >= max || bi as usize >= max {
            // 越界，也就是说，我们在一边
            continue;
        }
        let area = Triangle::new(
            orig.0[ai as usize],
            orig.0[current_point as usize],
            orig.0[bi as usize],
        )
        .unsigned_area();

        // 该逻辑仅适用于VW-Preserve
        // smallest.current的移除导致自交，并且此点在它之前，我们通过将其面积降低到负epsilon来确保它接下来被移除
        // 我们检查current_point是否小于smallest.current，因为
        // 如果它更大，问题中的点在smallest.current之后：我们只想移除
        // 在smallest.current之前的点
        let area = if smallest.intersector && (current_point as usize) < smallest.current {
            -*epsilon
        } else {
            area
        };

        let v = VScore {
            area,
            current: current_point as usize,
            left: ai as usize,
            right: bi as usize,
            intersector: false,
        };
        pq.push(v)
    }
}

// visvalingam_indices的包装器，将索引映射回点
fn visvalingam<T>(orig: &LineString<T>, epsilon: &T) -> Vec<Coord<T>>
where
    T: CoordFloat,
{
    // epsilon必须大于零才能进行有意义的简化
    if *epsilon <= T::zero() {
        return orig.0.to_vec();
    }
    let subset = visvalingam_indices(orig, epsilon);
    // 使用索引过滤orig
    // 在这里使用get更稳健，但在这种情况下输入子集保证是有效的
    orig.0
        .iter()
        .zip(subset.iter())
        .map(|(_, s)| orig[*s])
        .collect()
}

// 包装实际的VW函数，以便R*树可以共享。
// 这确保外壳和环可以访问所有段，从而可以检测外部和内部环之间的交集。
//
// 常量:
//
// * `INITIAL_MIN`
//   * 如果数量低于这个值，立即停止
// * `MIN_POINTS`
//   * 如果在点移除之前检测到自相交，并且只剩下`MIN_POINTS`，则停止：因为自相交会导致移除空间上之前的点，可能导致进一步的自相交，而没有移除更多点的可能性，潜在地使几何无效。
fn vwp_wrapper<T, const INITIAL_MIN: usize, const MIN_POINTS: usize>(
    exterior: &LineString<T>,
    interiors: Option<&[LineString<T>]>,
    epsilon: &T,
) -> Vec<Vec<Coord<T>>>
where
    T: GeoFloat + RTreeNum,
{
    let mut rings = vec![];
    // 用外部和内部样本填充R*树（如果有）
    let mut tree: RTree<CachedEnvelope<_>> = RTree::bulk_load(
        exterior
            .lines()
            .chain(
                interiors
                    .iter()
                    .flat_map(|ring| *ring)
                    .flat_map(|line_string| line_string.lines()),
            )
            .map(CachedEnvelope::new)
            .collect::<Vec<_>>(),
    );

    // 简化外壳
    rings.push(visvalingam_preserve::<T, INITIAL_MIN, MIN_POINTS>(
        exterior, epsilon, &mut tree,
    ));
    // 如果有的话，简化内部环
    if let Some(interior_rings) = interiors {
        for ring in interior_rings {
            rings.push(visvalingam_preserve::<T, INITIAL_MIN, MIN_POINTS>(
                ring, epsilon, &mut tree,
            ))
        }
    }
    rings
}

/// Visvalingam-Whyatt自交检测以保留拓扑
/// 这是一个基于https://www.jasondavies.com/simplify/的技术移植
//
// 常量:
//
// * `INITIAL_MIN`
//   * 如果数量低于这个值，立即停止
// * `MIN_POINTS`
//   * 如果在点移除之前检测到自相交，并且只剩下`MIN_POINTS`，则停止：因为自相交会导致移除空间上之前的点，可能导致进一步的自相交，而没有移除更多点的可能性，潜在地使几何无效。
fn visvalingam_preserve<T, const INITIAL_MIN: usize, const MIN_POINTS: usize>(
    orig: &LineString<T>,
    epsilon: &T,
    tree: &mut RTree<CachedEnvelope<Line<T>>>,
) -> Vec<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    if orig.0.len() < 3 || *epsilon <= T::zero() {
        return orig.0.to_vec();
    }
    let max = orig.0.len();
    let mut counter = orig.0.len();

    // 相邻的保留点。用`orig`中的索引模拟链表中的点。大数（大于等于`max`）表示没有下一个元素，（0，0）表示已删除的元素。
    let mut adjacent: Vec<_> = (0..orig.0.len())
        .map(|i| {
            if i == 0 {
                (-1_i32, 1_i32)
            } else {
                ((i - 1) as i32, (i + 1) as i32)
            }
        })
        .collect();
    // 基于三角形的面积，将所有三角形存储在最小优先队列中。
    //
    // 如果/当点被移除时，*不*删除无效的三角形；它们通过在主循环中检查相邻对应的（0，0）值来实现跳过。

    // 计算初始三角形
    let mut pq = orig
        .triangles()
        .enumerate()
        .map(|(i, triangle)| VScore {
            area: triangle.unsigned_area(),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: false,
        })
        .collect::<BinaryHeap<VScore<T>>>();

    // 当仍有其相关三角形的面积低于epsilon的点时
    while let Some(mut smallest) = pq.pop() {
        if smallest.area > *epsilon {
            // 无需继续：我们已经看到了所有的候选三角形；
            // 最小堆保证了这一点
            break;
        }
        if counter <= INITIAL_MIN {
            // 无论如何，我们不能再移除任何点了
            break;
        }
        let (left, right) = adjacent[smallest.current];
        // 自从创建此VScore后，此三角形中的一个点已被删除，因此跳过
        if left != smallest.left as i32 || right != smallest.right as i32 {
            continue;
        }
        // 如果移除此点导致自交，则我们也移除前一个点
        // 移除会改变几何，消除自交
        // 然而，如果我们距离绝对最小值1点之遥，我们不能移除此点或下一个点
        // 因为如果移除下一个也导致交集，我们就无法形成有效的几何。
        // 因此简化过程结束。
        smallest.intersector = tree_intersect(tree, &smallest, &orig.0);
        if smallest.intersector && counter <= MIN_POINTS {
            break;
        }
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        // 我们有一个有效的三角形，其面积小于容差，因此从模拟的“链表”中移除它
        adjacent[smallest.current] = (0, 0);
        counter -= 1;
        // 从R*树中删除陈旧的段
        let left_point = Point::from(orig.0[left as usize]);
        let middle_point = Point::from(orig.0[smallest.current]);
        let right_point = Point::from(orig.0[right as usize]);

        let line_1 = CachedEnvelope::new(Line::new(left_point, middle_point));
        let line_2 = CachedEnvelope::new(Line::new(middle_point, right_point));
        assert!(tree.remove(&line_1).is_some());
        assert!(tree.remove(&line_2).is_some());

        // 恢复连续线段
        tree.insert(CachedEnvelope::new(Line::new(left_point, right_point)));

        // 使用左和右相邻点重新计算相邻的三角形，这可能会将新三角形添加到堆中
        recompute_triangles(&smallest, orig, &mut pq, ll, left, right, rr, max, epsilon);
    }
    // 过滤掉已删除的点，返回剩下的点
    orig.0
        .iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(*tup) } else { None })
        .collect()
}

/// 检查新的候选线段是否与任何现有的几何线段相交
///
/// 为了高效地做到这一点，rtree会查询位于候选线段创建的三角形的边界框内的任何现有段
fn tree_intersect<T>(
    tree: &RTree<CachedEnvelope<Line<T>>>,
    triangle: &VScore<T>,
    orig: &[Coord<T>],
) -> bool
where
    T: GeoFloat + RTreeNum,
{
    let new_segment_start = orig[triangle.left];
    let new_segment_end = orig[triangle.right];
    // 由候选点移除创建
    let new_segment = CachedEnvelope::new(Line::new(
        Point::from(orig[triangle.left]),
        Point::from(orig[triangle.right]),
    ));
    let bounding_rect = Triangle::new(
        orig[triangle.left],
        orig[triangle.current],
        orig[triangle.right],
    )
    .bounding_rect();
    tree.locate_in_envelope_intersecting(&rstar::AABB::from_corners(
        bounding_rect.min().into(),
        bounding_rect.max().into(),
    ))
    .any(|candidate| {
        // 线段起点，终点
        let (candidate_start, candidate_end) = candidate.points();
        candidate_start.0 != new_segment_start
            && candidate_start.0 != new_segment_end
            && candidate_end.0 != new_segment_start
            && candidate_end.0 != new_segment_end
            && new_segment.intersects(&**candidate)
    })
}

/// 简化几何图形。
///
/// 通过对其组成环上的算法运行来简化多边形。这可能导致无效的多边形，并且不保证维护拓扑。Multi*对象通过单独简化其所有组成几何体来简化。
///
/// 小于或等于零的epsilon将返回未更改的几何图形版本。
pub trait SimplifyVw<T, Epsilon = T> {
    /// 使用[Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263)算法返回简化的几何图形表示
    ///
    /// 查看[这里](https://bost.ocks.org/mike/simplify/)获取图形说明
    ///
    /// # 注意
    /// 用于删除点的公差是`epsilon`，与GEOS一致。JTS使用`epsilon ^ 2`。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::SimplifyVw;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 3.0, y: 8.0),
    ///     (x: 6.0, y: 20.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let simplified = line_string.simplify_vw(&30.0);
    ///
    /// let expected = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_vw(&self, epsilon: &T) -> Self
    where
        T: CoordFloat;
}

/// 简化几何体，返回输出的保留_indices_
///
/// 此操作使用Visvalingam-Whyatt算法，
/// 不**保证返回的几何图形有效。
///
/// 较大的`epsilon`意味着在去除点时更激进，而无需过多关注
/// 保持现有形状。具体来说，当你考虑是否去除一个点时，你可以画一个由候选点及其前后点组成的三角形。
/// 如果此三角形的面积小于`epsilon`，我们将移除点。
///
/// 小于或等于零的`epsilon`将返回未更改的几何图形版本。
pub trait SimplifyVwIdx<T, Epsilon = T> {
    /// 使用[Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263)算法返回简化的几何图形表示
    ///
    /// 查看[这里](https://bost.ocks.org/mike/simplify/)获取图形说明
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::SimplifyVwIdx;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 3.0, y: 8.0),
    ///     (x: 6.0, y: 20.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let simplified = line_string.simplify_vw_idx(&30.0);
    ///
    /// let expected = vec![
    ///     0_usize,
    ///     3_usize,
    ///     4_usize,
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_vw_idx(&self, epsilon: &T) -> Vec<usize>
    where
        T: CoordFloat;
}

/// 简化几何体，试图通过去除自交来保持其拓扑
///
/// 较大的`epsilon`意味着在去除点时更激进，而无需过多关注
/// 保持现有形状。具体来说，当你考虑是否去除一个点时，你可以画一个由候选点及其前后点组成的三角形。
/// 如果此三角形的面积小于`epsilon`，我们将移除点。
///
/// 小于或等于零的`epsilon`将返回未更改的几何图形版本。
pub trait SimplifyVwPreserve<T, Epsilon = T> {
    /// 使用一种保持拓扑的变体
    /// [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263)算法返回简化的几何图形表示。
    ///
    /// 查看[这里](https://www.jasondavies.com/simplify/)获取图形说明。
    ///
    /// 拓扑保持算法使用[R*树](../../../rstar/struct.RTree.html)来有效地查找与给定三角形相交的候选线段。
    /// 如果发现交集，则前一点（即当前三角形的左边组成部分）
    /// 也将被移除，从而改变几何图形，消除交集。
    ///
    /// 在下面的例子中，标准算法会保留(135.0, 68.0)，
    /// 形成三角形(0, 1, 3)，其与(280.0, 19.0)，(117.0, 48.0)和(117.0, 48.0)，(300.0, 40.0)的线段相交。通过移除它，
    /// 新形成的索引为(0, 3, 4)的三角形不会引起自交。
    ///
    /// # 注意
    ///
    /// - 简化算法可能会将多边形的内环移出其壳。
    /// - 该算法不**保证有效的输出几何体，特别是在较小的几何体上。
    /// - 如果去除一个点导致自交，但几何仅剩`n + 1`个点（`LineString`需要3个，`Polygon`需要5个），该点将被保留，简化过程结束。这是因为无法保证移除两个点会去除交集，但进一步移除点将使图形无法形成有效几何体。
    /// - 用于删除点的公差是`epsilon`，与GEOS一致。JTS使用`epsilon ^ 2`。
    ///
    /// # 示例
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::SimplifyVwPreserve;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 10., y: 60.),
    ///     (x: 135., y: 68.),
    ///     (x: 94., y: 48.),
    ///     (x: 126., y: 31.),
    ///     (x: 280., y: 19.),
    ///     (x: 117., y: 48.),
    ///     (x: 300., y: 40.),
    ///     (x: 301., y: 10.),
    /// ];
    ///
    /// let simplified = line_string.simplify_vw_preserve(&668.6);
    ///
    /// let expected = line_string![
    ///     (x: 10., y: 60.),
    ///     (x: 126., y: 31.),
    ///     (x: 280., y: 19.),
    ///     (x: 117., y: 48.),
    ///     (x: 300., y: 40.),
    ///     (x: 301., y: 10.),
    /// ];
    ///
    /// assert_relative_eq!(expected, simplified, epsilon = 1e-6);
    /// ```
    fn simplify_vw_preserve(&self, epsilon: &T) -> Self
    where
        T: CoordFloat + RTreeNum;
}

impl<T> SimplifyVwPreserve<T> for LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: &T) -> LineString<T> {
        let mut simplified = vwp_wrapper::<_, 2, 4>(self, None, epsilon);
        LineString::from(simplified.pop().unwrap())
    }
}

impl<T> SimplifyVwPreserve<T> for MultiLineString<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString::new(
            self.0
                .iter()
                .map(|l| l.simplify_vw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVwPreserve<T> for Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: &T) -> Polygon<T> {
        let mut simplified =
        // min_points过去是6，但对于小型多边形太保守
            vwp_wrapper::<_, 4, 5>(self.exterior(), Some(self.interiors()), epsilon);
        let exterior = LineString::from(simplified.remove(0));
        let interiors = simplified.into_iter().map(LineString::from).collect();
        Polygon::new(exterior, interiors)
    }
}

impl<T> SimplifyVwPreserve<T> for MultiPolygon<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: &T) -> MultiPolygon<T> {
        MultiPolygon::new(
            self.0
                .iter()
                .map(|p| p.simplify_vw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVw<T> for LineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: &T) -> LineString<T> {
        LineString::from(visvalingam(self, epsilon))
    }
}

impl<T> SimplifyVwIdx<T> for LineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw_idx(&self, epsilon: &T) -> Vec<usize> {
        visvalingam_indices(self, epsilon)
    }
}

impl<T> SimplifyVw<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString::new(self.iter().map(|l| l.simplify_vw(epsilon)).collect())
    }
}

impl<T> SimplifyVw<T> for Polygon<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: &T) -> Polygon<T> {
        Polygon::new(
            self.exterior().simplify_vw(epsilon),
            self.interiors()
                .iter()
                .map(|l| l.simplify_vw(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVw<T> for MultiPolygon<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: &T) -> MultiPolygon<T> {
        MultiPolygon::new(self.iter().map(|p| p.simplify_vw(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::{visvalingam, vwp_wrapper, SimplifyVw, SimplifyVwPreserve};
    use crate::{
        line_string, polygon, Coord, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    };

    // 参见 https://github.com/georust/geo/issues/1049
    #[test]
    #[should_panic]
    fn vwp_bug() {
        let pol = polygon![
            (x: 1., y: 4.),
            (x: 3., y: 4.),
            (x: 1., y: 1.),
            (x: 7., y: 0.),
            (x: 1., y: 0.),
            (x: 0., y: 1.),
            (x: 1., y: 4.),
        ];
        let simplified = pol.simplify_vw_preserve(&2.25);
        assert_eq!(
            simplified,
            polygon![
                (x: 1., y: 4.),
                (x: 3., y: 4.),
                (x: 1., y: 1.),
                (x: 7., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 4.),
            ]
        );
    }

    #[test]
    fn visvalingam_test() {
        // 这是PostGIS的例子
        let ls = line_string![
            (x: 5.0, y: 2.0),
            (x: 3.0, y: 8.0),
            (x: 6.0, y: 20.0),
            (x: 7.0, y: 25.0),
            (x: 10.0, y: 10.0)
        ];

        let correct = [(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Coord::from((e.0, e.1))).collect();

        let simplified = visvalingam(&ls, &30.);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn simple_vwp_test() {
        // 如果删除与最小相关面积的点，则此LineString将具有自交
        // 相关的三角形为(1, 2, 3)，面积为668.5
        // 新三角形(0, 1, 3)与三角形(3, 4, 5)自相交
        // 还必须删除点1，得到一个最终有效的
        // LineString： (0, 3, 4, 5, 6, 7)
        let ls = line_string![
            (x: 10., y:60.),
            (x: 135., y: 68.),
            (x: 94.,  y: 48.),
            (x: 126., y: 31.),
            (x: 280., y: 19.),
            (x: 117., y: 48.),
            (x: 300., y: 40.),
            (x: 301., y: 10.)
        ];
        let simplified = vwp_wrapper::<_, 2, 4>(&ls, None, &668.6);
        // 这是正确的、无自交的LineString
        let correct = [
            (10., 60.),
            (126., 31.),
            (280., 19.),
            (117., 48.),
            (300., 40.),
            (301., 10.),
        ];
        let correct_ls: Vec<_> = correct.iter().map(|e| Coord::from((e.0, e.1))).collect();
        assert_eq!(simplified[0], correct_ls);
    }
    #[test]
    fn retained_vwp_test() {
        // 我们希望删除outer[2]，因为其相关面积小于epsilon。
        // 然而，这导致了与内环的自交，
        // 这也会触发outer[1]的移除，使几何低于min_points。因此它被保留。
        // 内环也应该减少，但对于Polygon类型来说，其点数等于initial_min
        let outer = line_string![
            (x: -54.4921875, y: 21.289374355860424),
            (x: -33.5, y: 56.9449741808516),
            (x: -22.5, y: 44.08758502824516),
            (x: -19.5, y: 23.241346102386135),
            (x: -54.4921875, y: 21.289374355860424)
        ];
        let inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -29.513671875, y: 47.32027765985069),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplify_vw_preserve(&95.4);
        assert_relative_eq!(simplified.exterior(), &outer, epsilon = 1e-6);
    }
    #[test]
    fn remove_inner_point_vwp_test() {
        // 我们希望删除outer[2]，因为其相关面积小于epsilon。
        // 然而，这导致了与内环的自交，
        // 这也会触发outer[1]的移除，使几何低于min_points。因此它被保留。
        // 内环应通过去除inner[2]减少到四个点
        let outer = line_string![
            (x: -54.4921875, y: 21.289374355860424),
            (x: -33.5, y: 56.9449741808516),
            (x: -22.5, y: 44.08758502824516),
            (x: -19.5, y: 23.241346102386135),
            (x: -54.4921875, y: 21.289374355860424)
        ];
        let inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -40.0, y: 45.),
            (x: -29.513671875, y: 47.32027765985069),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let correct_inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -40.0, y: 45.0),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplify_vw_preserve(&95.4);
        assert_eq!(simplified.exterior(), &outer);
        assert_eq!(simplified.interiors()[0], correct_inner);
    }
    #[test]
    fn very_long_vwp_test() {
        // 简化8k点的LineString，消除自交
        let points_ls = geo_test_fixtures::norway_main::<f64>();
        let simplified = vwp_wrapper::<_, 2, 4>(&points_ls, None, &0.0005);
        assert_eq!(simplified[0].len(), 3278);
    }

    #[test]
    fn visvalingam_test_long() {
        // 简化较长的LineString
        let points_ls = geo_test_fixtures::vw_orig::<f64>();
        let correct_ls = geo_test_fixtures::vw_simplified::<f64>();
        let simplified = visvalingam(&points_ls, &0.0005);
        assert_eq!(simplified, correct_ls.0);
    }
    #[test]
    fn visvalingam_preserve_test_long() {
        // 使用保持变体简化较长的LineString
        let points_ls = geo_test_fixtures::vw_orig::<f64>();
        let correct_ls = geo_test_fixtures::vw_simplified::<f64>();
        let simplified = points_ls.simplify_vw_preserve(&0.0005);
        assert_relative_eq!(simplified, correct_ls, epsilon = 1e-6);
    }
    #[test]
    fn visvalingam_test_empty_linestring() {
        let vec: Vec<[f32; 2]> = Vec::new();
        let compare = Vec::new();
        let simplified = visvalingam(&LineString::from(vec), &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn visvalingam_test_two_point_linestring() {
        let vec = vec![Point::new(0.0, 0.0), Point::new(27.8, 0.1)];
        let compare = vec![Coord::from((0.0, 0.0)), Coord::from((27.8, 0.1))];
        let simplified = visvalingam(&LineString::from(vec), &1.0);
        assert_eq!(simplified, compare);
    }

    #[test]
    fn multilinestring() {
        // 这是PostGIS的例子
        let points = [
            (5.0, 2.0),
            (3.0, 8.0),
            (6.0, 20.0),
            (7.0, 25.0),
            (10.0, 10.0),
        ];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();

        let correct = [(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();

        let mline = MultiLineString::new(vec![LineString::from(points_ls)]);
        assert_relative_eq!(
            mline.simplify_vw(&30.),
            MultiLineString::new(vec![LineString::from(correct_ls)]),
            epsilon = 1e-6
        );
    }

    #[test]
    fn polygon() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ];

        let poly2 = poly.simplify_vw(&10.);

        assert_relative_eq!(
            poly2,
            polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.),
            ],
            epsilon = 1e-6
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon::new(vec![Polygon::new(
            LineString::from(vec![
                (0., 0.),
                (0., 10.),
                (5., 11.),
                (10., 10.),
                (10., 0.),
                (0., 0.),
            ]),
            vec![],
        )]);

        let mpoly2 = mpoly.simplify_vw(&10.);

        assert_relative_eq!(
            mpoly2,
            MultiPolygon::new(vec![Polygon::new(
                LineString::from(vec![(0., 0.), (0., 10.), (10., 10.), (10., 0.), (0., 0.)]),
                vec![],
            )]),
            epsilon = 1e-6
        );
    }
}
