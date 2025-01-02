use geo_types::{Coord, Line, Point, Triangle};
use spade::{
    ConstrainedDelaunayTriangulation, DelaunayTriangulation, Point2, SpadeNum, Triangulation,
};

use crate::{
    line_intersection::line_intersection, CoordsIter, Distance, Euclidean, GeoFloat,
    LineIntersection, LinesIter,
};
use crate::{Centroid, Contains};

// ======== 配置 ============

/// 一组参数，影响算法的精度（参见此结构体字段上的说明）
///
/// 这实现了 `Default` 特质，你可以在大多数情况下直接使用它。
#[derive(Debug, Clone)]
pub struct SpadeTriangulationConfig<T: SpadeTriangulationFloat> {
    /// 在此半径内的坐标会被吸附到相同的位置。对于任何两个 `Coords`，在选择哪个是吸附者和哪个被吸附者时没有办法影响决定。
    pub snap_radius: T,
}

impl<T> Default for SpadeTriangulationConfig<T>
where
    T: SpadeTriangulationFloat,
{
    fn default() -> Self {
        Self {
            snap_radius: <T as std::convert::From<f32>>::from(0.000_1),
        }
    }
}

// ====== 错误 ========

#[derive(Debug)]
pub enum TriangulationError {
    SpadeError(spade::InsertionError),
    LoopTrap,
    ConstraintFailure,
}

impl std::fmt::Display for TriangulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TriangulationError {}

pub type TriangulationResult<T> = Result<T, TriangulationError>;

// ======= 浮点特质 ========

pub trait SpadeTriangulationFloat: GeoFloat + SpadeNum {}
impl<T: GeoFloat + SpadeNum> SpadeTriangulationFloat for T {}

// ======= 三角化特质 ========

pub type Triangles<T> = Vec<Triangle<T>>;

// 密封特质，需实现才能实现 TriangulateSpade。这样做是为了不在公共接口上泄漏这些奇怪的方法。
mod private {
    use super::*;

    pub(crate) type CoordsIter<'a, T> = Box<dyn Iterator<Item = Coord<T>> + 'a>;

