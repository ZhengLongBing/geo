use super::*;
use crate::GeoFloat;
use std::{cmp::Ordering, collections::BinaryHeap};
pub(crate) struct Sweep<C: Cross> {
    is_simple: bool,
    events: BinaryHeap<Event<C::Scalar, IMSegment<C>>>,
    active_segments: VecSet<Active<IMSegment<C>>>,
}
impl<C: Cross + Clone> Sweep<C> {
    pub(crate) fn new<I>(iter: I, is_simple: bool) -> Self
    where
        I: IntoIterator<Item = C>,
    {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let mut sweep = Sweep {
            events: BinaryHeap::with_capacity(size),
            active_segments: Default::default(),
            is_simple,
        };
        for cr in iter {
            IMSegment::create_segment(cr, None, None, |ev| sweep.events.push(ev));
        }
        sweep
    }

    /// 处理堆中的下一个事件。
    ///
    /// 除非事件是多余的，否则调用回调函数。
    #[inline]
    pub(super) fn next_event<F>(&mut self, mut cb: F) -> Option<SweepPoint<C::Scalar>>
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        self.events.pop().map(|event| {
            let pt = event.point;
            self.handle_event(event, &mut cb);
            pt
        })
    }

    /// 处理两个相邻的段。
    ///
    /// 第一个参数必须是活动段，另一个可以是也可以不是。
    /// 重叠从活动链接到其他。
    fn process_adjacent_segments(
        &mut self,
        active: Active<IMSegment<C>>,
        other: &IMSegment<C>,
    ) -> AdjProcOutput<C::Scalar> {
        // 注意：以下逻辑是一个循环，而不是一个
        // 条件判断，这归因于浮点问题。具体来说，
        // 有时两个不重叠的线段可能在交点处被分割后变得重叠！

        // 示例：
        // let pt_7 = Coord::from((-32.57812499999999, 241.33427773853316));
        // let pt_8 = Coord::from((-36.11348070978957, 237.7989220287436));
        // let pt_13 = Coord::from((-25.507080078124993, 248.40532266040816));
        // let pt_14 = Coord::from((-36.48784219165816, 237.424560546875));
        // let pt_16 = Coord::from((-36.048578439260666, 237.8638242992725));
        // 7-8 和 13-14 在 16 相交，因此 8-16 和 14-16 重叠！

        // 如果段在调整后重叠，我们通过进行两次相交来处理这一点。
        let mut out = AdjProcOutput {
            isec: None,
            should_continue: true,
            should_callback: false,
        };
        while let Some(isec) = other.geom().intersect_line_ordered(&active.geom()) {
            trace!(
                "找到相交（LL）：\n\tsegment1: {:?}\n\tsegment2: {:?}\n\tintersection: {:?}",
                other,
                active,
                isec
            );
            out.isec = Some(isec);

            // 1. 分割 adj_segment，并将额外的分割存储
            let adj_overlap = active.adjust_one_segment(isec, |e| self.events.push(e));

            // 2. 分割 segment，根据需要添加额外的段。
            let seg_overlap = other.adjust_one_segment(isec, |e| self.events.push(e));

            assert_eq!(
                adj_overlap.is_some(),
                seg_overlap.is_some(),
                "相交的段之一有重叠，但另一个没有！"
            );
            if let Some(adj_ovl) = adj_overlap {
                let tgt = seg_overlap.unwrap();
                trace!("设置重叠：{adj_ovl:?} -> {tgt:?}");
                adj_ovl.chain_overlap(tgt.clone());

                if &tgt == other {
                    // 整个事件段现在与另外一个活动段重叠。
                    //
                    // 我们不需要继续迭代，但如果
                    // 现在的父左事件已经处理完毕，就应该进行回调。
                    out.should_callback = adj_ovl.is_left_event_done();
                    out.should_continue = false;
                }

                // 重叠是精确计算的，所以我们不需要
                // 重新运行循环。
                return out;
            }

            if active.geom().partial_cmp(&other.geom()) == Some(Ordering::Equal) {
                continue;
            } else {
                break;
            }
        }
        out
    }

    fn handle_event<F>(&mut self, event: Event<C::Scalar, IMSegment<C>>, cb: &mut F) -> bool
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        use EventType::*;
        let segment = match IMSegment::is_correct(&event) {
            false => return false,
            _ => event.payload,
        };
        trace!(
            "处理事件：{pt:?} ({ty:?}) @ {seg:?}",
            pt = event.point,
            ty = event.ty,
            seg = segment,
        );

        // let prev = self.active_segments.previous(&segment).cloned();
        // let next = self.active_segments.next(&segment).cloned();

        match &event.ty {
            LineLeft => {
                let mut should_add = true;
                let mut insert_idx = self.active_segments.index_not_of(&segment);
                if !self.is_simple {
                    for is_next in [true, false].into_iter() {
                        let active = if is_next {
                            if insert_idx < self.active_segments.len() {
                                self.active_segments[insert_idx].clone()
                            } else {
                                continue;
                            }
                        } else if insert_idx > 0 {
                            self.active_segments[insert_idx - 1].clone()
                        } else {
                            continue;
                        };
                        let AdjProcOutput {
                            isec,
                            should_continue,
                            should_callback,
                        } = self.process_adjacent_segments(active.clone(), &segment);
                        let isec = match isec {
                            Some(isec) => isec,
                            None => continue,
                        };
                        // 一种特殊情况是如果 adj_segment 被分割，而交点在该段的起点。
                        // 在这种情况下，堆中会有一个右端事件，在最终完成此事件之前需要处理。
                        let handle_end_event = {
                            // 获取第一个交点
                            let int_pt = isec.left();
                            // 检查它不是调整的起点，而是当前段的起点
                            int_pt != active.geom().left() && int_pt == segment.geom().left()
                        };
                        if handle_end_event {
                            let event = self.events.pop().unwrap();
                            let done = self.handle_event(event, cb);
                            debug_assert!(done, "特别右端事件处理失败");
                            if !is_next {
                                // 之前的段现在被删除
                                insert_idx -= 1;
                            }
                        }

                        if !should_continue {
                            should_add = false;
                            if !should_callback {
                                return true;
                            }
                            break;
                        }

                        // let n = self.active_segments.len();
                        // if is_next && 1 + insert_idx < n {
                        //     (insert_idx..n).find(|&idx| !self.active_segments.check_swap(idx));
                        // } else if !is_next && insert_idx > 1 {
                        //     (0..insert_idx - 2)
                        //         .rev()
                        //         .find(|&idx| !self.active_segments.check_swap(idx));
                        // }
                    }
                }

                if should_add {
                    // 添加当前段为活动段
                    // 安全性：`self.segments` 是一个 `Box`，直到 `self` 被删除前不会被释放。
                    debug!("插入活动：{segment:?}");

                    // 注意：我们勇敢地跟踪当活动列表被调整时的 insert_idx
                    // self.active_segments.insert_active(segment.clone());
                    self.active_segments.insert_at(insert_idx, segment.clone());
                }

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty); // 在这里调用回调函数
                    seg.set_left_event_done(); // 标记左事件为已完成
                    cb_seg = seg.overlap(); // 获取重叠的段（如果存在）
                }
            }
            LineRight => {
                // 安全性：`self.segments` 是一个 `Box`，直到 `self` 被删除前不会被释放。
                debug!("移除活动：{segment:?}");
                let el_idx = self.active_segments.index_of(&segment);
                let prev = (el_idx > 0).then(|| self.active_segments[el_idx - 1].clone());
                let next = (1 + el_idx < self.active_segments.len())
                    .then(|| self.active_segments[el_idx + 1].clone());
                assert_eq!(self.active_segments.remove_at(el_idx), segment);

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty); // 在这里调用回调函数
                    cb_seg = seg.overlap(); // 获取重叠的段（如果存在）
                }

                if !self.is_simple {
                    if let (Some(prev), Some(next)) = (prev, next) {
                        let prev_geom = prev.geom();
                        let next_geom = next.geom();
                        if let Some(adj_intersection) = prev_geom.intersect_line_ordered(&next_geom)
                        {
                            // 1. 分割 prev_segment，并将额外的分割存储
                            let first = prev
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            let second = next
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            debug_assert!(first && second, "在移除时相邻段不能重叠！");
                        }
                    }
                }
            }
            PointLeft => {
                if !self.is_simple {
                    let insert_idx = self.active_segments.index_not_of(&segment);
                    let prev =
                        (insert_idx > 0).then(|| self.active_segments[insert_idx - 1].clone());
                    let next = (insert_idx < self.active_segments.len())
                        .then(|| self.active_segments[insert_idx].clone());

                    for adj_segment in prev.into_iter().chain(next.into_iter()) {
                        let geom = adj_segment.geom();
                        if let Some(adj_intersection) = segment.geom().intersect_line_ordered(&geom)
                        {
                            trace!("找到相交（PL）：\n\tsegment1: {:?}\n\tsegment2: {:?}\n\tintersection: {:?}", segment, adj_segment, adj_intersection);
                            // 1. 分割 adj_segment，并将额外的分割存储
                            let adj_overlap = adj_segment
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e));

                            // 无法出现与点的重叠
                            debug_assert!(adj_overlap.is_none());
                        }
                    }
                }

                // 点无需成为活动段。
                // 将点段发送至回调。
                cb(&segment, event.ty); // 在这里调用回调函数
            }
            PointRight => {
                // 什么都不做。我们可以在对逻辑有信心后移除此变体。
            }
        }
        true
    }

    #[inline]
    pub fn peek_point(&self) -> Option<SweepPoint<C::Scalar>> {
        self.events.peek().map(|e| e.point)
    }
}

/// 用于从 `process_adjacent_segments` 传递结果的内部枚举
struct AdjProcOutput<T: GeoFloat> {
    isec: Option<LineOrPoint<T>>,
    should_continue: bool,
    should_callback: bool,
}
