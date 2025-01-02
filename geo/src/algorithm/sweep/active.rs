use std::{borrow::Borrow, cmp::Ordering, fmt::Debug, ops::Deref};

/// 当前在扫描中的线段。
///
/// 当扫描线从左向右推进时，它会与一组线段相交。这些线段可以从下到上进行完全排序，
/// 并且高效访问线段的邻居是平面扫描算法的关键方面。
///
/// 我们断言`Ord`，即使内部类型通常只有`T: PartialOrd`。
/// 比较两个不能比较的Active是一个逻辑错误。这由算法确保（编译器无法推断出？）。
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub(in crate::algorithm) struct Active<T>(pub(in crate::algorithm) T);

impl<T> Active<T> {
    pub(in crate::algorithm) fn active_ref(t: &T) -> &Active<T> {
        unsafe { std::mem::transmute(t) }
    }
}

impl<T> Borrow<T> for Active<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Active<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 断言完全相等。
impl<T: PartialEq> Eq for Active<T> {}

/// 断言活动线段的完全排序。
impl<T: PartialOrd + Debug> Ord for Active<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(c) = T::partial_cmp(self, other) {
            c
        } else {
            warn!("无法比较线段:\n\t{self:?}\n\t{other:?}");
            panic!("无法比较活动线段！");
        }
    }
}

impl<T: PartialOrd + Debug> PartialOrd for Active<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 抽象出活动线段容器的特征。
#[allow(dead_code)]
pub(in crate::algorithm) trait ActiveSet: Default {
    type Seg;
    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn previous(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.previous_find(segment, |_| true)
    }
    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn next(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.next_find(segment, |_| true)
    }
    fn insert_active(&mut self, segment: Self::Seg);
    fn remove_active(&mut self, segment: &Self::Seg);
}
