use std::{cmp::Ordering, ops::Deref};

use super::SweepPoint;
use crate::{
    line_intersection::line_intersection, Coord, GeoFloat, GeoNum, Kernel, Line, LineIntersection,
    Orientation,
};

/// 要么是线段，要么是点。
///
/// 坐标是有序的（参见 [`SweepPoint`]），并且线
/// 段必须有不同的点（如果坐标相等，则使用 `Point` 变体）。
#[derive(Clone, Copy)]
pub enum LineOrPoint<T: GeoNum> {
    Point(SweepPoint<T>),
    Line {
        left: SweepPoint<T>,
        right: SweepPoint<T>,
    },
}

impl<T: GeoNum> std::fmt::Debug for LineOrPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineOrPoint::Point(p) => f.debug_tuple("Pt").field(&p.x_y()).finish(),
            LineOrPoint::Line { left, right } => f
                .debug_tuple("LPt")
                .field(&left.x_y())
                .field(&right.x_y())
                .finish(),
        }
    }
}

impl<T: GeoNum> From<SweepPoint<T>> for LineOrPoint<T> {
    fn from(pt: SweepPoint<T>) -> Self {
        Self::Point(pt)
    }
}

impl<T: GeoNum> From<(SweepPoint<T>, SweepPoint<T>)> for LineOrPoint<T> {
    fn from((start, end): (SweepPoint<T>, SweepPoint<T>)) -> Self {
        match start.cmp(&end) {
            Ordering::Less => Self::Line {
                left: start,
                right: end,
            },
            Ordering::Equal => Self::Point(start),
            Ordering::Greater => Self::Line {
                left: end,
                right: start,
            },
        }
    }
}

/// 从 [`Line`] 转换，确保终点顺序。
impl<T: GeoNum> From<Line<T>> for LineOrPoint<T> {
    fn from(l: Line<T>) -> Self {
        let start: SweepPoint<T> = l.start.into();
        let end = l.end.into();
        (start, end).into()
    }
}

/// 从 [`Coord`] 转换
impl<T: GeoNum> From<Coord<T>> for LineOrPoint<T> {
    fn from(c: Coord<T>) -> Self {
        Self::Point(c.into())
    }
}

impl<T: GeoNum> LineOrPoint<T> {
    /// 检查变体是否为线。
    #[inline]
    pub fn is_line(&self) -> bool {
        matches!(self, Self::Line { .. })
    }

    /// 返回自身的 [`Line`] 表示。
    #[inline]
    pub fn line(&self) -> Line<T> {
        match self {
            LineOrPoint::Point(p) => Line::new(**p, **p),
            LineOrPoint::Line { left, right } => Line::new(**left, **right),
        }
    }

    #[inline]
    pub fn left(&self) -> SweepPoint<T> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { left, .. } => *left,
        }
    }

    #[inline]
    pub fn right(&self) -> SweepPoint<T> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { right, .. } => *right,
        }
    }

    #[cfg(test)]
    pub fn coords_equal(&self, other: &LineOrPoint<T>) -> bool {
        self.is_line() == other.is_line() && self.end_points() == other.end_points()
    }

    #[inline]
    pub fn end_points(&self) -> (SweepPoint<T>, SweepPoint<T>) {
        match self {
            LineOrPoint::Point(p) => (*p, *p),
            LineOrPoint::Line { left, right } => (*left, *right),
        }
    }

    pub fn new(left: SweepPoint<T>, right: SweepPoint<T>) -> Self {
        if left == right {
            Self::Point(left)
        } else {
            Self::Line { left, right }
        }
    }

    pub fn orient2d(&self, other: Coord<T>) -> Orientation {
        let (left, right) = match self {
            LineOrPoint::Point(p) => (**p, **p),
            LineOrPoint::Line { left, right } => (**left, **right),
        };
        T::Ker::orient2d(left, right, other)
    }
}

