use crate::{coordinate_position::CoordPos, dimensions::Dimensions, GeoNum, GeometryCow};

use crate::geometry_cow::GeometryCow::Point;
use crate::relate::geomgraph::intersection_matrix::dimension_matcher::DimensionMatcher;
use std::str::FromStr;

/// 模型化一个*维度扩展的九交模型(DE-9IM)*矩阵。
///
/// DE-9IM 矩阵值（例如“212FF1FF2”）指定两个[几何体](struct.Geometry.html)之间的拓扑关系。
///
/// DE-9IM矩阵是表示几何体中发生的拓扑位置（内部、边界、外部）的3x3矩阵。
///
/// 索引用枚举[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside](CoordPos)提供。
///
/// 矩阵条目代表每个交点的[Dimensions](enum.Dimension.html)。
///
/// 有关 DE-9IM 及其派生的空间谓词的描述，请参阅以下参考文献：
/// - [OGC 99-049 OpenGIS SQL 简单特性规范](http://portal.opengeospatial.org/files/?artifact_id=829)，第 2.1.13 节
/// - [OGC 06-103r4 地理信息 - 简单特性访问 - 第 1 部分：通用架构的 OpenGIS 实现标准](http://portal.opengeospatial.org/files/?artifact_id=25355)，第 6.1.15 节（其中提供了有关某些谓词规范的更多详细信息）。
/// - 关于[DE-9IM](https://en.wikipedia.org/wiki/DE-9IM)的维基百科文章
///
/// 此实现很大程度上基于[JTS项目](https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/geom/IntersectionMatrix.java)。
#[derive(PartialEq, Eq, Clone)]
pub struct IntersectionMatrix(LocationArray<LocationArray<Dimensions>>);

/// 辅助结构以便我们可以通过 CoordPos 索引 IntersectionMatrix
///
/// CoordPos 枚举成员的顺序为：OnBoundary, Inside, Outside
/// DE-9IM 矩阵的顺序为：Inside, Boundary, Exterior
///
/// 因此，我们不能简单地使用 `CoordPos as usize`，否则会丢失元素的传统顺序，这对于调试/互操作很有用。
#[derive(PartialEq, Eq, Clone, Copy)]
struct LocationArray<T>([T; 3]);

impl<T> LocationArray<T> {
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T> std::ops::Index<CoordPos> for LocationArray<T> {
    type Output = T;

    fn index(&self, index: CoordPos) -> &Self::Output {
        match index {
            CoordPos::Inside => &self.0[0],
            CoordPos::OnBoundary => &self.0[1],
            CoordPos::Outside => &self.0[2],
        }
    }
}

impl<T> std::ops::IndexMut<CoordPos> for LocationArray<T> {
    fn index_mut(&mut self, index: CoordPos) -> &mut Self::Output {
        match index {
            CoordPos::Inside => &mut self.0[0],
            CoordPos::OnBoundary => &mut self.0[1],
            CoordPos::Outside => &mut self.0[2],
        }
    }
}

#[derive(Debug)]
pub struct InvalidInputError {
    message: String,
}

impl InvalidInputError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::error::Error for InvalidInputError {}
impl std::fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "输入无效:  {}", self.message)
    }
}

impl std::fmt::Debug for IntersectionMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn char_for_dim(dim: &Dimensions) -> &'static str {
            match dim {
                Dimensions::Empty => "F",
                Dimensions::ZeroDimensional => "0",
                Dimensions::OneDimensional => "1",
                Dimensions::TwoDimensional => "2",
            }
        }
        let text = self
            .0
            .iter()
            .flat_map(|r| r.iter().map(char_for_dim))
            .collect::<Vec<&str>>()
            .join("");

        write!(f, "IntersectionMatrix({})", &text)
    }
}

impl IntersectionMatrix {
    pub const fn empty() -> Self {
        IntersectionMatrix(LocationArray([LocationArray([Dimensions::Empty; 3]); 3]))
    }

