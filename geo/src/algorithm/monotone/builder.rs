//! 多边形单调细分算法
//!
//! 该实现基于David Mount的这些优秀[讲座笔记]。其大致思想是对多边形的段进行左右平面扫描，并尝试迭代地扩展平行单调链。
//!
//! [讲座笔记]:
//! //www.cs.umd.edu/class/spring2020/cmsc754/Lects/lect05-triangulate.pdf

use super::{MonoPoly, SimpleSweep};
use crate::{
    sweep::{EventType, LineOrPoint, SweepPoint},
    *,
};
use std::{cell::Cell, mem::replace};

/// 构建沿X轴的多边形迭代器的单调细分。
///
/// 返回构成细分的单调多边形集合。输入多边形必须是有效的`MultiPolygon`（参见[`MultiPolygon`]的有效性部分）。特别地，每个多边形必须简单、不可自交、只包含有限的坐标；此外，多边形必须有明显不同的内部，其边界只能在有限点相交。
pub fn monotone_subdivision<T: GeoNum, I: IntoIterator<Item = Polygon<T>>>(
    iter: I,
) -> Vec<MonoPoly<T>> {
    Builder::from_polygons_iter(iter).build()
}

pub(super) struct Builder<T: GeoNum> {
    sweep: SimpleSweep<T, Info>,
    chains: Vec<Option<Chain<T>>>,
    outputs: Vec<MonoPoly<T>>,
}

impl<T: GeoNum> Builder<T> {
    /// 从多边形创建一个新的构建器。
    pub fn from_polygons_iter<I: IntoIterator<Item = Polygon<T>>>(iter: I) -> Self {
        let iter = iter.into_iter().flat_map(|polygon| {
            let (ext, ints) = polygon.into_inner();
            Some(ext)
                .into_iter()
                .chain(ints)
                .flat_map(|ls| -> Vec<_> { ls.lines().collect() })
                .filter_map(|line| {
                    if line.start == line.end {
                        None
                    } else {
                        let line = LineOrPoint::from(line);
                        debug!("添加线 {:?}", line);
                        Some((line, Default::default()))
                    }
                })
        });
        Self {
            sweep: SimpleSweep::new(iter),
            chains: Vec::new(),
            outputs: Vec::new(),
        }
    }
    pub fn build(mut self) -> Vec<MonoPoly<T>> {
        while self.process_next_pt() {}
        self.outputs
    }

