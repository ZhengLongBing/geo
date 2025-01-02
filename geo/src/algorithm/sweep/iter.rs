use super::*;
use crate::{line_intersection::line_intersection, Coord, LineIntersection};

/// 一个输入 [`Cross`] 类型的段。
///
/// 该类型用于传达在给定交点处相交的输入几何部分。
/// 由 [`CrossingsIter::intersections`] 方法返回。
#[derive(Debug, Clone)]
pub(crate) struct Crossing<C: Cross> {
    /// 与此段相关的输入。
    pub cross: C,

    #[allow(unused)] // 忽略编译器对未使用字段的警告
    /// 此段的几何。
    ///
    /// 这是输入的 `crossable` 几何的一部分，
    /// 并且要么在上一次由 [`CrossingsIter`] 产生的交点处开始
    /// 要么在此处结束。
    /// 如果它在此点处结束（`at_left` 为 `false`），
    /// 则可以保证它的内部不包含其他交点。
    pub line: LineOrPoint<C::Scalar>,

    /// 此段是否为输入线的第一个段。
    pub first_segment: bool,

    /// 如果序列中的下一个几何与此段重叠
    /// （即在一个以上的点相交），则标记为 `true`。
    /// 如果是点则不相关且为 `false`。
    ///
    /// 请注意，重叠段可能不会总是
    /// _全部_ 批量处理在一起。
    /// 它们可能会以下一个或多个重叠段集合的任意顺序报告。
    pub has_overlap: bool,

    /// 如果 `geom` 在交点处开始则为 `true`，
    /// 否则在交点处结束。
    pub at_left: bool,

    #[allow(unused)] // 忽略编译器对未使用字段的警告
    pub(super) segment: IMSegment<C>,
}

impl<C: Cross + Clone> Crossing<C> {
    /// 将 `self` 转换为用于用户返回的 `Crossing`。
    pub(super) fn from_segment(segment: &IMSegment<C>, event_ty: EventType) -> Crossing<C> {
        Crossing {
            cross: segment.cross_cloned(),
            line: segment.geom(),
            first_segment: segment.is_first_segment(),
            has_overlap: segment.is_overlapping(),
            at_left: event_ty == EventType::LineLeft,
            segment: segment.clone(),
        }
    }
}

/// 迭代器，用于产生所有交汇点。
///
/// 产生一组线段和点的所有端点、交点和重叠。
/// 通过对 [`Cross`] 的迭代器执行 `collect` 构建。
/// 实现使用 [Bentley-Ottman] 算法，
/// 并在时间 O((n + k) log n) 下运行；
pub(crate) struct CrossingsIter<C>
where
    C: Cross + Clone,
{
    sweep: Sweep<C>,
    segments: Vec<Crossing<C>>,
}

impl<C> CrossingsIter<C>
where
    C: Cross + Clone,
{
    /// 返回与迭代器上次产生的点相交的段。
    pub fn intersections_mut(&mut self) -> &mut [Crossing<C>] {
        &mut self.segments
    }

    pub fn intersections(&self) -> &[Crossing<C>] {
        &self.segments
    }

    fn new_ex<T: IntoIterator<Item = C>>(iter: T, is_simple: bool) -> Self {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let sweep = Sweep::new(iter, is_simple);
        let segments = Vec::with_capacity(4 * size);
        Self { sweep, segments }
    }
}

impl<C> FromIterator<C> for CrossingsIter<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::new_ex(iter, false)
    }
}

impl<C> Iterator for CrossingsIter<C>
where
    C: Cross + Clone,
{
    type Item = Coord<C::Scalar>;

    fn next(&mut self) -> Option<Self::Item> {
        let segments = &mut self.segments;

        segments.clear();
        let mut last_point = self.sweep.peek_point();
        debug!("pt: {last_point:?}");
        while last_point == self.sweep.peek_point() && self.sweep.peek_point().is_some() {
            last_point = self.sweep.next_event(|seg, ty| {
                trace!(
                    "cb: {seg:?} {ty:?} (crossable = {cross:?})",
                    cross = seg.cross_cloned().line()
                );
                segments.push(Crossing::from_segment(seg, ty))
            });
        }

        if segments.is_empty() {
            None
        } else {
            last_point.map(|p| *p)
        }
    }
}

/// 迭代一个线集合的所有交点。
///
/// 为每一对相交或重叠的输入跨越对象产生元组 `(C, C, LineIntersection)`。
/// 这是对集合中所有对计算 [`LineIntersection`] 的直接替代，但通常更有效。
/// 实现使用 [Bentley-Ottman] 算法，
/// 并在时间 O((n + k) log n) 下运行；
pub struct Intersections<C: Cross + Clone> {
    inner: CrossingsIter<C>,
    idx: usize,
    jdx: usize,
    is_overlap: bool,
    pt: Option<Coord<C::Scalar>>,
}

impl<C> FromIterator<C> for Intersections<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self {
            inner: FromIterator::from_iter(iter),
            idx: 0,
            jdx: 0,
            is_overlap: false,
            pt: None,
        }
    }
}