    pub(crate) const fn empty_disjoint() -> Self {
        IntersectionMatrix(LocationArray([
            LocationArray([Dimensions::Empty, Dimensions::Empty, Dimensions::Empty]),
            LocationArray([Dimensions::Empty, Dimensions::Empty, Dimensions::Empty]),
            // 因为几何体是有限的并且嵌入在二维空间中，
            // 所以 `(Outside, Outside)` 元素必须始终是二维的
            LocationArray([
                Dimensions::Empty,
                Dimensions::Empty,
                Dimensions::TwoDimensional,
            ]),
        ]))
    }

    /// 如果几何体是不相交的，我们需要在IM的`Outside`行中输入它们的维度和边界维度
    pub(crate) fn compute_disjoint<F: GeoNum>(
        &mut self,
        geometry_a: &GeometryCow<F>,
        geometry_b: &GeometryCow<F>,
    ) {
        use crate::algorithm::dimensions::HasDimensions;
        {
            let dimensions = geometry_a.dimensions();
            if dimensions != Dimensions::Empty {
                self.set(CoordPos::Inside, CoordPos::Outside, dimensions);

                let boundary_dimensions = geometry_a.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    self.set(CoordPos::OnBoundary, CoordPos::Outside, boundary_dimensions);
                }
            }
        }