    pub trait TriangulationRequirementTrait<'a, T>
    where
        T: SpadeTriangulationFloat,
    {
        /// 从待进行三角剖分的几何对象中收集所有与三角剖分相关的线。
        ///
        /// 允许相交的线
        fn lines(&'a self) -> Vec<Line<T>>;
        /// 从待进行三角剖分的几何对象中收集所有与三角剖分相关的坐标
        fn coords(&'a self) -> CoordsIter<'a, T>;
        /// 定义一个谓词来判断点是否在对象内部（用于约束三角剖分）
        fn contains_point(&'a self, p: Point<T>) -> bool;

        // 准备进行三角剖分的线的处理。
        //
        // `spade` 有限制，不能有交叉的约束线，否则会引发异常。这就是为什么需要在交点处将线手动分割成更小的部分。
        //
        // 还有一个预处理步骤，试图通过边缘情况最小化算法失败的风险（尽量避免薄的/平的三角形，去重线等）
        fn cleanup_lines(lines: Vec<Line<T>>, snap_radius: T) -> TriangulationResult<Vec<Line<T>>> {
            let (known_coords, lines) = preprocess_lines(lines, snap_radius);
            prepare_intersection_contraint(lines, known_coords, snap_radius)
        }
    }
}

/// 使用[Delaunay三角剖分](https://en.wikipedia.org/wiki/Delaunay_triangulation)对多边形进行三角剖分
///
/// 此特质既包含约束三角剖分也包含非约束三角剖分方法。要了解这些方法之间的区别，请参阅[此页面](https://en.wikipedia.org/wiki/Constrained_Delaunay_triangulation)
pub trait TriangulateSpade<'a, T>: private::TriangulationRequirementTrait<'a, T>
where
    T: SpadeTriangulationFloat,
{
    /// 返回一个仅基于几何对象的点的三角剖分
    ///
    /// 三角剖分保证是Delaunay的
    ///
    /// 请注意，三角剖分的线不一定遵循输入几何的线。如果你希望实现这一点，请查看 `constrained_triangulation` 和 `constrained_outer_triangulation` 函数。
    ///
    /// ```rust
    /// use geo::TriangulateSpade;
    /// use geo::{Polygon, LineString, Coord};
    /// let u_shape = Polygon::new(
    ///     LineString::new(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 3.0 },
    ///         Coord { x: 0.0, y: 3.0 },
    ///     ]),
    ///     vec![],
    /// );
    /// let unconstrained_triangulation = u_shape.unconstrained_triangulation().unwrap();
    /// let num_triangles = unconstrained_triangulation.len();
    /// assert_eq!(num_triangles, 8);
    /// ```
    ///
    fn unconstrained_triangulation(&'a self) -> TriangulationResult<Triangles<T>> {
        let points = self.coords();
        points
            .into_iter()
            .map(to_spade_point)
            .try_fold(DelaunayTriangulation::<Point2<T>>::new(), |mut tris, p| {
                tris.insert(p).map_err(TriangulationError::SpadeError)?;
                Ok(tris)
            })
            .map(triangulation_to_triangles)
    }

    /// 返回基于几何对象的点并且也结合了输入几何体的线的三角剖分
    ///
    /// 由于约束线的原因，三角剖分不能保证是 Delaunay 的
    ///
    /// 如果输入几何体不是凸的，这个外部三角剖分还包含不在输入几何体中的三角形。以下是一个例子：
    ///
    /// ```text
    /// ┌──────────────────┐
    /// │\              __/│
    /// │ \          __/ / │
    /// │  \      __/   /  │
    /// │   \  __/     /   │
    /// │    \/       /    │
    /// │     ┌──────┐     │
    /// │    /│\:::::│\    │
    /// │   / │:\::::│ \   │
    /// │  /  │::\:::│  \  │
    /// │ /   │:::\::│   \ │
    /// │/    │::::\:│    \│
    /// └─────┘______└─────┘
    /// ```
    ///
    /// ```rust
    /// use geo::TriangulateSpade;
    /// use geo::{Polygon, LineString, Coord};
    /// let u_shape = Polygon::new(
    ///     LineString::new(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 3.0 },
    ///         Coord { x: 0.0, y: 3.0 },
    ///     ]),
    ///     vec![],
    /// );
    /// // 我们在这里使用默认的 [`SpadeTriangulationConfig`]
    /// let constrained_outer_triangulation =
    /// u_shape.constrained_outer_triangulation(Default::default()).unwrap();
    /// let num_triangles = constrained_outer_triangulation.len();
    /// assert_eq!(num_triangles, 8);
    /// ```
    ///
    /// 自上而下的 U 形状的外部三角剖分包含额外的三角形，标记为“:”。如果你想排除这些，请查看 `constrained_triangulation`
    fn constrained_outer_triangulation(
        &'a self,
        config: SpadeTriangulationConfig<T>,
    ) -> TriangulationResult<Triangles<T>> {
        let lines = self.lines();
        let lines = Self::cleanup_lines(lines, config.snap_radius)?;
        lines
            .into_iter()
            .map(to_spade_line)
            .try_fold(
                ConstrainedDelaunayTriangulation::<Point2<T>>::new(),
                |mut cdt, [start, end]| {
                    let start = cdt.insert(start).map_err(TriangulationError::SpadeError)?;
                    let end = cdt.insert(end).map_err(TriangulationError::SpadeError)?;
                    // 安全检查（以防止恐慌）检查我们是否可以添加这条线
                    if !cdt.can_add_constraint(start, end) {
                        return Err(TriangulationError::ConstraintFailure);
                    }
                    cdt.add_constraint(start, end);
                    Ok(cdt)
                },
            )
            .map(triangulation_to_triangles)
    }

    /// 返回基于几何对象的点并且也结合了输入几何体的线的三角剖分
    ///
    /// 由于约束线的原因，三角剖分不能保证是 Delaunay 的
    ///
    ///此三角剖分仅包含输入几何体内的三角形。以下是一个例子：
    ///
    /// ```text
    /// ┌──────────────────┐
    /// │\              __/│
    /// │ \          __/ / │
    /// │  \      __/   /  │
    /// │   \  __/     /   │
    /// │    \/       /    │
    /// │     ┌──────┐     │
    /// │    /│      │\    │
    /// │   / │      │ \   │
    /// │  /  │      │  \  │
    /// │ /   │      │   \ │
    /// │/    │      │    \│
    /// └─────┘      └─────┘
    /// ```
    ///
    /// ```rust
    /// use geo::TriangulateSpade;
    /// use geo::{Polygon, LineString, Coord};
    /// let u_shape = Polygon::new(
    ///     LineString::new(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 3.0 },
    ///         Coord { x: 0.0, y: 3.0 },
    ///     ]),
    ///     vec![],
    /// );
    /// // 我们在这里使用默认的 [`SpadeTriangulationConfig`]
    /// let constrained_triangulation = u_shape.constrained_triangulation(Default::default()).unwrap();
    /// let num_triangles = constrained_triangulation.len();
    /// assert_eq!(num_triangles, 6);
    /// ```
    ///
    /// 与 `constrained_outer_triangulation` 相比，它只包括输入几何体内部的三角形。
    fn constrained_triangulation(
        &'a self,
        config: SpadeTriangulationConfig<T>,
    ) -> TriangulationResult<Triangles<T>> {
        self.constrained_outer_triangulation(config)
            .map(|triangles| {
                triangles
                    .into_iter()
                    .filter(|triangle| {
                        let center = triangle.centroid();
                        self.contains_point(center)
                    })
                    .collect::<Vec<_>>()
            })
    }
}

/// 从 spade 三角剖分转换回 geo 三角形
fn triangulation_to_triangles<T, F>(triangulation: T) -> Triangles<F>
where
    T: Triangulation<Vertex = Point2<F>>,
    F: SpadeTriangulationFloat,
{
    triangulation
        .inner_faces()
        .map(|face| face.positions())
        .map(|points| points.map(|p| Coord::<F> { x: p.x, y: p.y }))
        .map(Triangle::from)
        .collect::<Vec<_>>()
}

// ========== Triangulation 特质实现 ============

// 任何满足要求的方法都自动实现三角剖分
impl<'a, T, G> TriangulateSpade<'a, T> for G
where
    T: SpadeTriangulationFloat,
    G: private::TriangulationRequirementTrait<'a, T>,
{
}

impl<'a, 'l, T, G> private::TriangulationRequirementTrait<'a, T> for G
where
    'a: 'l,
    T: SpadeTriangulationFloat,
    G: LinesIter<'l, Scalar = T> + CoordsIter<Scalar = T> + Contains<Point<T>>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.coords_iter())
    }

    fn lines(&'a self) -> Vec<Line<T>> {
        self.lines_iter().collect()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.contains(&p)
    }
}

// 实现特质 GS: AsRef<[G]> 会很酷，但我暂时无法让它编译。

impl<'a, T, G> private::TriangulationRequirementTrait<'a, T> for Vec<G>
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateSpade<'a, T>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.iter().flat_map(|g| g.coords()))
    }

    fn lines(&'a self) -> Vec<Line<T>> {
        self.iter().flat_map(|g| g.lines()).collect::<Vec<_>>()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.iter().any(|g| g.contains_point(p))
    }
}

