use std::marker::PhantomData;

use crate::iterator::MultiLineStringIterator;
use crate::line_string::UnimplementedLineString;
use crate::{Dimensions, LineStringTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, LineString, MultiLineString};

/// 用于从通用MultiLineString访问数据的特征。
///
/// MultiLineString是[`LineString`s][LineStringTrait]的集合。
///
/// 有关语义和有效性的信息，请参阅[geo_types::MultiLineString]。
pub trait MultiLineStringTrait: Sized {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层LineString的类型，实现[LineStringTrait]
    type LineStringType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 此MultiLineString中LineString的迭代器
    fn line_strings(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::LineStringType<'_>> {
        MultiLineStringIterator::new(self, 0, self.num_line_strings())
    }

    /// 此MultiLineString中line_string的数量
    fn num_line_strings(&self) -> usize;

    /// 访问此MultiLineString中指定的line_string
    /// 如果提供的索引超出范围，将返回None
    fn line_string(&self, i: usize) -> Option<Self::LineStringType<'_>> {
        if i >= self.num_line_strings() {
            None
        } else {
            unsafe { Some(self.line_string_unchecked(i)) }
        }
    }

    /// 访问此MultiLineString中指定的line_string
    ///
    /// # 安全性
    ///
    /// 访问超出范围的索引是未定义行为。
    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiLineStringTrait for MultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = &'a LineString<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiLineStringTrait for &'a MultiLineString<T> {
    type T = T;
    type LineStringType<'b>
        = &'a LineString<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

/// 实现[MultiLineStringTrait]的空结构体。
///
/// 这可以被没有MultiLineString概念的实现用作`GeometryTrait`的`MultiLineStringType`
pub struct UnimplementedMultiLineString<T>(PhantomData<T>);

impl<T> MultiLineStringTrait for UnimplementedMultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = UnimplementedLineString<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_line_strings(&self) -> usize {
        unimplemented!()
    }

    unsafe fn line_string_unchecked(&self, _i: usize) -> Self::LineStringType<'_> {
        unimplemented!()
    }
}