        {
            let dimensions = geometry_b.dimensions();
            if dimensions != Dimensions::Empty {
                self.set(CoordPos::Outside, CoordPos::Inside, dimensions);

                let boundary_dimensions = geometry_b.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    self.set(CoordPos::Outside, CoordPos::OnBoundary, boundary_dimensions);
                }
            }
        }
    }

    /// 设置由位置指定的格子的`dimensions`。
    ///
    /// `position_a`：第一个几何体中应用`dimensions`的位置
    /// `position_b`：第二个几何体中应用`dimensions`的位置
    /// `dimensions`：事件的维度
    pub(crate) fn set(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        dimensions: Dimensions,
    ) {
        self.0[position_a][position_b] = dimensions;
    }

    /// 报告一个`dimensions`事件，如果它大于到目前为止报道的，则更新IntersectionMatrix。
    ///
    /// `position_a`：第一个几何体中应用`minimum_dimensions`的位置
    /// `position_b`：第二个几何体中应用`minimum_dimensions`的位置
    /// `minimum_dimensions`：事件的维度
    pub(crate) fn set_at_least(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        minimum_dimensions: Dimensions,
    ) {
        if self.0[position_a][position_b] < minimum_dimensions {
            self.0[position_a][position_b] = minimum_dimensions;
        }
    }

    /// 如果两个几何体都具有某个位置，则将指定元素更改为至少`minimum_dimensions`。
    ///
    /// 否则，如果任意一个为无，则不做任何操作。
    ///
    /// `position_a`：第一个几何体中应用`minimum_dimensions`的位置，或者如果维度未与第一个几何体发生关系，则为`None`。
    /// `position_b`：第二个几何体中应用`minimum_dimensions`的位置，或者如果维度未与第二个几何体发生关系，则为`None`。
    /// `minimum_dimensions`：事件的维度
    pub(crate) fn set_at_least_if_in_both(
        &mut self,
        position_a: Option<CoordPos>,
        position_b: Option<CoordPos>,
        minimum_dimensions: Dimensions,
    ) {
        if let (Some(position_a), Some(position_b)) = (position_a, position_b) {
            self.set_at_least(position_a, position_b, minimum_dimensions);
        }
    }

    pub(crate) fn set_at_least_from_string(
        &mut self,
        dimensions: &str,
    ) -> Result<(), InvalidInputError> {
        if dimensions.len() != 9 {
            let message = format!("期望的维度长度为9，发现: {}", dimensions.len());
            return Err(InvalidInputError::new(message));
        }

        let mut chars = dimensions.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                match chars.next().expect("长度已经验证为9") {
                    '0' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::ZeroDimensional),
                    '1' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::OneDimensional),
                    '2' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::TwoDimensional),
                    'F' => {}
                    other => {
                        let message = format!("预期为'0', '1', '2', 或 'F'。发现: {other}");
                        return Err(InvalidInputError::new(message));
                    }
                }
            }
        }

        Ok(())
    }

    // 实现者注意
    // 请参阅 https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates 以了解谓词与矩阵之间的映射
    // 您的关系列函数中的约束数量必须与非掩码（T 或 F）矩阵条目数匹配

    // IntersectionMatrix的索引映射到DE-9IM规范字符串的索引如下：
    // ==================================================================
    // self.0[CoordPos::Inside][CoordPos::Inside]: 0
    // self.0[CoordPos::Inside][CoordPos::OnBoundary]: 1
    // self.0[CoordPos::Inside][CoordPos::Outside]: 2

    // self.0[CoordPos::OnBoundary][CoordPos::Inside]: 3
    // self.0[CoordPos::OnBoundary][CoordPos::OnBoundary]: 4
    // self.0[CoordPos::OnBoundary][CoordPos::Outside]: 5

    // self.0[CoordPos::Outside][CoordPos::Inside]: 6
    // self.0[CoordPos::Outside][CoordPos::OnBoundary]: 7
    // self.0[CoordPos::Outside][CoordPos::Outside]: 8
    // ==================================================================

    // 矩阵条目与Dimensions之间的关系
    // ==================================================================
    // 一个 `T` 条目转换为 `!= Dimensions::Empty`
    // 一个 `F` 条目转换为 `== Dimensions::Empty`
    // 一个 `*`（掩码）条目被省略
    // ==================================================================

    // 示例
    // ==================================================================
    // `[T********]` -> `self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty`
    // `[********F]` -> `self.0[CoordPos::Outside][CoordPos::Outside] == Dimensions::Empty`
    // `[**T****F*]` -> `self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
    //     && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty`
    // ==================================================================

    /// 如果几何体 `a` 和 `b` 是分开的，返回 `true`：它们没有共同点，
    /// 形成一组不连接的几何体。
    ///
    /// # 注意
    /// - 匹配 `[FF*FF****]`
    /// - 这个谓词是 **反自反** 的
    pub fn is_disjoint(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] == Dimensions::Empty
    }

    /// 测试 [`IntersectionMatrix::is_disjoint`] 是否返回 `false`。
    ///
    /// 如果这个矩阵关系的两个几何体相交，返回 `true`：它们至少有一个共同点。
    ///
    /// # 注意
    /// - 匹配任何 `[T********], [*T*******], [***T*****], [****T****]`
    /// - 这个谓词是**自反和对称**的
    pub fn is_intersects(&self) -> bool {
        !self.is_disjoint()
    }

    /// 如果第一个几何体在第二个几何体内，返回 `true`：`a` 位于 `b` 的内部。
    ///
    ///
    /// # 注意
    /// - 也称为 **inside**
    /// - 掩码 `[T*F**F***`] 出现在 [`IntersectionMatrix::is_within`] 和 [`IntersectionMatrix::is_coveredby`] 的定义中；对于**大多数**情况，应该优先使用 [`IntersectionMatrix::is_coveredby`] 而不是 [`IntersectionMatrix::is_within`]
    /// - 这个谓词是 **自反和传递** 的
    pub fn is_within(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    /// 如果几何体 `a` 包含几何体 `b`，返回 `true`。
    ///
    /// # 注意
    /// - 匹配 `[T*****FF*]`
    /// - 这个谓词是 **自反和传递** 的
    pub fn is_contains(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
    }

    /// 如果第一个几何体与第二个几何体 *拓扑等价*，返回 `true`。
    ///
    /// # 注意
    /// - 匹配 `[T*F**FFF*]`
    /// - 这个谓词是**自反、对称和传递**的
    pub fn is_equal_topo(&self) -> bool {
        if self == &Self::empty_disjoint() {
            // 任何两个空几何体在拓扑上是相等的
            return true;
        }

        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    /// 如果几何体 `a` 中的每个点都在几何体 `b` 内部(即与`b`的内部或边界相交)，返回 true。
    ///
    /// 等效地，测试 `a` 的任何点都不在 `b` 的外部(即在 `b` 的外部):
    /// - `a` 被 `b` 覆盖(扩展[`IntersectionMatrix::is_within`])：几何体`a`位于`b`内部或者
    /// - `a` 的至少**一个**点位于 `b`，并且 `a` 的**没有**点位于 `b` 的**外部**，或者
    /// - `a` 的**每个**点都是( `b` 的**内部**或**边界**的)一点
    ///
    /// 返回 true 如果第一个几何体被第二个几何体覆盖。
    ///
    /// ```
    /// use geo_types::{Polygon, polygon};
    /// use geo::relate::Relate;
    ///
    /// let poly1 = polygon![
    ///     (x: 125., y: 179.),
    ///     (x: 110., y: 150.),
    ///     (x: 160., y: 160.),
    ///     (x: 125., y: 179.),
    /// ];
    /// let poly2 = polygon![
    ///     (x: 124., y: 182.),
    ///     (x: 106., y: 146.),
    ///     (x: 162., y: 159.),
    ///     (x: 124., y: 182.),
    /// ];
    ///
    /// let intersection = poly1.relate(&poly2);
    /// assert_eq!(intersection.is_coveredby(), true);
    /// ```
    ///
    /// # 注意
    /// - 匹配任何 `[T*F**F***], [*TF**F***], [**FT*F***], [**F*TF***]`
    /// - 这个谓词是 **自反和传递** 的
    #[allow(clippy::nonminimal_bool)]
    pub fn is_coveredby(&self) -> bool {
        // [T*F**F***]
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [*TF**F***]
        self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [**FT*F***]
        self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty ||
        // [**F*TF***]
        self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    /// 如果几何体 `b` 中的每个点都在几何体 `a` 内部
    /// (即与`a`的内部或边界相交)。等效地，
    /// 测试 `b` 的任何点都不在 `a` 的外部。
    ///
    /// # 注意
    /// - 与 [`IntersectionMatrix::is_contains`] 不同，它**不**区分几何体中边界点和内部点
    /// - 对于**大多数**情况，应该优先使用 [`IntersectionMatrix::is_covers`] 而不是 [`IntersectionMatrix::is_contains`]
    /// - 匹配任何 `[T*****FF*], [*T****FF*], [***T**FF*], [****T*FF*]`
    /// - 这个谓词是 **自反和传递** 的
    #[allow(clippy::nonminimal_bool)]
    pub fn is_covers(&self) -> bool {
        // [T*****FF*]
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [*T****FF*]
        self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [***T**FF*]
        self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty ||
        // [****T*FF*]
        self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
    }

    /// 如果 `a` 接触 `b`，返回 `true`：它们至少有一个共同点，但它们的内部不相交。
    ///
    /// # 注意
    /// - 匹配任何 `[FT*******], [F**T*****], [F***T****]`
    /// - 这个谓词是 **对称的**
    #[allow(clippy::nonminimal_bool)]
    pub fn is_touches(&self) -> bool {
        // [FT*******]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::Inside][CoordPos::OnBoundary] != Dimensions::Empty ||
        // [F**T*****]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::OnBoundary][CoordPos::Inside] != Dimensions::Empty ||
        // [F***T****]
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
        && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] != Dimensions::Empty
    }

    /// 比较两个几何对象并返回`true`如果它们的交点“空间上交叉”；
    /// 也就是说，几何体有一些，但不是所有的内部点是共同的。
    ///
    /// ```
    /// use geo_types::{LineString, line_string, polygon};
    /// use geo::relate::Relate;
    ///
    /// let line_string: LineString = line_string![(x: 85.0, y: 194.0), (x: 162.0, y: 135.0)];
    /// let poly = polygon![
    ///     (x: 125., y: 179.),
    ///     (x: 110., y: 150.),
    ///     (x: 160., y: 160.),
    ///     (x: 125., y: 179.),
    /// ];
    ///
    /// let intersection = line_string.relate(&poly);
    /// assert_eq!(intersection.is_crosses(), true);
    /// ```
    ///
    /// # 注意
    /// - 如果以下任何条件不成立，则函数将返回 false:
    ///     - 几何体的内部交点必须为非空
    ///     - 交点的维度必须小于两个输入几何体的最大维度（两个多边形不能交叉）
    ///     - 两个几何体的交点不能与任一几何体相等（两点不能交叉）
    /// - 匹配一个 `[T*T******] (a < b)`, `[T*****T**] (a > b)`, `[0********] (dimensions == 1)`
    /// - 这个谓词是 **对称和非自反的**
    pub fn is_crosses(&self) -> bool {
        let dims_a = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::Inside][CoordPos::OnBoundary])
            .max(self.0[CoordPos::Inside][CoordPos::Outside]);

        let dims_b = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::OnBoundary][CoordPos::Inside])
            .max(self.0[CoordPos::Outside][CoordPos::Inside]);
        match (dims_a, dims_b) {
            // a < b
            _ if dims_a < dims_b =>
            // [T*T******]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
            }
            // a > b
            _ if dims_a > dims_b =>
            // [T*****T**]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            // a == b, only line / line permitted
            (Dimensions::OneDimensional, Dimensions::OneDimensional) =>
            // [0********]
            {
                self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::ZeroDimensional
            }
            _ => false,
        }
    }

    /// 如果几何体 `a` 和 `b` "空间重叠"，返回 `true`。如果两个几何体具有相同的维度，它们的内部在该维度上相交，并且每个几何体至少有一个点在另一个几何体之内（或者等效地，两个几何体没有覆盖对方）
    ///
    /// ```
    /// use geo_types::{Polygon, polygon};
    /// use geo::relate::Relate;
    ///
    /// let poly1 = polygon![
    ///     (x: 125., y: 179.),
    ///     (x: 110., y: 150.),
    ///     (x: 160., y: 160.),
    ///     (x: 125., y: 179.),
    /// ];
    /// let poly2 = polygon![
    ///     (x: 126., y: 179.),
    ///     (x: 110., y: 150.),
    ///     (x: 161., y: 160.),
    ///     (x: 126., y: 179.),
    /// ];
    ///
    /// let intersection = poly1.relate(&poly2);
    /// assert_eq!(intersection.is_overlaps(), true);
    /// ```
    ///
    /// # 注意
    /// - 匹配一个 `[1*T***T**] (dimensions == 1)`, `[T*T***T**] (dimensions == 0 OR 2)`
    /// - 这个谓词是 **对称的**
    #[allow(clippy::nonminimal_bool)]
    pub fn is_overlaps(&self) -> bool {
        // dimensions 必须为非空，相等，并且线/线是一个特殊情况
        let dims_a = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::Inside][CoordPos::OnBoundary])
            .max(self.0[CoordPos::Inside][CoordPos::Outside]);

        let dims_b = self.0[CoordPos::Inside][CoordPos::Inside]
            .max(self.0[CoordPos::OnBoundary][CoordPos::Inside])
            .max(self.0[CoordPos::Outside][CoordPos::Inside]);
        match (dims_a, dims_b) {
            // 线/线: [1*T***T**]
            (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::OneDimensional
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            // 点/点 或 多边形/多边形: [T*T***T**]
            (Dimensions::ZeroDimensional, Dimensions::ZeroDimensional)
            | (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
                    && self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
                    && self.0[CoordPos::Outside][CoordPos::Inside] != Dimensions::Empty
            }
            _ => false,
        }
    }

    /// 直接访问这个矩阵
    ///
    /// ```
    /// use geo_types::{LineString, Rect, line_string};
    /// use geo::{coordinate_position::CoordPos, dimensions::Dimensions, relate::Relate};
    ///
    /// let line_string: LineString = line_string![(x: 0.0, y: 0.0), (x: 10.0, y: 0.0), (x: 5.0, y: 5.0)];
    /// let rect = Rect::new((0.0, 0.0), (5.0, 5.0));
    ///
    /// let intersection = line_string.relate(&rect);
    ///
    /// // 两个内部的交点是空的，因为字符串的任何部分都不在矩形内
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::Inside), Dimensions::Empty);
    ///
    /// // 线字符串的内部与矩形边界的交点是一维的，因为第一条线的一部分与矩形的边重叠
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::OnBoundary), Dimensions::OneDimensional);
    ///
    /// // 线字符串的内部与矩形外部的交点是一维的，因为字符串的一部分在矩形外
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::Outside), Dimensions::OneDimensional);
    ///
    /// // 线字符串的边界与矩形内部的交点是空的，因为它的两个端点都不在矩形内
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::Inside), Dimensions::Empty);
    ///
    /// // 线字符串的边界与矩形边界的交点是零维的，因为字符串的起点和终点都在矩形的边上
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::OnBoundary), Dimensions::ZeroDimensional);
    ///
    /// // 线字符串的边界与矩形外部的交点是空的，因为它的两个端点都不在矩形外
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::Outside), Dimensions::Empty);
    ///
    /// // 线的外部与矩形内部的交点是二维的，因为它简单地是矩形的内部
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::Inside), Dimensions::TwoDimensional);
    ///
    /// // 线的外部与矩形边界的交点是一维的，因为它是矩形的边(减去字符串覆盖它的地方)
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::OnBoundary), Dimensions::OneDimensional);
    ///
    /// // 两个外部的交点是二维的，因为它是两种形状的整个平面
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::Outside), Dimensions::TwoDimensional);
    /// ```
    pub fn get(&self, lhs: CoordPos, rhs: CoordPos) -> Dimensions {
        self.0[lhs][rhs]
    }

    /// 交点矩阵是否匹配提供的 DE-9IM 规范字符串？
    ///
    /// DE-9IM 规范字符串必须是9个字符长，并且每个字符必须是以下之一:
    ///
    /// - 0: 匹配一个0维（点）交点
    /// - 1: 匹配一个1维（线）交点
    /// - 2: 匹配一个2维（面积）交点
    /// - f 或 F: 仅匹配空维度
    /// - t 或 T: 匹配任何非空的
    /// - *: 匹配任何
    ///
    /// ```
    /// use geo::algorithm::Relate;
    /// use geo::geometry::Polygon;
    /// use wkt::TryFromWkt;
    ///
    /// let a = Polygon::<f64>::try_from_wkt_str("POLYGON((0 0,4 0,4 4,0 4,0 0))").expect("有效的 WKT");
    /// let b = Polygon::<f64>::try_from_wkt_str("POLYGON((1 1,4 0,4 4,0 4,1 1))").expect("有效的 WKT");
    /// let im = a.relate(&b);
    /// assert!(im.matches("212F11FF2").expect("有效的 DE-9IM 规范"));
    /// assert!(im.matches("TTT***FF2").expect("有效的 DE-9IM 规范"));
    /// assert!(!im.matches("TTT***FFF").expect("有效的 DE-9IM 规范"));
    /// ```
    pub fn matches(&self, spec: &str) -> Result<bool, InvalidInputError> {
        if spec.len() != 9 {
            return Err(InvalidInputError::new(format!(
                "DE-9IM 规范必须正好是9个字符。得到 {len}",
                len = spec.len()
            )));
        }

        let mut chars = spec.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                let dim_spec = dimension_matcher::DimensionMatcher::try_from(
                    chars.next().expect("长度已经验证为9"),
                )?;
                if !dim_spec.matches(self.0[*a][*b]) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

/// 根据字符串规范构建一个IntersectionMatrix。
/// ```
/// use geo::algorithm::relate::IntersectionMatrix;
/// use std::str::FromStr;
///
/// let intersection_matrix = IntersectionMatrix::from_str("212101212").expect("有效的 DE-9IM 规范");
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_contains());
/// ```
impl FromStr for IntersectionMatrix {
    type Err = InvalidInputError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut im = IntersectionMatrix::empty();
        im.set_at_least_from_string(str)?;
        Ok(im)
    }
}