impl<'a, T, G> private::TriangulationRequirementTrait<'a, T> for &[G]
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateSpade<'a, T>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.iter().flat_map(|g| g.coords()))
    }

    fn lines(&'a self) -> Vec<Line<T>> {
        self.iter().flat_map(|g| g.lines()).collect::<Vec<_>>()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.iter().any(|g| g.contains_point(p))
    }
}

// ========== Triangulation 特质实现助手 ============

fn prepare_intersection_contraint<T: SpadeTriangulationFloat>(
    mut lines: Vec<Line<T>>,
    mut known_points: Vec<Coord<T>>,
    snap_radius: T,
) -> Result<Vec<Line<T>>, TriangulationError> {
    // "Power of 10" 规则 2 (NASA)
    // 安全网。我们无法证明 `while let` 循环不会无限运行，因此在固定的迭代次数之后终止。万一迭代似乎无限循环，此检查将返回一个指示无限循环的错误。
    let mut loop_count = 1000;
    let mut loop_check = || {
        loop_count -= 1;
        (loop_count != 0)
            .then_some(())
            .ok_or(TriangulationError::LoopTrap)
    };

    while let Some((indices, intersection)) = {
        let mut iter = iter_line_pairs(&lines);
        iter.find_map(find_intersecting_lines_fn)
    } {
        loop_check()?;
        let [l0, l1] = remove_lines_by_index(indices, &mut lines);
        let new_lines = split_lines([l0, l1], intersection);
        let new_lines = cleanup_filter_lines(new_lines, &lines, &mut known_points, snap_radius);

        lines.extend(new_lines);
    }

    Ok(lines)
}

