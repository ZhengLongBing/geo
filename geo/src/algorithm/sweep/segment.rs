use super::*;
use crate::GeoFloat;
use std::{cmp::Ordering, fmt::Debug};

/// 扫描过程中生成的输入 [`LineOrPoint`] 的一段。
#[derive(Clone)]
pub(super) struct Segment<C: Cross> {
    pub(super) geom: LineOrPoint<C::Scalar>,
    pub(super) cross: C,
    pub(super) first_segment: bool,
    pub(super) left_event_done: bool,
    pub(super) overlapping: Option<IMSegment<C>>,
    pub(super) is_overlapping: bool,
}

impl<C: Cross> Segment<C> {
    pub fn new(cross: C, geom: Option<LineOrPoint<C::Scalar>>) -> Self {
        let first_segment = geom.is_none();
        let geom = geom.unwrap_or_else(|| cross.line());
        Self {
            geom,
            cross,
            first_segment,
            left_event_done: false,
            overlapping: None,
            is_overlapping: false,
        }
    }

    /// 在交点处将线段分割成多个部分。
    ///
    /// 初始段就地变动，如果有其他段，则返回。假设精确算术，
    /// self 的顺序在活动段中应保持不变。然而，在有限精度下，
    /// 可能情况并非如此。
    pub fn adjust_for_intersection(
        &mut self,
        intersection: LineOrPoint<C::Scalar>,
    ) -> SplitSegments<C::Scalar> {
        use SplitSegments::*;

        // 我们仅支持拆分线段。
        debug_assert!(self.geom.is_line());
        let (p, q) = self.geom.end_points();

        if !intersection.is_line() {
            // 处理点交点
            let r = intersection.left();
            debug_assert!(p <= r, "交点未在线上排序: {p:?} <= {r:?} <=> {q:?}",);
            if p == r || q == r {
                // 如果交点在端点处，则无需分割此段。
                Unchanged { overlap: false }
            } else {
                // 否则，将其拆分。将 `self` 变为第一部分，并返回第二部分。
                self.geom = (p, r).into();
                // self.first_segment = false;
                SplitOnce {
                    overlap: None,
                    right: (r, q).into(),
                }
            }
        } else {
            let (r1, r2) = intersection.end_points();
            debug_assert!(p <= r1 && r2 <= q, "重叠段未在行中排序！");
            if p == r1 {
                if r2 == q {
                    // 整个段重叠。
                    Unchanged { overlap: true }
                } else {
                    self.geom = (p, r2).into();
                    // self.first_segment = false;
                    SplitOnce {
                        overlap: Some(false),
                        right: (r2, q).into(),
                    }
                }
            } else if r2 == q {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitOnce {
                    overlap: Some(true),
                    right: (r1, q).into(),
                }
            } else {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitTwice {
                    right: (r2, q).into(),
                }
            }
        }
    }
}

/// 更简洁的调试实现。
impl<C: Cross> Debug for Segment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment{{ {geom:?}\n\tof {c:?}\n\t{first} [{has}/{ovl}] }}",
            c = self.cross,
            geom = self.geom,
            first = if self.first_segment { "[1st]" } else { "" },
            has = if self.overlapping.is_some() {
                "HAS"
            } else {
                "NON"
            },
            ovl = if self.is_overlapping { "OVL" } else { "NON" },
        )
    }
}

/// 基于键的部分相等性。
///
/// 这与 `PartialOrd` 的实现一致。
impl<C: Cross> PartialEq for Segment<C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// 按算法定义的部分排序。
///
/// 这要求与 [`LineOrPoint`] 相同的先决条件。
impl<C: Cross> PartialOrd for Segment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.geom.partial_cmp(&other.geom)
    }
}

/// 存储调整段以进行交叉的拆分类型和额外几何。
#[derive(Debug)]
pub(super) enum SplitSegments<T: GeoFloat> {
    Unchanged {
        overlap: bool,
    },
    SplitOnce {
        overlap: Option<bool>,
        right: LineOrPoint<T>,
    },
    SplitTwice {
        right: LineOrPoint<T>,
    },
}

#[cfg(test)]
mod tests {

    use super::*;

    impl<T: GeoFloat> PartialEq for SplitSegments<T> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    Self::Unchanged { overlap: l_overlap },
                    Self::Unchanged { overlap: r_overlap },
                ) => l_overlap == r_overlap,
                (
                    Self::SplitOnce {
                        overlap: l_overlap,
                        right: l_right,
                    },
                    Self::SplitOnce {
                        overlap: r_overlap,
                        right: r_right,
                    },
                ) => l_overlap == r_overlap && l_right.coords_equal(r_right),
                (Self::SplitTwice { right: l_right }, Self::SplitTwice { right: r_right }) => {
                    l_right.coords_equal(r_right)
                }
                _ => false,
            }
        }
    }

    #[test]
    fn test_split() {
        let lines: Vec<_> = vec![
            LineOrPoint::from(((0., 0.).into(), (10., 10.).into())),
            ((10.0, 0.).into(), (0., 10.).into()).into(),
            ((0., 0.).into(), (0., 10.).into()).into(),
            ((0., 0.).into(), (5., 5.).into()).into(),
            ((10., 10.).into(), (5., 5.).into()).into(),
        ]
        .into_iter()
        .map(|lp| Segment::new(lp, None))
        .collect();

        struct TestCase {
            a: usize,
            b: usize,
            isec: Option<LineOrPoint<f64>>,
            split: Option<SplitSegments<f64>>,
        }

        impl TestCase {
            fn assert_equality(&self, lines: &[Segment<LineOrPoint<f64>>]) {
                let isec = lines[self.a]
                    .geom
                    .intersect_line_ordered(&lines[self.b].geom);
                assert_eq!(isec, self.isec);

                if isec.is_none() {
                    return;
                }
                let isec = isec.unwrap();
                let mut copy_seg = lines[self.a].clone();
                let split = copy_seg.adjust_for_intersection(isec);
                assert_eq!(&split, self.split.as_ref().unwrap(),)
            }
        }

        let test_cases = vec![
            TestCase {
                a: 0,
                b: 0,
                isec: Some(lines[0].geom),
                split: Some(SplitSegments::Unchanged { overlap: true }),
            },
            TestCase {
                a: 0,
                b: 1,
                isec: Some(LineOrPoint::from(SweepPoint::from((5., 5.)))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: None,
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
            TestCase {
                a: 0,
                b: 2,
                isec: Some(LineOrPoint::from(SweepPoint::from((0., 0.)))),
                split: Some(SplitSegments::Unchanged { overlap: false }),
            },
            TestCase {
                a: 0,
                b: 3,
                isec: Some(LineOrPoint::from(((0., 0.).into(), (5., 5.).into()))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: Some(false),
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
            TestCase {
                a: 0,
                b: 4,
                isec: Some(LineOrPoint::from(((5., 5.).into(), (10., 10.).into()))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: Some(true),
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
        ];

        test_cases.iter().for_each(|t| t.assert_equality(&lines));
    }
}
