use std::collections::HashMap;

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon, Triangle};

use crate::winding_order::{triangle_winding_order, WindingOrder};
use crate::{Contains, GeoFloat};

// ========= 异常类型 ============

#[derive(Debug)]
pub enum LineStitchingError {
    IncompleteRing(&'static str), // 环不完整
}

impl std::fmt::Display for LineStitchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for LineStitchingError {}

pub(crate) type TriangleStitchingResult<T> = Result<T, LineStitchingError>;

// ========= 主算法 ============

/// 用于拼接分割三角形的特质。
pub trait StitchTriangles<T: GeoFloat>: private::Stitchable<T> {
    /// 这个拼接只发生在位于两个独立几何体中的相同边缘上。请阅读输入的必要先决条件！
    ///
    /// ```text
    /// ┌─────x        ┌─────┐
    /// │    /│        │     │
    /// │   / │        │     │
    /// │  /  │  ───►  │     │
    /// │ /   │        │     │
    /// │/    │        │     │
    /// x─────┘        └─────┘
    /// ```
    ///
    /// # 前置条件
    ///
    /// - 输入中的三角形不能重叠！这也禁止在输入集中存在相同的三角形。如果想对重叠三角形进行并集操作，请参阅 `SpadeBoolops`。
    /// - 输入三角形应该是有效的多边形。关于有效性的定义请参阅 <https://www.postgis.net/workshops/postgis-intro/validity.html>
    ///
    /// # 示例
    ///
    /// ```text
    /// use geo::StitchTriangles;
    /// use geo::{Coord, Triangle, polygon};
    ///
    /// let tri1 = Triangle::from([
    ///     Coord { x: 0.0, y: 0.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    /// ]);
    /// let tri2 = Triangle::from([
    ///     Coord { x: 1.0, y: 1.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    /// ]);
    ///
    /// let result = vec![tri1, tri2].stitch_triangulation();
    ///
    /// assert!(result.is_ok());
    ///
    /// let mp = result.unwrap();
    ///
    /// assert_eq!(mp.0.len(), 1);
    ///
    /// let poly = mp.0[0].clone();
    /// // 4个坐标 + 1个用于闭合的重复坐标
    /// assert_eq!(poly.exterior().0.len(), 4 + 1);
    ///
    /// let expected = polygon![
    ///     Coord { x: 1.0, y: 1.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    ///     Coord { x: 0.0, y: 0.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    /// ];
    ///
    /// assert_eq!(poly, expected);
    /// ```
    ///
    /// # 额外说明
    ///
    /// 当拼接三角形导致一个多边形的孔与外轮廓相接（如香蕉多边形中提到的 [banana polygon](https://postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity)），将导致一个没有内环的单一多边形，而不是一个带有单个内环的多边形。
    ///
    /// ```text
    /// ┌────────x────────┐
    /// │\....../ \....../│
    /// │.\..../   \..../.│
    /// │..\../     \../..│
    /// │...\/       \/...│
    /// │...───────────...│
    /// │../\....^..../\..│
    /// │./..\../.\../..\.│
    /// │/....\/...\/....\│
    /// └─────────────────┘
    ///
    ///     │    │    │
    ///     ▼    ▼    ▼
    ///
    /// ┌────────x────────┐
    /// │       / \       │
    /// │      /   \      │
    /// │     /     \     │
    /// │    /       \    │
    /// │   ───────────   │
    /// │                 │
    /// │                 │
    /// │                 │
    /// └─────────────────┘
    /// ```
    ///
    /// ---
    ///
    /// 如果想进行更一般的操作，比如 [`布尔操作联合`](https://en.wikipedia.org/wiki/Boolean_operations_on_polygons)，你应该使用 `BooleanOps` 或 `SpadeBoolops` 特质。
    fn stitch_triangulation(&self) -> TriangleStitchingResult<MultiPolygon<T>>;
}

mod private {
    use super::*;

