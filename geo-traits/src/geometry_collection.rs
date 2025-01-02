use std::marker::PhantomData;

use crate::iterator::GeometryCollectionIterator;
use crate::{Dimensions, GeometryTrait, UnimplementedGeometry};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, Geometry, GeometryCollection};

/// 用于从通用几何集合访问数据的特征。
///
/// 几何集合是 [Geometry][GeometryTrait] 类型的集合。
pub trait GeometryCollectionTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层几何体的类型，需实现 [GeometryTrait]
    type GeometryType<'a>: 'a + GeometryTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此几何集合中几何体的迭代器
    fn geometries(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::GeometryType<'_>> {
        GeometryCollectionIterator::new(self, 0, self.num_geometries())
    }

    /// 此几何集合中几何体的数量
    fn num_geometries(&self) -> usize;

    /// 访问此几何集合中指定的几何体
    /// 如果提供的索引超出范围，将返回 None
    fn geometry(&self, i: usize) -> Option<Self::GeometryType<'_>> {
        if i >= self.num_geometries() {
            None
        } else {
            unsafe { Some(self.geometry_unchecked(i)) }
        }
    }

    /// 访问此几何集合中指定的几何体
    ///
    /// # 安全性
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> GeometryCollectionTrait for GeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = &'a Geometry<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> GeometryCollectionTrait for &'a GeometryCollection<T> {
    type T = T;
    type GeometryType<'b>
        = &'a Geometry<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.0.get_unchecked(i)
    }
}

/// 实现 [GeometryCollectionTrait] 的空结构体。
///
/// 这可以被用作那些没有几何集合概念的实现的 `GeometryTrait` 的 `GeometryCollectionType`
pub struct UnimplementedGeometryCollection<T>(PhantomData<T>);

impl<T> GeometryCollectionTrait for UnimplementedGeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = UnimplementedGeometry<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_geometries(&self) -> usize {
        unimplemented!()
    }

    unsafe fn geometry_unchecked(&self, _i: usize) -> Self::GeometryType<'_> {
        unimplemented!()
    }
}
