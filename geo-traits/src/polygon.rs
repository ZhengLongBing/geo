/// Start of Selection
use std::marker::PhantomData;

use crate::iterator::PolygonInteriorIterator;
use crate::line_string::UnimplementedLineString;
use crate::{Dimensions, LineStringTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, LineString, Polygon};

/// 用于从通用Polygon访问数据的特征。
///
/// `Polygon` 的外边界（_exterior ring_ ）由[`LineString`][LineStringTrait]表示。它可能包含零个或多个洞（_interior rings_ ），也由`LineString`表示。
///
/// 有关语义和有效性的信息，请参阅 [geo_types::Polygon]。
pub trait PolygonTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层环的类型，实现 [LineStringTrait]
    type RingType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 多边形的外环
    fn exterior(&self) -> Option<Self::RingType<'_>>;

    /// 此多边形的内环迭代器
    fn interiors(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::RingType<'_>> {
        PolygonInteriorIterator::new(self, 0, self.num_interiors())
    }

    /// 此多边形中内环的数量
    fn num_interiors(&self) -> usize;

    /// 访问此多边形中指定的内环
    /// 如果提供的索引超出范围，将返回None
    fn interior(&self, i: usize) -> Option<Self::RingType<'_>> {
        if i >= self.num_interiors() {
            None
        } else {
            unsafe { Some(self.interior_unchecked(i)) }
        }
    }

    /// 访问此多边形中指定的内环
    ///
    /// # 安全
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> PolygonTrait for Polygon<T> {
    type T = T;
    type RingType<'a>
        = &'a LineString<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        let ext_ring = Polygon::exterior(self);
        if LineStringTrait::num_coords(&ext_ring) == 0 {
            None
        } else {
            Some(ext_ring)
        }
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> PolygonTrait for &'a Polygon<T> {
    type T = T;
    type RingType<'b>
        = &'a LineString<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        let ext_ring = Polygon::exterior(self);
        if LineStringTrait::num_coords(&ext_ring) == 0 {
            None
        } else {
            Some(ext_ring)
        }
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}

/// 实现 [PolygonTrait] 的空结构体。
///
/// 这可以被没有Polygon概念的实现用作`GeometryTrait`的`PolygonType`
pub struct UnimplementedPolygon<T>(PhantomData<T>);

impl<T> PolygonTrait for UnimplementedPolygon<T> {
    type T = T;
    type RingType<'a>
        = UnimplementedLineString<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        unimplemented!()
    }

    fn num_interiors(&self) -> usize {
        unimplemented!()
    }

    unsafe fn interior_unchecked(&self, _i: usize) -> Self::RingType<'_> {
        unimplemented!()
    }
}
