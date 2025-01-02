use super::{utils, CoordIndex, Validation};
use crate::{GeoFloat, Rect};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidRect {
    /// 一个有效的 [`Rect`] 必须有有限的坐标。
    /// 索引 `0` 表示最小坐标，索引 `1` 表示最大坐标。
    NonFiniteCoord(CoordIndex),
}

impl std::error::Error for InvalidRect {}

impl fmt::Display for InvalidRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidRect::NonFiniteCoord(idx) => {
                let corner = if idx.0 == 0 { "min" } else { "max" }; // 判断索引是最小还是最大
                write!(f, "rect 的坐标 {corner} 是非有限的") // 输出错误信息
            }
        }
    }
}

impl<F: GeoFloat> Validation for Rect<F> {
    type Error = InvalidRect;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查最小坐标是否非有限
        if utils::check_coord_is_not_finite(&self.min()) {
            handle_validation_error(InvalidRect::NonFiniteCoord(CoordIndex(0)))?;
        }
        // 检查最大坐标是否非有限
        if utils::check_coord_is_not_finite(&self.max()) {
            handle_validation_error(InvalidRect::NonFiniteCoord(CoordIndex(1)))?;
        }
        Ok(())
    }
}
