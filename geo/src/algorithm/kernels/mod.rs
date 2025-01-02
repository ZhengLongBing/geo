use num_traits::Zero;
use std::cmp::Ordering;

use crate::{coord, Coord, CoordNum};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Orientation {
    CounterClockwise, // 逆时针
    Clockwise,        // 顺时针
    Collinear,        // 共线
}

impl Orientation {
    /// 帮助将2D方向转换为排序。
    #[inline]
    pub(crate) fn as_ordering(&self) -> Ordering {
        match self {
            Orientation::CounterClockwise => Ordering::Less,
            Orientation::Clockwise => Ordering::Greater,
            Orientation::Collinear => Ordering::Equal,
        }
    }
}

/// Kernel特性用于提供操作不同标量类型的谓词。
pub trait Kernel<T: CoordNum> {
    /// 给出三个二维点的方向：逆时针、顺时针或共线（None）
    fn orient2d(p: Coord<T>, q: Coord<T>, r: Coord<T>) -> Orientation {
        let res = (q.x - p.x) * (r.y - q.y) - (q.y - p.y) * (r.x - q.x);
        if res > Zero::zero() {
            Orientation::CounterClockwise
        } else if res < Zero::zero() {
            Orientation::Clockwise
        } else {
            Orientation::Collinear
        }
    }

    /// 计算两个点之间的平方欧几里得距离
    fn square_euclidean_distance(p: Coord<T>, q: Coord<T>) -> T {
        (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
    }

    /// 使用鲁棒谓词计算向量`u`和`v`点积的符号。
    /// 如果符号为正，则输出为`CounterClockwise`，
    /// 如果为负，则为`Clockwise`，如果为零，则为`Collinear`。
    fn dot_product_sign(u: Coord<T>, v: Coord<T>) -> Orientation {
        let zero = Coord::zero();
        let vdash = coord! {
            x: T::zero() - v.y,
            y: v.x,
        };
        Self::orient2d(zero, u, vdash)
    }
}

pub mod robust; // 鲁棒模块
pub use self::robust::RobustKernel;

pub mod simple; // 简单模块
pub use self::simple::SimpleKernel;
