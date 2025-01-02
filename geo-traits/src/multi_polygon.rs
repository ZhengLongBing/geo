use std::marker::PhantomData;

use crate::iterator::MultiPolygonIterator;
use crate::polygon::UnimplementedPolygon;
use crate::{Dimensions, PolygonTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, MultiPolygon, Polygon};

/// 用于从通用MultiPolygon访问数据的特征。
///
/// 有关语义和有效性的信息，请参阅 [geo_types::MultiPolygon]。
pub trait MultiPolygonTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层Polygon的类型，实现 [PolygonTrait]
    type PolygonType<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此MultiPolygon中Polygon的迭代器
    fn polygons(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::PolygonType<'_>> {
        MultiPolygonIterator::new(self, 0, self.num_polygons())
    }

    /// 此MultiPolygon中polygon的数量
    fn num_polygons(&self) -> usize;

    /// 访问此MultiPolygon中指定的polygon
    /// 如果提供的索引超出范围，将返回None
    fn polygon(&self, i: usize) -> Option<Self::PolygonType<'_>> {
        if i >= self.num_polygons() {
            None
        } else {
            unsafe { Some(self.polygon_unchecked(i)) }
        }
    }

    /// 访问此MultiPolygon中指定的polygon
    ///
    /// # 安全性
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiPolygonTrait for MultiPolygon<T> {
    type T = T;
    type PolygonType<'a>
        = &'a Polygon<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiPolygonTrait for &'a MultiPolygon<T> {
    type T = T;
    type PolygonType<'b>
        = &'a Polygon<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.0.get_unchecked(i)
    }
}

/// 实现 [MultiPolygonTrait] 的空结构体。
///
/// 这可以被没有MultiPolygon概念的实现用作 `GeometryTrait` 的 `MultiPolygonType`
pub struct UnimplementedMultiPolygon<T>(PhantomData<T>);

impl<T> MultiPolygonTrait for UnimplementedMultiPolygon<T> {
    type T = T;
    type PolygonType<'a>
        = UnimplementedPolygon<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_polygons(&self) -> usize {
        unimplemented!()
    }

    unsafe fn polygon_unchecked(&self, _i: usize) -> Self::PolygonType<'_> {
        unimplemented!()
    }
}
