use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Rect};

use crate::{CoordTrait, Dimensions, UnimplementedCoord};

/// 从通用矩形访问数据的特征。
///
/// 矩形是一个_轴对齐_的有界二维矩形，其面积由最小和最大[`点`][CoordTrait]定义。
pub trait RectTrait {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层坐标的类型，实现[CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此矩形的最小坐标
    fn min(&self) -> Self::CoordType<'_>;

    /// 此矩形的最大坐标
    fn max(&self) -> Self::CoordType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> RectTrait for Rect<T> {
    type T = T;
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::CoordType<'_> {
        Rect::min(*self)
    }

    fn max(&self) -> Self::CoordType<'_> {
        Rect::max(*self)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type T = T;
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::CoordType<'_> {
        Rect::min(**self)
    }

    fn max(&self) -> Self::CoordType<'_> {
        Rect::max(**self)
    }
}

/// 实现[RectTrait]的空结构体。
///
/// 这可以被没有矩形概念的实现用作`GeometryTrait`的`RectType`
pub struct UnimplementedRect<T>(PhantomData<T>);

impl<T> RectTrait for UnimplementedRect<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn min(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn max(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