/// 基于算法定义的段的排序进行相等性判断。
impl<T: GeoNum> PartialEq for LineOrPoint<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// 段的排序根据算法定义。
///
/// 需要满足以下条件：
///
/// 1. 如果比较两个线段，则两者的左端必须严格
///    小于两者的右端。
///
/// 2. 一个点被视为一个以其坐标为中心的无穷小垂直线段。
impl<T: GeoNum> PartialOrd for LineOrPoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (LineOrPoint::Point(p), LineOrPoint::Point(o)) => {
                if p == o {
                    Some(Ordering::Equal)
                } else {
                    // 不等的点不满足前提条件，不能排序。
                    None
                }
            }
            (LineOrPoint::Point(_), LineOrPoint::Line { .. }) => {
                other.partial_cmp(self).map(Ordering::reverse)
            }
            (LineOrPoint::Line { left, right }, LineOrPoint::Point(p)) => {
                if p > right || left > p {
                    return None;
                }
                Some(
                    T::Ker::orient2d(**left, **right, **p)
                        .as_ordering()
                        .then(Ordering::Greater),
                )
            }
            (
                LineOrPoint::Line {
                    left: left_a,
                    right: right_a,
                },
                LineOrPoint::Line {
                    left: left_b,
                    right: right_b,
                },
            ) => {
                if left_a > left_b {
                    return other.partial_cmp(self).map(Ordering::reverse);
                }
                if left_a >= right_b || left_b >= right_a {
                    return None;
                }

                // 断言：p1 <= p2
                // 断言：pi < q_j
                Some(
                    T::Ker::orient2d(**left_a, **right_a, **left_b)
                        .as_ordering()
                        .then_with(|| {
                            T::Ker::orient2d(**left_a, **right_a, **right_b).as_ordering()
                        }),
                )
            }
        }
    }
}

