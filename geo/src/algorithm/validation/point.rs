use super::{utils, Validation};
use crate::{GeoFloat, Point};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidPoint {
    /// 一个有效的 [`Point`] 必须具有有限的坐标。
    NonFiniteCoord,
}

impl fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidPoint::NonFiniteCoord => write!(f, "点的坐标不是有限的"),
        }
    }
}

impl std::error::Error for InvalidPoint {}

impl<F: GeoFloat> Validation for Point<F> {
    type Error = InvalidPoint;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查点坐标是否不是有限的
        if utils::check_coord_is_not_finite(&self.0) {
            handle_validation_error(InvalidPoint::NonFiniteCoord)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::Point;

    #[test]
    fn test_point_valid() {
        // 测试有效的点
        let p = Point::new(0., 0.);
        assert_valid!(p);
    }

    #[test]
    fn test_point_validation_errors() {
        // 测试具有非有限坐标的点
        let p = Point::new(f64::NAN, f64::NAN);
        assert_validation_errors!(p, vec![InvalidPoint::NonFiniteCoord]);
    }

    #[test]
    fn test_point_check_validation() {
        // 测试检查点的验证错误
        let p = Point::new(f64::NAN, f64::NAN);

        let err = p.check_validation().unwrap_err();
        assert_eq!(err, InvalidPoint::NonFiniteCoord);
    }
}