impl<C> Intersections<C>
where
    C: Cross + Clone,
{
    fn intersection(&mut self) -> Option<(C, C, LineIntersection<C::Scalar>)> {
        let (si, sj) = {
            let segments = self.inner.intersections();
            (&segments[self.idx], &segments[self.jdx])
        };
        debug!(
            "comparing intersection: [{iso}]",
            iso = if self.is_overlap { "OVL" } else { "" }
        );
        for i in [si, sj] {
            debug!(
                "\t{geom:?} ({at_left}) [{ovl}] [{first}]",
                geom = i.cross.line(),
                first = if i.first_segment { "FIRST" } else { "" },
                at_left = if i.at_left { "S" } else { "E" },
                ovl = if i.has_overlap { "OVL" } else { "" },
            );
        }
        // 忽略已处理的交点
        let should_compute = if self.is_overlap {
            // 对于重叠，我们只在两个段都是第一个段，并且都是左侧时才返回交点。
            debug_assert_eq!(si.at_left, sj.at_left);
            si.at_left && (si.first_segment && sj.first_segment)
        } else {
            (!si.at_left || si.first_segment) && (!sj.at_left || sj.first_segment)
        };

        if should_compute {
            let si = si.cross.clone();
            let sj = sj.cross.clone();

            let int = line_intersection(si.line().line(), sj.line().line())
                .expect("line_intersection 返回 `None`，与 `CrossingsIter` 不一致");

            Some((si, sj, int))
        } else {
            None
        }
    }

    fn step(&mut self) -> bool {
        let seg_len = self.inner.intersections_mut().len();
        if 1 + self.jdx < seg_len {
            self.is_overlap =
                self.is_overlap && self.inner.intersections_mut()[self.jdx].has_overlap;
            self.jdx += 1;
        } else {
            self.idx += 1;
            if 1 + self.idx >= seg_len {
                loop {
                    self.pt = self.inner.next();
                    if self.pt.is_none() {
                        return false;
                    }
                    if self.inner.intersections_mut().len() > 1 {
                        break;
                    }
                }
                self.idx = 0;
            }
            self.is_overlap = self.inner.intersections_mut()[self.idx].has_overlap;
            self.jdx = self.idx + 1;
        }
        true
    }
}

impl<C> Iterator for Intersections<C>
where
    C: Cross + Clone,
{
    type Item = (C, C, LineIntersection<C::Scalar>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.step() {
                return None;
            }
            let it = self.intersection();
            debug!("\t{it:?}", it = it.is_some());
            if let Some(result) = it {
                return Some(result);
            }
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use crate::Line;
    use log::info;
    use pretty_env_logger::env_logger;
    use std::{io::Write, rc::Rc};

    use super::*;

    pub(super) fn init_log() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
            .try_init();
    }

    #[test]
    fn simple_iter() {
        let input = vec![
            Rc::from(Line::from([(1., 0.), (0., 1.)])),
            Line::from([(0., 0.), (1., 1.)]).into(),
        ];
        let iter: CrossingsIter<_> = input.into_iter().collect();
        assert_eq!(iter.count(), 5);
    }

    #[test]
    fn overlap_intersect() {
        init_log();

        let input = [
            Line::from([(0., 0.), (1., 1.)]),
            [(1., 0.), (0., 1.)].into(),
            [(0., 0.5), (1., 0.5)].into(),
            [(-1., 0.5), (0.5, 0.5)].into(),
            [(0.5, 0.5), (0.5, 0.5)].into(),
            [(0., 0.), (0., 0.)].into(),
        ];
        // 交点（按索引）：
        // (0, 1), (0, 2), (0, 3), (0, 4), (0, 5),
        // (1, 2), (1, 3), (1, 4),
        // (2, 3)
        let mut verify = 0;
        for (i, l1) in input.iter().enumerate() {
            for (j, l2) in input.iter().enumerate() {
                if j <= i {
                    continue;
                }
                if line_intersection(*l1, *l2).is_some() {
                    let lp_a = LineOrPoint::from(*l1);
                    let lp_b = LineOrPoint::from(*l2);
                    eprintln!("{lp_a:?} intersects {lp_b:?}",);
                    verify += 1;
                }
            }
        }

        let iter: Intersections<_> = input.iter().collect();
        let count = iter
            .inspect(|(a, b, _int)| {
                let lp_a = LineOrPoint::from(**a);
                let lp_b = LineOrPoint::from(**b);
                eprintln!("{lp_a:?} intersects {lp_b:?}",);
            })
            .count();
        assert_eq!(count, verify);
    }

    #[test]
    #[ignore]
    fn check_adhoc_crossings() {
        init_log();

        let input = vec![
            Line::from([(0., 0.), (1., 1.)]),
            [(1., 0.), (0., 1.)].into(),
            [(0., 0.5), (1., 0.5)].into(),
            [(-1., 0.5), (0.5, 0.5)].into(),
            [(0.5, 0.5), (0.5, 0.5)].into(),
            [(0., 0.), (0., 0.)].into(),
        ];

        let mut iter: CrossingsIter<_> = input.into_iter().collect();
        while let Some(pt) = iter.next() {
            info!("pt: {pt:?}");
            iter.intersections().iter().for_each(|i| {
                info!(
                    "\t{geom:?} ({at_left}) {ovl}",
                    geom = i.line,
                    at_left = if i.at_left { "S" } else { "E" },
                    ovl = if i.has_overlap { "[OVL] " } else { "" },
                );
            });
        }
    }
}
