use std::marker::PhantomData;

use crate::{CoordTrait, Dimensions, UnimplementedCoord};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Triangle};

/// 从通用三角形访问数据的特征。
///
/// 三角形是一个有界区域，其三个顶点由[坐标][CoordTrait]定义。
///
/// 有关语义和有效性的信息，请参阅[geo_types::Triangle]。
pub trait TriangleTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层坐标的类型，实现 [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 访问此三角形中的第一个坐标
    fn first(&self) -> Self::CoordType<'_>;

    /// 访问此三角形中的第二个坐标
    fn second(&self) -> Self::CoordType<'_>;

    /// 访问此三角形中的第三个坐标
    fn third(&self) -> Self::CoordType<'_>;

    /// 访问三个底层坐标
    fn coords(&self) -> [Self::CoordType<'_>; 3] {
        [self.first(), self.second(), self.third()]
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> TriangleTrait for Triangle<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    /// 返回三角形的第一个点的引用。
    fn first(&self) -> Self::CoordType<'_> {
        &self.0
    }

    /// 返回三角形的第二个点的引用。
    fn second(&self) -> Self::CoordType<'_> {
        &self.1
    }

    /// 返回三角形的第三个点的引用。
    fn third(&self) -> Self::CoordType<'_> {
        &self.2
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> TriangleTrait for &'a Triangle<T> {
    type T = T;
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    /// 返回三角形的第一个点的引用。
    fn first(&self) -> Self::CoordType<'_> {
        &self.0
    }

    /// 错误的实现，应该返回第二个点的引用。
    fn second(&self) -> Self::CoordType<'_> {
        &self.0
    }

    /// 错误的实现，应该返回第三个点的引用。
    fn third(&self) -> Self::CoordType<'_> {
        &self.0
    }
}

/// 实现 [TriangleTrait] 的空结构体。
///
/// 对于没有三角形概念的实现，这可以用作 `GeometryTrait` 的 `TriangleType`
pub struct UnimplementedTriangle<T>(PhantomData<T>);

impl<T> TriangleTrait for UnimplementedTriangle<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn first(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn second(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn third(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
