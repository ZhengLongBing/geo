use std::marker::PhantomData;

use crate::{CoordTrait, Dimensions, UnimplementedCoord};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Line};

/// 用于从通用线段访问数据的特征。
///
/// 线段由恰好两个[坐标][CoordTrait]组成。
///
/// 有关语义和有效性的信息，请参阅 [geo_types::Line]。
pub trait LineTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层坐标的类型，实现 [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 访问此线段的起始坐标
    fn start(&self) -> Self::CoordType<'_>;

    /// 访问此线段的结束坐标
    fn end(&self) -> Self::CoordType<'_>;

    /// 访问两个底层坐标
    fn coords(&self) -> [Self::CoordType<'_>; 2] {
        [self.start(), self.end()]
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> LineTrait for Line<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> LineTrait for &'a Line<T> {
    type T = T;
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

/// 实现 [LineTrait] 的空结构体。
///
/// 这可以被没有线段概念的实现用作 `GeometryTrait` 的 `LineType`
pub struct UnimplementedLine<T>(PhantomData<T>);

impl<T> LineTrait for UnimplementedLine<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn start(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn end(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