    fn process_next_pt(&mut self) -> bool {
        // 步骤1：获取下一个点处的所有进出段，并按扫描顺序对它们进行排序。
        let mut incoming = vec![];
        let mut outgoing = vec![];

        let pt = if let Some(pt) = self.sweep.next_point(|seg, ev| match ev {
            EventType::LineRight => {
                let rt = seg.line().right();
                incoming.push(seg);
                let chain_idx = incoming.last().unwrap().payload().chain_idx.get();
                self.chains[chain_idx].as_mut().unwrap().fix_top(*rt);
            }
            EventType::LineLeft => {
                outgoing.push(seg);
            }
            _ => unreachable!("意外的事件类型"),
        }) {
            pt
        } else {
            return false;
        };
        incoming.sort_by(|a, b| a.partial_cmp(b).unwrap());
        outgoing.sort_by(|a, b| a.partial_cmp(b).unwrap());

        info!(
            "\n\n处理点 {:?}, #入={}, #出={}",
            pt,
            incoming.len(),
            outgoing.len()
        );

        // 步骤2：计算点下方的区域，并检查是否有任何先前的点注册了帮助。
        let bot_segment = self.sweep.prev_active_from_geom(pt.into());
        let (bot_region, bot_help) = bot_segment
            .as_ref()
            .map(|seg| (seg.payload().next_is_inside.get(), seg.payload().help.get()))
            .unwrap_or((false, None));
        debug!("下方区域: {:?}", bot_region);
        debug!("下方段: {:?}", bot_segment.as_ref().map(|s| s.line()));

        // 步骤3：简化进入段。任何包围输入区域的两段连续进入段现在应该完成一个单调多边形；因此我们“完成”它们的链。我们应该至多剩下两段进入段。
        if !incoming.is_empty() {
            let n = incoming.len();

            #[allow(clippy::bool_to_int_with_if)]
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            debug!("简化进入段: {n} -> {start_idx}..{ub_idx}");

            let mut iter = incoming.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let fc = self.chains[first.payload().chain_idx.get()].take().unwrap();
                let sc = self.chains[second.payload().chain_idx.get()]
                    .take()
                    .unwrap();

                // 考虑在第一个段注册的任何帮助。
                if let Some(help) = first.payload().help.get() {
                    first.payload().help.set(None);
                    let mut fhc = self.chains[help[0]].take().unwrap();
                    let mut shc = self.chains[help[1]].take().unwrap();
                    fhc.push(*pt);
                    shc.push(*pt);
                    self.outputs.push(fc.finish_with(fhc));
                    self.outputs.push(shc.finish_with(sc));
                } else {
                    self.outputs.push(fc.finish_with(sc));
                }
            }
        }
        debug_assert!(incoming.len() <= 2);
        // 处理下段上的帮助并进一步减少到需要扩展的至多两个链索引。
        let in_chains = if let Some(h) = bot_help {
            debug!("提供帮助: {h:?}");
            bot_segment.as_ref().unwrap().payload().help.set(None);
            if !incoming.is_empty() {
                let sc = self.chains[incoming[0].payload().chain_idx.get()]
                    .take()
                    .unwrap();
                let mut shc = self.chains[h[1]].take().unwrap();
                shc.push(*pt);
                self.chains[h[0]].as_mut().unwrap().push(*pt);
                self.outputs.push(shc.finish_with(sc));
                if incoming.len() == 1 {
                    (Some(h[0]), None)
                } else {
                    let last_idx = if let Some(h) = incoming[1].payload().help.get() {
                        let mut fhc = self.chains[h[0]].take().unwrap();
                        let fc = self.chains[incoming[1].payload().chain_idx.get()]
                            .take()
                            .unwrap();
                        fhc.push(*pt);
                        self.chains[h[1]].as_mut().unwrap().push(*pt);
                        self.outputs.push(fc.finish_with(fhc));
                        h[1]
                    } else {
                        incoming[1].payload().chain_idx.get()
                    };
                    (Some(h[0]), Some(last_idx))
                }
            } else {
                self.chains[h[0]].as_mut().unwrap().push(*pt);
                self.chains[h[1]].as_mut().unwrap().push(*pt);
                (Some(h[0]), Some(h[1]))
            }
        } else if incoming.is_empty() {
            (None, None)
        } else {
            let last_incoming = incoming.last().unwrap();
            let last_idx = if let Some(h) = last_incoming.payload().help.get() {
                let mut fhc = self.chains[h[0]].take().unwrap();
                let fc = self.chains[last_incoming.payload().chain_idx.get()]
                    .take()
                    .unwrap();
                fhc.push(*pt);
                self.chains[h[1]].as_mut().unwrap().push(*pt);
                self.outputs.push(fc.finish_with(fhc));
                h[1]
            } else {
                last_incoming.payload().chain_idx.get()
            };
            if incoming.len() == 1 {
                (Some(last_idx), None)
            } else {
                (Some(incoming[0].payload().chain_idx.get()), Some(last_idx))
            }
        };

