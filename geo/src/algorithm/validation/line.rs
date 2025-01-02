use super::{utils, CoordIndex, Validation};
use crate::{GeoFloat, Line};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidLine {
    /// 一个有效的 [`Line`] 必须至少有两个不同的点 - 必须有非零长度。
    IdenticalCoords,

    /// 一个有效的 [`Line`] 必须具有有限的坐标。
    NonFiniteCoord(CoordIndex),
}

impl fmt::Display for InvalidLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidLine::IdenticalCoords => write!(f, "线有相同的坐标"),
            InvalidLine::NonFiniteCoord(idx) => write!(
                f,
                "{} 坐标不是有限的",
                if idx.0 == 0 { "起始" } else { "结束" }
            ),
        }
    }
}

impl std::error::Error for InvalidLine {}

impl<F: GeoFloat> Validation for Line<F> {
    type Error = InvalidLine;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查起始坐标是否不是有限的
        if utils::check_coord_is_not_finite(&self.start) {
            handle_validation_error(InvalidLine::NonFiniteCoord(CoordIndex(0)))?;
        }
        // 检查结束坐标是否不是有限的
        if utils::check_coord_is_not_finite(&self.end) {
            handle_validation_error(InvalidLine::NonFiniteCoord(CoordIndex(1)))?;
        }
        // 检查起始和结束坐标是否相同
        if self.start == self.end {
            handle_validation_error(InvalidLine::IdenticalCoords)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};

    #[test]
    fn test_line_valid() {
        // 测试有效的线段
        let l = Line::new((0., 0.), (1., 1.));
        assert_valid!(l);
    }

    #[test]
    fn test_line_invalid_not_finite_coords() {
        // 测试线段具有非有限坐标
        let l = Line::new((0., 0.), (f64::NEG_INFINITY, 0.));
        assert_validation_errors!(l, vec![InvalidLine::NonFiniteCoord(CoordIndex(1))]);
    }

    #[test]
    fn test_line_invalid_same_points() {
        // 测试线段起始和结束点相同
        let l = Line::new((0., 0.), (0., 0.));
        assert_validation_errors!(l, vec![InvalidLine::IdenticalCoords]);
    }
}
