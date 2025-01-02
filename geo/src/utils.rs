//! 内部工具函数、类型和数据结构。  

use geo_types::{Coord, CoordFloat, CoordNum};
use num_traits::FromPrimitive;

/// 对可变切片进行原地划分，使其包含所有满足 `predicate(e)` 为 `true` 的元素，
/// 后跟所有 `predicate(e)` 为 `false` 的元素。分别返回包含满足和不满足谓词条件的
/// 子切片。
///
/// https://github.com/llogiq/partition/blob/master/src/lib.rs
pub fn partition_slice<T, P>(data: &mut [T], predicate: P) -> (&mut [T], &mut [T])
where
    P: Fn(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return (&mut [], &mut []);
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return data.split_at_mut(l);
        }
        data.swap(l, r);
    }
}

pub enum EitherIter<I1, I2> {
    A(I1),
    B(I2),
}

impl<I1, I2> ExactSizeIterator for EitherIter<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        match self {
            EitherIter::A(i1) => i1.len(),
            EitherIter::B(i2) => i2.len(),
        }
    }
}

impl<T, I1, I2> Iterator for EitherIter<I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::A(iter) => iter.next(),
            EitherIter::B(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            EitherIter::A(iter) => iter.size_hint(),
            EitherIter::B(iter) => iter.size_hint(),
        }
    }
}

// Rust 标准库为 `Ord` 提供了 `max`，但没有为 `PartialOrd` 提供
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

// Rust 标准库为 `Ord` 提供了 `min`，但没有为 `PartialOrd` 提供
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

use std::cmp::Ordering;

/// 按字典顺序比较两个坐标：先按 x 坐标比较，如相等则比较 y 坐标。
/// 假设坐标都是可比较的（例如，没有 nan）。
#[inline]
pub fn lex_cmp<T: CoordNum>(p: &Coord<T>, q: &Coord<T>) -> Ordering {
    p.x.partial_cmp(&q.x)
        .unwrap()
        .then(p.y.partial_cmp(&q.y).unwrap())
}

/// 计算切片中最小点的索引。比较使用 [`lex_cmp`]。
///
/// 仅应在非空切片且无 `nan` 坐标时调用。
pub fn least_index<T: CoordNum>(pts: &[Coord<T>]) -> usize {
    pts.iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| lex_cmp(p, q))
        .unwrap()
        .0
}

/// 在一次遍历中计算字典顺序上的最小和最大的坐标索引。
///
/// 仅应在非空切片且无 `nan` 坐标时调用。
pub fn least_and_greatest_index<T: CoordNum>(pts: &[Coord<T>]) -> (usize, usize) {
    assert_ne!(pts.len(), 0);
    let (min, max) = pts
        .iter()
        .enumerate()
        .fold((None, None), |(min, max), (idx, p)| {
            (
                if let Some((midx, min)) = min {
                    if lex_cmp(p, min) == Ordering::Less {
                        Some((idx, p))
                    } else {
                        Some((midx, min))
                    }
                } else {
                    Some((idx, p))
                },
                if let Some((midx, max)) = max {
                    if lex_cmp(p, max) == Ordering::Greater {
                        Some((idx, p))
                    } else {
                        Some((midx, max))
                    }
                } else {
                    Some((idx, p))
                },
            )
        });
    (min.unwrap().0, max.unwrap().0)
}

/// 规范化经度坐标以确保其在 [-180,180] 范围内
pub fn normalize_longitude<T: CoordFloat + FromPrimitive>(coord: T) -> T {
    let one_eighty = T::from(180.0f64).unwrap();
    let three_sixty = T::from(360.0f64).unwrap();
    let five_forty = T::from(540.0f64).unwrap();

    ((coord + five_forty) % three_sixty) - one_eighty
}

#[cfg(test)]
mod test {
    use super::{partial_max, partial_min};

    #[test]
    fn test_partial_max() {
        assert_eq!(5, partial_max(5, 4));
        assert_eq!(5, partial_max(5, 5));
    }

    #[test]
    fn test_partial_min() {
        assert_eq!(4, partial_min(5, 4));
        assert_eq!(4, partial_min(4, 4));
    }
}