/// 在向量中迭代所有组合 (a,b)，其中 a != b
fn iter_line_pairs<T: SpadeTriangulationFloat>(
    lines: &[Line<T>],
) -> impl Iterator<Item = [(usize, &Line<T>); 2]> {
    lines.iter().enumerate().flat_map(|(idx0, line0)| {
        lines
            .iter()
            .enumerate()
            .skip(idx0 + 1)
            .filter(move |(idx1, line1)| *idx1 != idx0 && line0 != *line1)
            .map(move |(idx1, line1)| [(idx0, line0), (idx1, line1)])
    })
}

/// 检查两条线是否相交，如果是，则检查交点是否不规整
///
/// 返回
/// - [usize;2] : 排序后的线索引，较小的在前
/// - intersection : 交叉点类型
fn find_intersecting_lines_fn<T: SpadeTriangulationFloat>(
    [(idx0, line0), (idx1, line1)]: [(usize, &Line<T>); 2],
) -> Option<([usize; 2], LineIntersection<T>)> {
    line_intersection(*line0, *line1)
        .filter(|intersection| {
            match intersection {
                // 交点不在两条线上
                LineIntersection::SinglePoint { is_proper, .. } if !is_proper => false,
                // 共线交点是零长度线
                LineIntersection::Collinear { intersection }
                    if intersection.start == intersection.end =>
                {
                    false
                }
                _ => true,
            }
        })
        .map(|intersection| ([idx0, idx1], intersection))
}

/// 通过索引移除两条线，以安全的方式，因为在移除第一条线后，第二个索引可能无效（记住 `.remove(idx)` 返回元素并将向量的尾部朝起始方向移动以填补空缺）
fn remove_lines_by_index<T: SpadeTriangulationFloat>(
    mut indices: [usize; 2],
    lines: &mut Vec<Line<T>>,
) -> [Line<T>; 2] {
    indices.sort();
    let [idx0, idx1] = indices;
    let l1 = lines.remove(idx1);
    let l0 = lines.remove(idx0);
    [l0, l1]
}

/// 根据交叉类型划分线：
///
/// - 交点：从现有线的端点到交点创建4条新线
/// - 共线：创建3条新线（重叠之前，重叠，重叠之后）
fn split_lines<T: SpadeTriangulationFloat>(
    [l0, l1]: [Line<T>; 2],
    intersection: LineIntersection<T>,
) -> Vec<Line<T>> {
    match intersection {
        LineIntersection::SinglePoint { intersection, .. } => [
            (l0.start, intersection),
            (l0.end, intersection),
            (l1.start, intersection),
            (l1.end, intersection),
        ]
        .map(|(a, b)| Line::new(a, b))
        .to_vec(),
        LineIntersection::Collinear { .. } => {
            let mut points = [l0.start, l0.end, l1.start, l1.end];
            // 根据它们的坐标值对点进行排序，以解决歧义
            points.sort_by(|a, b| {
                a.x.partial_cmp(&b.x)
                    .expect("根据坐标x排序点失败")
                    .then_with(|| a.y.partial_cmp(&b.y).expect("根据坐标y排序点失败"))
            });
            // 由于所有点在一条线上，排序后创建连续点的新线
            points
                .windows(2)
                .map(|win| Line::new(win[0], win[1]))
                .collect::<Vec<_>>()
        }
    }
}

/// 来自 `split_lines` 函数的新线可能包含多种不当形成的线，此函数清理所有这些情况
fn cleanup_filter_lines<T: SpadeTriangulationFloat>(
    lines_need_check: Vec<Line<T>>,
    existing_lines: &[Line<T>],
    known_points: &mut Vec<Coord<T>>,
    snap_radius: T,
) -> Vec<Line<T>> {
    lines_need_check
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, known_points, snap_radius);
            line.end = snap_or_register_point(line.end, known_points, snap_radius);
            line
        })
        .filter(|l| l.start != l.end)
        .filter(|l| !existing_lines.contains(l))
        .filter(|l| !existing_lines.contains(&Line::new(l.end, l.start)))
        .collect::<Vec<_>>()
}

/// 如果非常接近，则将点吸附到最近的现有点上
///
/// 吸附半径可以通过此函数的第三个参数进行配置
fn snap_or_register_point<T: SpadeTriangulationFloat>(
    point: Coord<T>,
    known_points: &mut Vec<Coord<T>>,
    snap_radius: T,
) -> Coord<T> {
    known_points
        .iter()
        // 找到最接近的
        .min_by(|a, b| {
            Euclidean::distance(**a, point)
                .partial_cmp(&Euclidean::distance(**b, point))
                .expect("无法比较坐标距离")
        })
        // 仅当最近的在误差范围内时才吸附
        .filter(|nearest_point| Euclidean::distance(**nearest_point, point) < snap_radius)
        .cloned()
        // 否则注册并使用输入点
        .unwrap_or_else(|| {
            known_points.push(point);
            point
        })
}