impl<T: GeoFloat> LineOrPoint<T> {
    /// 与自身相交并返回一个点、重叠的线段或 `None`。
    ///
    /// `other` 参数必须是线的变体（在调试版本中，否则将导致 panic）。
    pub fn intersect_line(&self, other: &Self) -> Option<Self> {
        debug_assert!(other.is_line(), "尝试与点变体相交！");

        let line = other.line();
        match self {
            LineOrPoint::Point(p) => {
                use crate::Intersects;
                if line.intersects(&**p) {
                    Some(*self)
                } else {
                    None
                }
            }
            LineOrPoint::Line { left, right } => {
                line_intersection(self.line(), line).map(|l| match l {
                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    } => {
                        let mut pt = intersection;
                        if is_proper && (&pt == left.deref()) {
                            if left.x == right.x {
                                pt.y = pt.y.next_after(T::infinity());
                            } else {
                                pt.x = pt.x.next_after(T::infinity());
                            }
                        }
                        pt.into()
                    }
                    LineIntersection::Collinear { intersection } => intersection.into(),
                })
            }
        }
    }

    pub fn intersect_line_ordered(&self, other: &Self) -> Option<Self> {
        let ord = self.partial_cmp(other);
        match self.intersect_line(other) {
            Some(Self::Point(p)) => {
                // 注意：使用非精确数值（f64 等）在这个算法中存在的一个关键问题是
                // 交点可能返回非直观的点。
                //
                // 具体来说，这会导致两个问题：
                //
                // 1. 交点 r 按字典顺序位于终点之间。然而，在有限表示下，线 (1, 1) - (1 + eps, -1)，其中 eps 是 ulp(1)，不
                // 承认 r 位于终点之间。另外，终点可能是实际交点的一个非常糟糕的近似（例如，与 x 轴相交）。
                //
                // 我们检测并强制 r 大于两个终点；另一种情况不容易处理，因为扫描已经进
                // 行到一个严格大于 r 的 p。
                //
                // 2. 更严重的问题是，r 通常可能不完全在直线上。因此，随着段存储在活动段树（B 树 / Splay）中，这
                // 在不利情况下，可能会导致段之间的排序不正确，从而使段无效。这不容易在没有专为此算法构建的侵入数据结
                // 构的情况下进行校正，该数据结构可以跟踪树节点的邻居，并修复或报告此问题。crate `btree-slab`
                // 似乎是一个很好的起点。
                let (mut x, y) = p.x_y();

                let c = self.left();
                if x == c.x && y < c.y {
                    x = x.next_after(T::infinity());
                }

                let p = Coord { x, y }.into();
                debug_assert!(
                    p >= self.left(),
                    "交点在第一条线之前: {p:?}\n\tline({lp1:?} - {lp2:?}) X line({lp3:?} - {lp4:?})",
                    lp1 = self.left(),
                    lp2 = self.right(),
                    lp3 = other.left(),
                    lp4 = other.right(),
                );
                debug_assert!(
                    p >= other.left(),
                    "交点在第二条线之前: {p:?}\n\tline({lp1:?} - {lp2:?}) X line({lp3:?} - {lp4:?})",
                    lp1 = self.left(),
                    lp2 = self.right(),
                    lp3 = other.left(),
                    lp4 = other.right(),
                );

                if let Some(ord) = ord {
                    let l1 = LineOrPoint::from((self.left(), p));
                    let l2 = LineOrPoint::from((other.left(), p));
                    let cmp = l1.partial_cmp(&l2).unwrap();
                    if l1.is_line() && l2.is_line() && cmp.then(ord) != ord {
                        debug!(
                            "由交点改变的排序：{l1:?} {ord:?} {l2:?}",
                            l1 = self,
                            l2 = other
                        );
                        debug!("\t部分: {l1:?}, {l2:?}");
                        debug!("\t交点: {p:?} {cmp:?}");

                        // RM: 这是一个改变排序的复杂交点。
                        // 启发式：用一个简单的交点近似以保持拓扑结构。
                        return Some(if self.left() > other.left() {
                            self.left().into()
                        } else {
                            other.left().into()
                        });
                    }
                }
                Some(Self::Point(p))
            }
            e => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use geo_types::{Coord, LineString};
    use wkt::ToWkt;

    use crate::{GeoFloat, GeoNum, Kernel};

    use super::LineOrPoint;

    // 用于调试扫描浮点问题
    #[test]
    #[ignore]
    fn check_ordering() {
        let pt_7 = Coord::from((-32.57812499999999, 241.33427773853316));
        let pt_8 = Coord::from((-36.11348070978957, 237.7989220287436));
        let pt_13 = Coord::from((-25.507080078124993, 248.40532266040816));
        let pt_14 = Coord::from((-36.48784219165816, 237.424560546875));
        let _pt_15 = Coord::from((4.4929199218750036, 196.44379843334184));
        let pt_16 = Coord::from((-36.048578439260666, 237.8638242992725));
        let pt_17 = Coord::from((3.545624214480127, 197.39109414073673));

        fn check_isection<T: GeoFloat>(abcd: [Coord<T>; 4]) -> Option<LineOrPoint<T>> {
            let l1 = LineOrPoint::from((abcd[0].into(), abcd[1].into()));
            let l2 = LineOrPoint::from((abcd[2].into(), abcd[3].into()));
            l1.intersect_line_ordered(&l2)
        }
        fn check_lines<T: GeoNum>(abcd: [Coord<T>; 4]) -> Ordering {
            let l1 = LineOrPoint::from((abcd[0].into(), abcd[1].into()));
            let l2 = LineOrPoint::from((abcd[2].into(), abcd[3].into()));
            l1.partial_cmp(&l2).unwrap()
        }

        eprintln!(
            "(14-17) {cmp:?} (14-16)",
            cmp = check_lines([pt_14, pt_17, pt_14, pt_16])
        );
        eprintln!(
            "(8-16) {cmp:?} (14-16)",
            cmp = check_lines([pt_8, pt_16, pt_14, pt_16]),
        );
        eprintln!(
            "(8-7) {cmp:?} (14-16)",
            cmp = check_lines([pt_8, pt_7, pt_14, pt_16]),
        );
        eprintln!(
            "(8-7) {cmp:?} (14-13)",
            cmp = check_lines([pt_8, pt_7, pt_14, pt_13]),
        );
        eprintln!(
            "(8-7) {isect:?} (14-13)",
            isect = check_isection([pt_8, pt_7, pt_14, pt_13]),
        );
        let l87 = LineString::new(vec![pt_8, pt_16, pt_7]);
        let lo = LineString::new(vec![pt_14, pt_16, pt_13]);
        eprintln!("l1: {}", l87.to_wkt());
        eprintln!("lo: {}", lo.to_wkt());

        eprintln!(
            "pred: {:?}",
            <f64 as GeoNum>::Ker::orient2d(pt_8, pt_7, pt_17)
        );
        eprintln!(
            "pred: {:?}",
            <f64 as GeoNum>::Ker::orient2d(pt_8, pt_14, pt_16)
        );
    }
}
