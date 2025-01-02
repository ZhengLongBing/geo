use std::{fmt::Debug, rc::Rc, sync::Arc};

use geo_types::Line;

use super::*;
use crate::GeoFloat;

/// 可以处理以检测交叉点的类型接口。
///
/// 此类型由 [`LineOrPoint`] 实现，但用户也可以在自定义类型上实现
/// 以存储额外的信息。任何代表有序线段的类型都可以实现此接口。
///
/// # 克隆
///
/// 注意，为了在平面扫描迭代器中使用，该类型必须实现 `Clone`。
/// 如果自定义类型克隆成本较高，请使用对类型的引用、[`Rc`] 或 [`Arc`]。
/// 所有这些都通过 blanket 特征实现来支持。
pub trait Cross: Sized + Debug {
    /// 用于坐标的标量。
    type Scalar: GeoFloat;

    /// 与此类型关联的几何。使用带有 `start` 和 `end` 坐标的 `Line` 来表示一个点。
    fn line(&self) -> LineOrPoint<Self::Scalar>;
}

impl<T: Cross> Cross for &'_ T {
    type Scalar = T::Scalar;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        T::line(*self)
    }
}

impl<T: GeoFloat> Cross for LineOrPoint<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        *self
    }
}

impl<T: GeoFloat> Cross for Line<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        (*self).into()
    }
}

// 定义 blanket_impl_smart_pointer 宏，用于实现 Cross 特征，方便为智能指针类型添加实现
macro_rules! blanket_impl_smart_pointer {
    ($ty:ty) => {
        impl<T: Cross> Cross for $ty {
            type Scalar = T::Scalar;

            fn line(&self) -> LineOrPoint<Self::Scalar> {
                T::line(self)
            }
        }
    };
}
// 使用 blanket_impl_smart_pointer 宏为 Box、Rc 和 Arc 类型实现 Cross 特征
blanket_impl_smart_pointer!(Box<T>);
blanket_impl_smart_pointer!(Rc<T>);
blanket_impl_smart_pointer!(Arc<T>);