/// 在使用spade三角剖分时，预处理线以减少问题
fn preprocess_lines<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    snap_radius: T,
) -> (Vec<Coord<T>>, Vec<Line<T>>) {
    let mut known_coords: Vec<Coord<T>> = vec![];
    let capacity = lines.len();
    let lines = lines
        .into_iter()
        .fold(Vec::with_capacity(capacity), |mut lines, mut line| {
            // 去重：

            // 1. 吸附线的坐标到现有坐标
            line.start = snap_or_register_point(line.start, &mut known_coords, snap_radius);
            line.end = snap_or_register_point(line.end, &mut known_coords, snap_radius);
            if
            // 2. 确保线段不是退化的（当起点=终点时，不存在长度）
            line.start != line.end
                // 3. 确保线或翻转后的线尚未添加
                && !lines.contains(&line)
                && !lines.contains(&Line::new(line.end, line.start))
            {
                lines.push(line)
            }

            lines
        });
    (known_coords, lines)
}

/// 将Line转换为某个在spade世界中类似的东西
fn to_spade_line<T: SpadeTriangulationFloat>(line: Line<T>) -> [Point2<T>; 2] {
    [to_spade_point(line.start), to_spade_point(line.end)]
}

/// 将Coord转换为某个在spade世界中类似的东西
fn to_spade_point<T: SpadeTriangulationFloat>(coord: Coord<T>) -> Point2<T> {
    Point2::new(coord.x, coord.y)
}

#[cfg(test)]
mod spade_triangulation {
    use super::*;
    use geo_types::*;

    fn assert_num_triangles<T: SpadeTriangulationFloat>(
        triangulation: &TriangulationResult<Triangles<T>>,
        num: usize,
    ) {
        assert_eq!(
            triangulation
                .as_ref()
                .map(|tris| tris.len())
                .expect("三角剖分成功"),
            num
        )
    }

    #[test]
    fn basic_triangle_triangulates() {
        let triangulation = Triangle::new(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        )
        .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 1);
    }

    #[test]
    fn basic_rectangle_triangulates() {
        let triangulation = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 })
            .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 2);
    }

    #[test]
    fn basic_polygon_triangulates() {
        let triangulation = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 1.0 },
                Coord { x: -1.0, y: 0.0 },
                Coord { x: -0.5, y: -1.0 },
                Coord { x: 0.5, y: -1.0 },
                Coord { x: 1.0, y: 0.0 },
            ]),
            vec![],
        )
        .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 3);
    }

    #[test]
    fn overlapping_triangles_triangulate_unconstrained() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let unconstrained_triangulation = triangles.unconstrained_triangulation();
        assert_num_triangles(&unconstrained_triangulation, 4);
    }

    #[test]
    fn overlapping_triangles_triangulate_constrained_outer() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let constrained_outer_triangulation =
            triangles.constrained_outer_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 8);
    }

    #[test]
    fn overlapping_triangles_triangulate_constrained() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let constrained_outer_triangulation =
            triangles.constrained_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 6);
    }

    #[test]
    fn u_shaped_polygon_triangulates_unconstrained() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let unconstrained_triangulation = u_shape.unconstrained_triangulation();
        assert_num_triangles(&unconstrained_triangulation, 8);
    }

    #[test]
    fn u_shaped_polygon_triangulates_constrained_outer() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let constrained_outer_triangulation =
            u_shape.constrained_outer_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 8);
    }

    #[test]
    fn u_shaped_polygon_triangulates_constrained_inner() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let constrained_triangulation = u_shape.constrained_triangulation(Default::default());
        assert_num_triangles(&constrained_triangulation, 6);
    }

    #[test]
    fn various_snap_radius_works() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        for snap_with in (1..6).map(|pow| 0.1_f64.powi(pow)) {
            let constrained_triangulation =
                u_shape.constrained_triangulation(SpadeTriangulationConfig {
                    snap_radius: snap_with,
                });
            assert_num_triangles(&constrained_triangulation, 6);
        }
    }
}
