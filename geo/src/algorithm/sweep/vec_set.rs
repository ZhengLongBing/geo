use super::{Active, ActiveSet};
use std::{cmp::Ordering, fmt::Debug, ops::Index};

/// 一个简单的有序集合实现，基于 `Vec`。
#[derive(Debug, Clone)]
pub struct VecSet<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T: PartialOrd + Debug> VecSet<Active<T>> {
    /// 获取满足条件 `pred` 的分割点索引
    pub fn partition_point<P>(&self, mut pred: P) -> usize
    where
        P: FnMut(&T) -> bool,
    {
        self.data.partition_point(|s| pred(&s.0))
    }

    /// 获取指定段的索引
    pub fn index_of(&self, segment: &T) -> usize {
        self.data
            .binary_search(Active::active_ref(segment))
            .expect("段未在active-vec-set中找到")
    }

    /// 获取不存在的指定段的索引
    pub fn index_not_of(&self, segment: &T) -> usize {
        self.data
            .binary_search(Active::active_ref(segment))
            .expect_err("段已经在active-vec-set中找到")
    }

    /// 获取集合的长度
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 在指定索引处插入一个段
    pub fn insert_at(&mut self, idx: usize, segment: T) {
        self.data.insert(idx, Active(segment))
    }

    /// 从指定索引移除段并返回它
    pub fn remove_at(&mut self, idx: usize) -> T {
        self.data.remove(idx).0
    }

    /// 检查并根据顺序交换数据
    #[allow(unused)]
    pub fn check_swap(&mut self, idx: usize) -> bool {
        if self.data[idx].cmp(&self.data[idx + 1]) == Ordering::Greater {
            self.data.swap(idx, idx + 1);
            true
        } else {
            false
        }
    }
}

impl<T: Ord> Index<usize> for VecSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: PartialOrd + Debug> ActiveSet for VecSet<Active<T>> {
    type Seg = T;

    /// 在数据集合里向后查找满足条件的段
    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        let segment = Active::active_ref(segment);
        let ub = match self.data.binary_search(segment) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.data[..ub].iter().rev().find(|s| f(s))
    }

    /// 在数据集合里向前查找满足条件的段
    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        let segment = Active::active_ref(segment);
        let start = match self.data.binary_search(segment) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        self.data[start..].iter().find(|s| f(s))
    }

    /// 插入一个活动段
    fn insert_active(&mut self, segment: Self::Seg) {
        let idx = {
            let segment = Active::active_ref(&segment);
            self.data
                .binary_search(segment)
                .expect_err("元素已在active-vec-set中")
        };
        self.data.insert(idx, Active(segment));
    }

    /// 移除一个活动段
    fn remove_active(&mut self, segment: &Self::Seg) {
        let segment = Active::active_ref(segment);
        let idx = self
            .data
            .binary_search(segment)
            .expect("元素未在active-vec-set中找到");
        self.data.remove(idx);
    }
}
