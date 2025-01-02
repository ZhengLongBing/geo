use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum};

use crate::Dimensions;

/// 用于从通用坐标访问数据的特征。
///
/// 有关语义和有效性的信息，请参阅 [geo_types::Coord]。
pub trait CoordTrait {
    /// 此几何体的坐标类型
    type T;

    /// 坐标元组的维度
    fn dim(&self) -> Dimensions;

    /// 访问坐标元组的第n个（从0开始）元素。
    /// 如果 `n >= DIMENSION`，则返回 `None`。
    ///
    /// 另请参阅 [`nth_or_panic()`](Self::nth_or_panic) 和 [`nth_unchecked()`](Self::nth_unchecked)。
    ///
    /// # 可能的恐慌
    ///
    /// 如果 [`dim()`](Self::dim) 与此坐标中的实际维度数不对应，此方法可能会恐慌。
    fn nth(&self, n: usize) -> Option<Self::T> {
        if n < self.dim().size() {
            Some(self.nth_or_panic(n))
        } else {
            None
        }
    }

    /// 此坐标的x分量。
    fn x(&self) -> Self::T;

    /// 此坐标的y分量。
    fn y(&self) -> Self::T;

    /// 返回包含坐标的x/水平和y/垂直分量的元组。
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }

    /// 访问坐标元组的第n个（从0开始）元素。
    /// 如果 n >= DIMENSION，可能会恐慌。
    /// 另请参阅 [`nth()`](Self::nth)。
    fn nth_or_panic(&self, n: usize) -> Self::T;

    /// 访问坐标元组的第n个（从0开始）元素。
    /// 如果 n >= DIMENSION，可能会恐慌。
    ///
    /// 另请参阅 [`nth()`](Self::nth), [`nth_or_panic()`](Self::nth_or_panic)。
    ///
    /// 如果您可以提供更高效的实现，可能需要覆盖此方法的默认实现。
    ///
    /// # 安全性
    ///
    /// 虽然它可能会恐慌，但默认实现实际上是安全的。然而，实现者可以使用不安全的实现来实现此方法。
    /// 有关各自的安全性考虑，请参阅各个实现。
    unsafe fn nth_unchecked(&self, n: usize) -> Self::T {
        self.nth_or_panic(n)
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> CoordTrait for Coord<T> {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Coord 仅支持2个维度"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> CoordTrait for &Coord<T> {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Coord 仅支持2个维度"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: Copy> CoordTrait for (T, T) {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("(T, T) 仅支持2个维度"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.0
    }

    fn y(&self) -> Self::T {
        self.1
    }
}

/// 实现 [CoordTrait] 的空结构体。
///
/// 对于没有坐标概念的实现，可以将其用作 `GeometryTrait` 的 `CoordType`
pub struct UnimplementedCoord<T>(PhantomData<T>);

impl<T> CoordTrait for UnimplementedCoord<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn nth_or_panic(&self, _n: usize) -> Self::T {
        unimplemented!()
    }

    fn x(&self) -> Self::T {
        unimplemented!()
    }

    fn y(&self) -> Self::T {
        unimplemented!()
    }
}
