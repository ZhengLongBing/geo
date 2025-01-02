use std::marker::PhantomData;

use crate::iterator::MultiPointIterator;
use crate::{Dimensions, PointTrait, UnimplementedPoint};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, MultiPoint, Point};

/// 用于从通用MultiPoint访问数据的特征。
///
/// MultiPoint是[`Point`s][PointTrait]的集合。
///
/// 有关语义和有效性的信息，请参阅[geo_types::MultiPoint]。
pub trait MultiPointTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层Point的类型，实现[PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此MultiPoint中点的迭代器
    fn points(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::PointType<'_>> {
        MultiPointIterator::new(self, 0, self.num_points())
    }

    /// 此MultiPoint中点的数量
    fn num_points(&self) -> usize;

    /// 访问此MultiPoint中指定的点
    /// 如果提供的索引超出范围，将返回None
    fn point(&self, i: usize) -> Option<Self::PointType<'_>> {
        if i >= self.num_points() {
            None
        } else {
            unsafe { Some(self.point_unchecked(i)) }
        }
    }

    /// 访问此MultiPoint中指定的点
    ///
    /// # 安全性
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type PointType<'a>
        = &'a Point<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiPointTrait for &'a MultiPoint<T> {
    type T = T;
    type PointType<'b>
        = &'a Point<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.0.get_unchecked(i)
    }
}

/// 实现[MultiPointTrait]的空结构体。
///
/// 这可以被没有MultiPoint概念的实现用作`GeometryTrait`的`MultiPointType`
pub struct UnimplementedMultiPoint<T>(PhantomData<T>);

impl<T> MultiPointTrait for UnimplementedMultiPoint<T> {
    type T = T;
    type PointType<'a>
        = UnimplementedPoint<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_points(&self) -> usize {
        unimplemented!()
    }

    unsafe fn point_unchecked(&self, _i: usize) -> Self::PointType<'_> {
        unimplemented!()
    }
}