        // 步骤4：简化出段。任何包围输入区域的两段连续出段可以开始一个新区域。这再次将出段列表简化到最多两个段。
        if !outgoing.is_empty() {
            let n = outgoing.len();
            #[allow(clippy::bool_to_int_with_if)]
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            debug!("简化出段: {n} -> {start_idx}..{ub_idx}");
            let mut iter = outgoing.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let bot = first.line().right();
                let top = second.line().right();
                self.chains
                    .extend(Chain::from_segment_pair(*pt, *bot, *top).map(Some));
                first.payload().next_is_inside.set(true);
                second.payload().next_is_inside.set(false);
                first.payload().chain_idx.set(self.chains.len() - 2);
                second.payload().chain_idx.set(self.chains.len() - 1);
            }
        }
        debug_assert!(outgoing.len() <= 2);

        // 步骤5：绑扎适用的进出段
        debug!("in_chains: {in_chains:?}");
        match in_chains {
            (None, None) => {
                // 简化后没有剩余的进入段。由于我们已经简化了出段，唯一的情况是“分裂顶点”或“<”情况。在这里，我们将使用helper_chain来扩展链。
                if !outgoing.is_empty() {
                    assert!(outgoing.len() == 2);
                    let first = &outgoing[0];
                    let second = &outgoing[1];
                    let bot_segment = bot_segment.as_ref().unwrap();

                    let idx = bot_segment
                        .payload()
                        .helper_chain
                        .get()
                        .unwrap_or_else(|| bot_segment.payload().chain_idx.get());
                    let mut new_chains = self.chains[idx].as_mut().unwrap().swap_at_top(*pt);
                    new_chains[0].push(*first.line().right());
                    new_chains[1].push(*second.line().right());
                    self.chains.extend(new_chains.map(Some));
                    first.payload().next_is_inside.set(false);
                    second.payload().next_is_inside.set(true);
                    first.payload().chain_idx.set(self.chains.len() - 2);
                    second.payload().chain_idx.set(self.chains.len() - 1);

                    bot_segment
                        .payload()
                        .helper_chain
                        .set(Some(self.chains.len() - 2));
                } else {
                    debug_assert!(!bot_region);
                }
            }
            (Some(idx), None) => {
                assert!(outgoing.len() == 1);
                let first = &outgoing[0];
                let bot = first.line().right();
                self.chains[idx].as_mut().unwrap().push(*bot);
                first.payload().next_is_inside.set(!bot_region);
                first.payload().chain_idx.set(idx);
                if let Some(b) = bot_segment {
                    b.payload().helper_chain.set(Some(idx))
                }
            }
            (Some(idx), Some(jdx)) => {
                if !outgoing.is_empty() {
                    assert!(outgoing.len() == 2);
                    let first = &outgoing[0];
                    let second = &outgoing[1];
                    let bot = first.line().right();
                    let top = second.line().right();
                    self.chains[idx].as_mut().unwrap().push(*bot);
                    self.chains[jdx].as_mut().unwrap().push(*top);
                    first.payload().next_is_inside.set(false);
                    second.payload().next_is_inside.set(true);
                    first.payload().chain_idx.set(idx);
                    second.payload().chain_idx.set(jdx);
                } else {
                    debug!("注册帮助: [{}, {}]", idx, jdx);
                    bot_segment
                        .as_ref()
                        .unwrap()
                        .payload()
                        .help
                        .set(Some([idx, jdx]));
                }
                if let Some(b) = bot_segment {
                    b.payload().helper_chain.set(Some(idx))
                }
            }
            _ => unreachable!(),
        }

        true
    }
}

pub(super) struct Chain<T: GeoNum>(LineString<T>);

impl<T: GeoNum + std::fmt::Debug> std::fmt::Debug for Chain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bot: Vec<SweepPoint<T>> = self.0 .0.iter().map(|c| (*c).into()).collect();
        f.debug_tuple("链").field(&bot).finish()
    }
}

impl<T: GeoNum> Chain<T> {
    pub fn from_segment_pair(start: Coord<T>, first: Coord<T>, second: Coord<T>) -> [Self; 2] {
        let [x, y, z] = [SweepPoint::from(start), first.into(), second.into()];
        debug!("从 {x:?} -> {y:?} 创建链");
        debug!("                    {x:?} -> {z:?}");
        [
            Chain(line_string![start, first]),
            Chain(line_string![start, second]),
        ]
    }

    pub fn fix_top(&mut self, pt: Coord<T>) {
        *self.0 .0.last_mut().unwrap() = pt;
    }

    pub fn swap_at_top(&mut self, pt: Coord<T>) -> [Self; 2] {
        let top = self.0 .0.pop().unwrap();
        let prev = *self.0 .0.last().unwrap();
        debug!(
            "链交换: {:?} -> {:?}",
            SweepPoint::from(top),
            SweepPoint::from(pt)
        );
        debug!("\tprev = {:?}", SweepPoint::from(prev));
        self.0 .0.push(pt);

        let old_chain = Chain(replace(&mut self.0 .0, vec![prev, top]).into());
        let new_chain = Chain(vec![prev, pt].into());

        let lp1 = LineOrPoint::from((prev.into(), top.into()));
        let lp2 = LineOrPoint::from((prev.into(), pt.into()));
        if lp1 > lp2 {
            [old_chain, new_chain]
        } else {
            [new_chain, old_chain]
        }
    }

    pub fn push(&mut self, pt: Coord<T>) {
        debug!("链推进: {:?} -> {:?}", self.0 .0.last().unwrap(), pt);
        self.0 .0.push(pt);
    }

    pub fn finish_with(self, other: Self) -> MonoPoly<T> {
        assert!(
            self.0 .0[0] == other.0 .0[0]
                && self.0 .0.last().unwrap() == other.0 .0.last().unwrap(),
            "链必须以相同的起始/结束点结束"
        );
        debug!("结束 {self:?} 与 {other:?}");
        MonoPoly::new(other.0, self.0)
    }
}

#[derive(Debug, Default, Clone)]
struct Info {
    next_is_inside: Cell<bool>,        // 存储下一个点是否位于内侧
    helper_chain: Cell<Option<usize>>, // 存储帮助链的索引
    help: Cell<Option<[usize; 2]>>,    // 存储包含两个索引的帮助信息
    chain_idx: Cell<usize>,            // 存储链索引
}