    pub trait Stitchable<T: GeoFloat>: AsRef<[Triangle<T>]> {}
    impl<S, T> Stitchable<T> for S
    where
        S: AsRef<[Triangle<T>]>,
        T: GeoFloat,
    {
    }
}

impl<S, T> StitchTriangles<T> for S
where
    S: private::Stitchable<T>,
    T: GeoFloat,
{
    fn stitch_triangulation(&self) -> TriangleStitchingResult<MultiPolygon<T>> {
        stitch_triangles(self.as_ref().iter())
    }
}

// 主拼接算法
fn stitch_triangles<'a, T, S>(triangles: S) -> TriangleStitchingResult<MultiPolygon<T>>
where
    T: GeoFloat + 'a,
    S: Iterator<Item = &'a Triangle<T>>,
{
    let lines = triangles.flat_map(ccw_lines).collect::<Vec<_>>();

    let boundary_lines = find_boundary_lines(lines);
    let stitched_multipolygon = stitch_multipolygon_from_lines(boundary_lines)?;

    let polys = stitched_multipolygon
        .into_iter()
        .map(find_and_fix_holes_in_exterior)
        .collect::<Vec<_>>();

    Ok(MultiPolygon::new(polys))
}

/// 返回逆时针方向的三角形边
fn ccw_lines<T: GeoFloat>(tri: &Triangle<T>) -> [Line<T>; 3] {
    match triangle_winding_order(tri) {
        Some(WindingOrder::CounterClockwise) => tri.to_lines(),
        _ => {
            let [a, b, c] = tri.to_array();
            [(b, a), (a, c), (c, b)].map(|(start, end)| Line::new(start, end))
        }
    }
}

/// 检查两条线是否相等或互为倒置
#[inline]
fn same_line<T: GeoFloat>(l1: &Line<T>, l2: &Line<T>) -> bool {
    (l1.start == l2.start && l1.end == l2.end) || (l1.start == l2.end && l2.start == l1.end)
}

/// 给定一个由多个多边形分割区域线条的集合，我们可以有两种线条：
///
/// - 边界线：这是由一组多边形形成的复合形状边界的唯一线
/// - 内部线：这是所有非边界线，它们不是唯一的，并且在集合中的一个相邻多边形上有一个完全相同的副本（只要输入有效！）
fn find_boundary_lines<T: GeoFloat>(lines: Vec<Line<T>>) -> Vec<Line<T>> {
    lines.into_iter().fold(Vec::new(), |mut lines, new_line| {
        if let Some(idx) = lines.iter().position(|line| same_line(line, &new_line)) {
            lines.remove(idx);
        } else {
            lines.push(new_line);
        }
        lines
    })
}

// 未来注意：这可能属于一个 `Validify` 特质或类似的东西
/// 在多边形外部查找并修复孔洞
///
/// 这对像香蕉多边形这样的场景很重要。这被认为是无效的
/// https://www.postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity
fn find_and_fix_holes_in_exterior<F: GeoFloat>(mut poly: Polygon<F>) -> Polygon<F> {
    fn detect_if_rings_closed_with_point<F: GeoFloat>(
        points: &mut Vec<Coord<F>>,
        p: Coord<F>,
    ) -> Option<Vec<Coord<F>>> {
        // 如果没有找到，则直接返回
        let pos = points.iter().position(|&c| c == p)?;

        // 如果找到，则通过收集点创建环
        let ring = points
            .drain(pos..)
            .chain(std::iter::once(p))
            .collect::<Vec<_>>();
        Some(ring)
    }

    // 查找环
    let rings = {
        let (points, mut rings) =
            poly.exterior()
                .into_iter()
                .fold((vec![], vec![]), |(mut points, mut rings), coord| {
                    rings.extend(detect_if_rings_closed_with_point(&mut points, *coord));
                    points.push(*coord);
                    (points, rings)
                });

        // 将剩余的坐标作为最后一个环添加
        rings.push(points);

        rings
    };

    // 转换为多边形用于包含性检查
    let mut rings = rings
        .into_iter()
        // 过滤掉上面的代码可能产生的退化多边形
        .filter(|cs| cs.len() >= 3)
        .map(|cs| Polygon::new(LineString::new(cs), vec![]))
        .collect::<Vec<_>>();

    // 性能： O(n^2) 也许有人可以减少这个。请进行基准测试！
    fn find_outmost_ring<F: GeoFloat>(rings: &[Polygon<F>]) -> Option<usize> {
        let enumerated_rings = || rings.iter().enumerate();
        enumerated_rings()
            .find(|(i, ring)| {
                enumerated_rings()
                    .filter(|(j, _)| i != j)
                    .all(|(_, other)| ring.contains(other))
            })
            .map(|(i, _)| i)
    }

    // 如果存在包含所有其他环的外部环，则重新创建多边形：
    //
    // - 外环作为外部环
    // - 其他环计为内环
    // - 先前存在的内环将被保留
    if let Some(outer_index) = find_outmost_ring(&rings) {
        let exterior = rings.remove(outer_index).exterior().clone();
        let interiors = poly
            .interiors()
            .iter()
            .cloned()
            .chain(rings.into_iter().map(|p| p.exterior().clone()))
            .collect::<Vec<_>>();
        poly = Polygon::new(exterior, interiors);
    }
    poly
}

/// 该函数的输入是必须形成一个有效多边形的无序线条集合
fn stitch_multipolygon_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> TriangleStitchingResult<MultiPolygon<F>> {
    let rings = stitch_rings_from_lines(lines)?;

    fn find_parent_idxs<F: GeoFloat>(
        ring_idx: usize,
        ring: &LineString<F>,
        all_rings: &[LineString<F>],
    ) -> Vec<usize> {
        all_rings
            .iter()
            .enumerate()
            .filter(|(other_idx, _)| ring_idx != *other_idx)
            .filter_map(|(idx, maybe_parent)| {
                Polygon::new(maybe_parent.clone(), vec![])
                    .contains(ring)
                    .then_some(idx)
            })
            .collect()
    }

    // 关联每个环与其父级（包含它们的环）
    let parents_of: HashMap<usize, Vec<usize>> = rings
        .iter()
        .enumerate()
        .map(|(ring_idx, ring)| {
            let parent_idxs = find_parent_idxs(ring_idx, ring, &rings);
            (ring_idx, parent_idxs)
        })
        .collect();

    // 关联外环和其内部环
    let mut polygons_idxs: HashMap<usize, Vec<usize>> = HashMap::default();

    // 直接父级是自身具有最多父级环的父级环
    fn find_direct_parent(
        parent_rings: &[usize],
        parents_of: &HashMap<usize, Vec<usize>>,
    ) -> Option<usize> {
        parent_rings
            .iter()
            .filter_map(|ring_idx| {
                parents_of
                    .get(ring_idx)
                    .map(|grandparent_rings| (ring_idx, grandparent_rings))
            })
            .max_by_key(|(_, grandparent_rings)| grandparent_rings.len())
            .map(|(idx, _)| idx)
            .copied()
    }

    // 对于每个环，我们检查它有多少个父级，否则它就是一个外部环
    //
    // 在“甜甜圈”场景中这点很重要，因为外部甜甜圈形状的多边形完全包含了其孔内的较小多边形。
    for (ring_index, parent_idxs) in parents_of.iter() {
        let parent_count = parent_idxs.len();

        // 如果有偶数个父级，则它是一个外部环，因此如果它缺失，我们就可以直接添加
        if parent_count % 2 == 0 {
            polygons_idxs.entry(*ring_index).or_default();
            continue;
        }

        // 如果有奇数个父级，那它是一个内部环

        // 为了找到与之相关的特定外部环，我们搜索直接父级。
        let maybe_direct_parent = find_direct_parent(parent_idxs, &parents_of);

        // 如上所述，这里的父级数量是奇数，因此至少为一。
        // 由于每个环都在 `parents` 哈希表中注册，因此在迭代时我们至少会找到一个元素。因此 `max_by_key` 永远不会返回空，而是返回 `Some` 。
        debug_assert!(maybe_direct_parent.is_some(), "一个直接的父级必须存在");

        // 我没有直接解引用，因为我担心出现恐慌
        if let Some(direct_parent) = maybe_direct_parent {
            polygons_idxs
                .entry(direct_parent)
                .or_default()
                .push(*ring_index);
        }
    }

    // 根据索引查找环并创建多边形
    let polygons = polygons_idxs
        .into_iter()
        .map(|(parent_idx, children_idxs)| {
            // 性能：这里有过多的克隆操作，也许有人能改进这个。请进行基准测试！
            let exterior = rings[parent_idx].clone();
            let interiors = children_idxs
                .into_iter()
                .map(|child_idx| rings[child_idx].clone())
                .collect::<Vec<_>>();
            (exterior, interiors)
        })
        .map(|(exterior, interiors)| Polygon::new(exterior, interiors));

    Ok(polygons.collect())
}

// ============== 帮助函数 ================

fn stitch_rings_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> TriangleStitchingResult<Vec<LineString<F>>> {
    // 初始的环部分只是线段，它们将被逐步拼接在一起
    let mut ring_parts: Vec<Vec<Coord<F>>> = lines
        .iter()
        .map(|line| vec![line.start, line.end])
        .collect();

    let mut rings: Vec<LineString<F>> = vec![];
    // 循环终止条件是每轮循环我们将把两个元素合并成一个，因此每次循环元素总数减少至少一个（在完成一个环的情况下是两个）
    while let Some(last_part) = ring_parts.pop() {
        let (j, compound_part) = ring_parts
            .iter()
            .enumerate()
            .find_map(|(j, other_part)| {
                let new_part = try_stitch(&last_part, other_part)?;
                Some((j, new_part))
            })
            .ok_or(LineStitchingError::IncompleteRing(
                "无法从输入中重构多边形。请检查它们的有效性。",
            ))?;
        ring_parts.remove(j);

        let is_ring = compound_part.first() == compound_part.last() && !compound_part.is_empty();

        if is_ring {
            let new_ring = LineString::new(compound_part);
            rings.push(new_ring);
        } else {
            ring_parts.push(compound_part);
        }
    }

    Ok(rings)
}

fn try_stitch<F: GeoFloat>(a: &[Coord<F>], b: &[Coord<F>]) -> Option<Vec<Coord<F>>> {
    let a_first = a.first()?;
    let a_last = a.last()?;
    let b_first = b.first()?;
    let b_last = b.last()?;

    let a = || a.iter();
    let b = || b.iter();

    // _ -> X  |  X -> _
    (a_last == b_first)
        .then(|| a().chain(b().skip(1)).cloned().collect())
        // X -> _  |  _ -> X
        .or_else(|| (a_first == b_last).then(|| b().chain(a().skip(1)).cloned().collect()))
}

// ============= 测试 ===========

#[cfg(test)]
mod polygon_stitching_tests {

