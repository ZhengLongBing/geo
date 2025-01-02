use super::{GeometryIndex, Validation};
use crate::algorithm::validation::line_string::InvalidLineString;
use crate::{GeoFloat, MultiLineString};

use std::fmt;

/// 只有当 [`MultiLineString`] 中的每个 [`LineString`](crate::LineString) 都有效时，它才是有效的。
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidMultiLineString {
    /// 表示哪个元素无效，以及无效的原因。
    InvalidLineString(GeometryIndex, InvalidLineString),
}

impl fmt::Display for InvalidMultiLineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidMultiLineString::InvalidLineString(idx, err) => {
                write!(f, "索引为 {} 的线字符串无效: {}", idx.0, err)
            }
        }
    }
}

impl std::error::Error for InvalidMultiLineString {}

impl<F: GeoFloat> Validation for MultiLineString<F> {
    type Error = InvalidMultiLineString;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 遍历 MultiLineString 中的每个 LineString
        for (i, line_string) in self.0.iter().enumerate() {
            line_string.visit_validation(Box::new(&mut |line_string_err| {
                // 如果 LineString 无效，处理验证错误
                let err =
                    InvalidMultiLineString::InvalidLineString(GeometryIndex(i), line_string_err);
                handle_validation_error(err)
            }))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{
        assert_valid, assert_validation_errors, InvalidLineString, InvalidMultiLineString,
    };
    use crate::wkt;

    #[test]
    fn test_multilinestring_valid() {
        let mls = wkt!(
            MULTILINESTRING(
                (0. 0.,1. 1.),
                (3. 1.,4. 1.)
            )
        );
        assert_valid!(&mls);
    }

    #[test]
    fn test_multilinestring_invalid_too_few_points_with_duplicate() {
        // 这个 MultiLineString 的第二个 LineString（索引为1）是无效的，因为它只有一个（去重后的）点
        let mls = wkt!(
            MULTILINESTRING(
                (0. 0.,1. 1.),
                (0. 0.,0. 0.)
            )
        );
        assert_validation_errors!(
            &mls,
            vec![InvalidMultiLineString::InvalidLineString(
                GeometryIndex(1),
                InvalidLineString::TooFewPoints
            )]
        );
    }
}
