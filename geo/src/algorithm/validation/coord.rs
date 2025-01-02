use super::{utils, Validation};
use crate::{Coord, GeoFloat};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidCoord {
    /// 一个有效的 [`Coord`] 必须是有限的。
    NonFinite,
}

impl fmt::Display for InvalidCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidCoord::NonFinite => write!(f, "坐标不是有限的"),
        }
    }
}

impl std::error::Error for InvalidCoord {}

impl<F: GeoFloat> Validation for Coord<F> {
    type Error = InvalidCoord;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查坐标是否不是有限的
        if utils::check_coord_is_not_finite(self) {
            // 处理验证错误
            handle_validation_error(InvalidCoord::NonFinite)?;
        }
        Ok(())
    }
}
