use super::{utils, CoordIndex, Validation};
use crate::{CoordFloat, Triangle};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidTriangle {
    /// 一个有效的 [`Triangle`] 必须具有有限的坐标。
    NonFiniteCoord(CoordIndex),
    /// 一个有效的 [`Triangle`] 必须具有不同的点。
    IdenticalCoords(CoordIndex, CoordIndex),
    /// 一个有效的 [`Triangle`] 必须具有非共线的点。
    CollinearCoords,
}

impl fmt::Display for InvalidTriangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidTriangle::NonFiniteCoord(idx) => {
                write!(f, "坐标在索引 {} 处不是有限的", idx.0)
            }
            InvalidTriangle::IdenticalCoords(idx1, idx2) => {
                write!(f, "坐标在索引 {} 和 {} 处是相同的", idx1.0, idx2.0)
            }
            InvalidTriangle::CollinearCoords => write!(f, "三角形具有共线的坐标"),
        }
    }
}

impl std::error::Error for InvalidTriangle {}

impl<F: CoordFloat> Validation for Triangle<F> {
    type Error = InvalidTriangle;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查坐标是否不是有限的
        if utils::check_coord_is_not_finite(&self.0) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(0)))?;
        }
        if utils::check_coord_is_not_finite(&self.1) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(1)))?;
        }
        if utils::check_coord_is_not_finite(&self.2) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(2)))?;
        }

        // 如果点是相同的，则不会检查它们是否共线
        let mut identical = false;

        if self.0 == self.1 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(0),
                CoordIndex(1),
            ))?;
            identical = true;
        }
        if self.0 == self.2 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(0),
                CoordIndex(2),
            ))?;
            identical = true;
        }
        if self.1 == self.2 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(1),
                CoordIndex(2),
            ))?;
            identical = true;
        }

        // 检查点是否共线
        if !identical && utils::robust_check_points_are_collinear::<F>(&self.0, &self.1, &self.2) {
            handle_validation_error(InvalidTriangle::CollinearCoords)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};

    #[test]
    fn test_triangle_valid() {
        // 测试有效的三角形
        let t = Triangle((0., 0.).into(), (0., 1.).into(), (0.5, 2.).into());
        assert_valid!(t);
    }

    #[test]
    fn test_triangle_invalid_same_points() {
        // 测试具有相同点的无效三角形
        let t = Triangle((0., 0.).into(), (0., 1.).into(), (0., 1.).into());
        assert_validation_errors!(
            t,
            vec![InvalidTriangle::IdenticalCoords(
                CoordIndex(1),
                CoordIndex(2)
            )]
        );
    }

    #[test]
    fn test_triangle_invalid_points_collinear() {
        // 测试具有共线点的无效三角形
        let t = Triangle((0., 0.).into(), (1., 1.).into(), (2., 2.).into());
        assert_validation_errors!(t, vec![InvalidTriangle::CollinearCoords]);
    }
}