pub(crate) mod dimension_matcher {
    use super::Dimensions;
    use super::InvalidInputError;

    /// DE-9IM 匹配规范中的单个字母，如 "1*T**FFF*"
    pub(crate) enum DimensionMatcher {
        Anything,
        NonEmpty,
        Exact(Dimensions),
    }

    impl DimensionMatcher {
        pub fn matches(&self, dim: Dimensions) -> bool {
            match (self, dim) {
                (Self::Anything, _) => true,
                (DimensionMatcher::NonEmpty, d) => d != Dimensions::Empty,
                (DimensionMatcher::Exact(a), b) => a == &b,
            }
        }
    }

    impl TryFrom<char> for DimensionMatcher {
        type Error = InvalidInputError;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                '*' => Self::Anything,
                't' | 'T' => Self::NonEmpty,
                'f' | 'F' => Self::Exact(Dimensions::Empty),
                '0' => Self::Exact(Dimensions::ZeroDimensional),
                '1' => Self::Exact(Dimensions::OneDimensional),
                '2' => Self::Exact(Dimensions::TwoDimensional),
                _ => {
                    return Err(InvalidInputError::new(format!(
                        "无效的 DE-9IM 规范字符：{value}"
                    )))
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Relate;
    use crate::geometry::*;
    use crate::wkt;

    #[test]
    fn test_crosses() {
        // 这些多边形看起来像是交叉的，但两个多边形不能交叉
        let a: Geometry<_> = wkt! { POLYGON ((3.4 15.7, 2.2 11.3, 5.8 11.4, 3.4 15.7)) }.into();
        let b: Geometry<_> = wkt! { POLYGON ((5.2 13.1, 4.5 10.9, 6.3 11.1, 5.2 13.1)) }.into();
        // 这个线串是 b 的一个单独的腿：它可以跨过多边形 a
        let c: Geometry<_> = wkt! { LINESTRING (5.2 13.1, 4.5 10.9) }.into();
        let relate_ab = a.relate(&b);
        let relate_ca = c.relate(&a);
        assert!(!relate_ab.is_crosses());
        assert!(relate_ca.is_crosses());
    }

    #[test]
    fn test_crosses_2() {
        // 两条线可以交叉
        // 与 test_crosses 相同的几何结构：多边形 a 和 b 的单个腿
        let a: Geometry<_> = wkt! { LINESTRING (5.2 13.1, 4.5 10.9) }.into();
        let b: Geometry<_> = wkt! { LINESTRING (3.4 15.7, 2.2 11.3, 5.8 11.4) }.into();
        let relate_ab = a.relate(&b);
        assert!(relate_ab.is_crosses());
    }

    mod test_matches {
        use super::*;

        fn subject() -> IntersectionMatrix {
            // 拓扑上，这是一个无意义的 IM
            IntersectionMatrix::from_str("F00111222").unwrap()
        }

        #[test]
        fn matches_exactly() {
            assert!(subject().matches("F00111222").unwrap());
        }

        #[test]
        fn doesnt_match() {
            assert!(!subject().matches("222222222").unwrap());
        }

        #[test]
        fn matches_truthy() {
            assert!(subject().matches("FTTTTTTTT").unwrap());
        }

        #[test]
        fn matches_wildcard() {
            assert!(subject().matches("F0011122*").unwrap());
        }
    }

    #[test]
    fn empty_is_equal_topo() {
        let empty_polygon = Polygon::<f64>::new(LineString::new(vec![]), vec![]);
        let im = empty_polygon.relate(&empty_polygon);
        assert!(im.is_equal_topo());
    }
}
