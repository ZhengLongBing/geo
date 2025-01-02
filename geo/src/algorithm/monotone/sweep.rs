use crate::sweep::{Active, Event, EventType, LineOrPoint, SweepPoint, VecSet};
use crate::{GeoNum, Orientation};
use std::{collections::BinaryHeap, fmt::Debug};

use super::{RcSegment, Segment};

/// 简单平面扫描算法。
///
/// 对一组线段或点沿X轴执行平面扫描。
/// 这可以通过线段或点的迭代器初始化，以及一个可选的负载。
///
/// 扫描用于：
///
/// - 以词汇顺序迭代输入线段或点的所有终点。
///
/// - 查询在当前迭代点处的活动段集合：这些是当前与扫描线相交的段，并按它们在线上的位置排序。
///
/// # 注意
///
/// 这是一个更简单的版本，不支持两个段内部的交点。
/// 即，每个段之间的交叉点应在其中至少一个的终点处。
/// 特别是，重叠也不支持（会触发恐慌）。
pub(crate) struct SimpleSweep<T: GeoNum, P: Debug> {
    events: BinaryHeap<Event<T, RcSegment<T, P>>>,
    active_segments: VecSet<Active<RcSegment<T, P>>>,
}

impl<T: GeoNum, P: Debug + Clone> SimpleSweep<T, P> {
    pub(crate) fn new<I, D>(iter: I) -> Self
    where
        I: IntoIterator<Item = D>,
        D: Into<Segment<T, P>>,
    {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let mut events = BinaryHeap::with_capacity(size);
        let active_segments = VecSet::default();

        for cr in iter {
            let segment = RcSegment::from(cr.into());
            events.extend(segment.events());
        }

        SimpleSweep {
            events,
            active_segments,
        }
    }

    /// 进展并获取下一个扫描点以及在当前扫描点结束的段集合。
    ///
    /// 段按照其`EventType`的顺序返回；特别是，所有在当前扫描点结束的段优先于在当前扫描点开始的段返回。
    /// 返回第一个在当前扫描点开始的段的索引在`split_idx`参数中。
    pub(crate) fn next_point<F: FnMut(RcSegment<T, P>, EventType)>(
        &mut self,
        mut f: F,
    ) -> Option<SweepPoint<T>> {
        let point = self.peek_point();
        while let Some(pt) = point {
            let ev = self.events.pop().unwrap();
            self.handle_event(ev, &mut |ev| {
                let segment = ev.payload;
                let ty = ev.ty;
                f(segment, ty);
            });
            if self.peek_point() != Some(pt) {
                break;
            }
        }
        point
    }

    fn handle_event<F>(&mut self, event: Event<T, RcSegment<T, P>>, cb: &mut F)
    where
        F: FnMut(Event<T, RcSegment<T, P>>),
    {
        // 我们可能会从调整线段中获得虚假事件。忽略。
        if event.point != event.payload.line().left() && event.point != event.payload.line().right()
        {
            return;
        }

        use EventType::*;
        let segment = &event.payload;
        trace!(
            "处理事件: {pt:?} ({ty:?}) @ {seg:?}",
            pt = event.point,
            ty = event.ty,
            seg = segment,
        );

        match &event.ty {
            LineLeft => {
                let mut idx = self.active_segments.index_not_of(segment);
                for is_next in [false, true] {
                    let (active, split) = if !is_next {
                        if idx > 0 {
                            let active = &self.active_segments[idx - 1];
                            (active, self.check_interior_intersection(active, segment))
                        } else {
                            continue;
                        }
                    } else if idx < self.active_segments.len() {
                        let active = &self.active_segments[idx];
                        (active, self.check_interior_intersection(active, segment))
                    } else {
                        continue;
                    };

                    match split {
                        SplitResult::SplitA(pt) => {
                            let new_seg = active.split_at(pt);
                            let [_, ev] = active.events();
                            self.events.push(ev);
                            self.events.extend(new_seg.events());
                        }
                        SplitResult::SplitB(pt) => {
                            let new_seg = segment.split_at(pt);
                            let [_, ev] = segment.events();
                            self.events.push(ev);
                            self.events.extend(new_seg.events());
                        }
                        SplitResult::None => {}
                    }

                    // 特殊情况：如果我们在当前事件点分割，那么队列中需要在此之前处理 LineRight 事件。

                    // 由于这是一个左事件，总会有一个顶点。
                    while self.events.peek().unwrap() > &event {
                        debug_assert_eq!(self.events.peek().unwrap().ty, LineRight);
                        debug_assert_eq!(self.events.peek().unwrap().point, event.point);

                        let ev = self.events.pop().unwrap();
                        self.handle_event(ev, cb);
                        if !is_next {
                            idx -= 1;
                        }
                    }
                }

                self.active_segments.insert_at(idx, segment.clone());
            }
            LineRight => {
                let idx = self.active_segments.index_of(segment);
                self.active_segments.remove_at(idx);

                if idx > 0 && idx < self.active_segments.len() {
                    let prev = &self.active_segments[idx - 1];
                    let next = &self.active_segments[idx];
                    self.check_interior_intersection(prev, next);
                }
            }
            _ => {}
        }
        cb(event);
    }

    #[inline]
    pub(super) fn peek_point(&self) -> Option<SweepPoint<T>> {
        self.events.peek().map(|e| e.point)
    }

    pub(super) fn prev_active_from_geom(&self, geom: LineOrPoint<T>) -> Option<RcSegment<T, P>> {
        let part_idx = self.active_segments.partition_point(|s| s.line() < geom);
        if part_idx == 0 {
            None
        } else {
            Some(self.active_segments[part_idx - 1].0.clone())
        }
    }

    /// 检查两个段是否在其中一个内部的点相交。
    fn check_interior_intersection(
        &self,
        a: &RcSegment<T, P>,
        b: &RcSegment<T, P>,
    ) -> SplitResult<T> {
        let la = a.line();
        let lb = b.line();

        let lal = la.left();
        let lar = la.right();

        let lbl = lb.left();
        let lbr = lb.right();

        if lal < lbl && lbl < lar && la.orient2d(*lbl) == Orientation::Collinear {
            SplitResult::SplitA(lbl)
        } else if lal < lbr && lbr < lar && la.orient2d(*lbr) == Orientation::Collinear {
            SplitResult::SplitA(lbr)
        } else if lbl < lal && lal < lbr && lb.orient2d(*lal) == Orientation::Collinear {
            SplitResult::SplitB(lal)
        } else if lbl < lar && lar < lbr && lb.orient2d(*lar) == Orientation::Collinear {
            SplitResult::SplitB(lar)
        } else {
            SplitResult::None
        }
    }
}

enum SplitResult<T: GeoNum> {
    SplitA(SweepPoint<T>),
    SplitB(SweepPoint<T>),
    None,
}
