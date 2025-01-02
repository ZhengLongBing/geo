use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Point};

use crate::{CoordTrait, Dimensions, UnimplementedCoord};

/// 用于从通用点访问数据的特征。
///
/// 有关语义和有效性的信息，请参阅 [geo_types::Point]。
pub trait PointTrait {
    /// 此几何体的坐标类型
    type T;

    /// 底层坐标的类型，实现 [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// 坐标元组的维度
    fn dim(&self) -> Dimensions;

    /// 此0维几何体的位置。
    ///
    /// 根据简单要素规范，一个点可以没有坐标并被视为"空"。
    fn coord(&self) -> Option<Self::CoordType<'_>>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> PointTrait for Point<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> PointTrait for &Point<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }
}

/// 实现 [PointTrait] 的空结构体。
///
/// 这可以被没有点概念的实现用作 `GeometryTrait` 的 `PointType`
pub struct UnimplementedPoint<T>(PhantomData<T>);

impl<T> PointTrait for UnimplementedPoint<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        unimplemented!()
    }

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }
}