    use crate::{Relate, TriangulateEarcut, Winding};

    use super::*;
    use geo_types::*;

    #[test]
    fn poly_inside_a_donut() {
        _ = pretty_env_logger::try_init();
        let zero = Coord::zero();
        let one = Point::new(1.0, 1.0).0;
        let outer_outer = Rect::new(zero, one * 5.0);
        let inner_outer = Rect::new(one, one * 4.0);
        let outer = Polygon::new(
            outer_outer.to_polygon().exterior().clone(),
            vec![inner_outer.to_polygon().exterior().clone()],
        );
        let inner = Rect::new(one * 2.0, one * 3.0).to_polygon();

        let mp = MultiPolygon::new(vec![outer.clone(), inner.clone()]);

        let tris = [inner, outer].map(|p| p.earcut_triangles()).concat();

        let result = tris.stitch_triangulation().unwrap();

        assert!(mp.relate(&result).is_equal_topo());
    }

    #[test]
    fn stitch_independent_of_orientation() {
        _ = pretty_env_logger::try_init();
        let mut tri1 = Triangle::from([
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        ])
        .to_polygon();
        let mut tri2 = Triangle::from([
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        ])
        .to_polygon();

        tri1.exterior_mut(|ls| ls.make_ccw_winding());
        tri2.exterior_mut(|ls| ls.make_ccw_winding());
        let result_1 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_ccw_winding());
        let result_2 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_3 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_ccw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_4 = [tri1, tri2]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert!(result_1.relate(&result_2).is_equal_topo());
        assert!(result_2.relate(&result_3).is_equal_topo());
        assert!(result_3.relate(&result_4).is_equal_topo());
    }

    #[test]
    fn stitch_creating_hole() {
        let poly1 = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 1.0, y: 2.0 },
                Coord { x: 2.0, y: 2.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::new(vec![
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 1.0, y: 1.0 },
            ]),
            vec![],
        );

        let result = [poly1, poly2]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].interiors().len(), 1);
    }

    #[test]
    fn inner_banana_produces_hole() {
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 3.0, y: 2.0 },
                Coord { x: 5.0, y: 2.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 8.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
            ]),
            vec![],
        );

        let result = [poly]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].interiors().len(), 1);
    }

    #[test]
    fn outer_banana_doesnt_produce_hole() {
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 3.0, y: -2.0 },
                Coord { x: 5.0, y: -2.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 8.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
            ]),
            vec![],
        );

        let result = [poly]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].interiors().len(), 0);
    }
}
