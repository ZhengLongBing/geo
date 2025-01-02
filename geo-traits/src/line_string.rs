use std::marker::PhantomData;

use crate::iterator::LineStringIterator;
use crate::{CoordTrait, Dimensions, UnimplementedCoord};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, LineString};

/// 用于从通用LineString访问数据的特征。
///
/// LineString是两个或更多[点][CoordTrait]的有序集合，表示位置之间的路径。
///
/// 有关语义和有效性的信息，请参阅[geo_types::LineString]。
pub trait LineStringTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层坐标的类型，实现[CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此LineString中坐标的迭代器
    fn coords(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::CoordType<'_>> {
        LineStringIterator::new(self, 0, self.num_coords())
    }

    /// 此LineString中坐标的数量
    fn num_coords(&self) -> usize;

    /// 访问此LineString中指定的坐标
    /// 如果提供的索引超出范围，将返回None
    #[inline]
    fn coord(&self, i: usize) -> Option<Self::CoordType<'_>> {
        if i >= self.num_coords() {
            None
        } else {
            unsafe { Some(self.coord_unchecked(i)) }
        }
    }

    /// 访问此LineString中指定的坐标
    ///
    /// # 安全性
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> LineStringTrait for LineString<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> LineStringTrait for &'a LineString<T> {
    type T = T;
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

/// 实现[LineStringTrait]的空结构体。
///
/// 这可以被用作那些没有LineString概念的实现的`GeometryTrait`的`LineStringType`
pub struct UnimplementedLineString<T>(PhantomData<T>);

impl<T> LineStringTrait for UnimplementedLineString<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_coords(&self) -> usize {
        unimplemented!()
    }

    unsafe fn coord_unchecked(&self, _i: usize) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
