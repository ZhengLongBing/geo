//! 提供了一种检查几何体有效性的方法，基于 [OGC Simple Feature Access - Part 1: Common Architecture standard]。
//!
//! [OGC Simple Feature Access - Part 1: Common Architecture standard]: https://www.ogc.org/standards/sfa
mod coord;
mod geometry;
mod geometry_collection;
mod line;
mod line_string;
mod multi_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;
mod rect;
#[cfg(test)]
mod tests;
mod triangle;
mod utils;

pub use geometry::InvalidGeometry;
pub use geometry_collection::InvalidGeometryCollection;
pub use line::InvalidLine;
pub use line_string::InvalidLineString;
pub use multi_line_string::InvalidMultiLineString;
pub use multi_point::InvalidMultiPoint;
pub use multi_polygon::InvalidMultiPolygon;
pub use point::InvalidPoint;
pub use polygon::InvalidPolygon;
pub use rect::InvalidRect;
pub use triangle::InvalidTriangle;

use std::boxed::Box;
use std::fmt;

/// 验证几何体是否有效，并报告无效原因的一个 trait。
///
/// ```
/// use geo::algorithm::Validation;
/// use geo::wkt;
///
/// let valid_polygon = wkt!(POLYGON((0. 0., 1. 1., 1. 0., 0. 0.)));
/// assert!(valid_polygon.is_valid());
///
/// let invalid_polygon = wkt!(POLYGON((0. 0., 1. 1.),(3. 3., 3. 4.,4. 4.)));
/// assert!(!invalid_polygon.is_valid());
///
/// // 获取第一个验证错误，作为 `Result`
/// let validation_error = invalid_polygon.check_validation().unwrap_err();
/// use geo::algorithm::validation::{InvalidPolygon, RingRole};
/// assert_eq!(validation_error, InvalidPolygon::TooFewPointsInRing(RingRole::Exterior));
///
/// // 获取一个可读的错误信息
/// let text = validation_error.to_string();
/// assert_eq!(text, "exterior ring must have at least 3 distinct points");
///
/// // 获取所有的验证错误
/// let all_validation_errors = invalid_polygon.validation_errors();
/// assert_eq!(all_validation_errors.len(), 2);
/// assert_eq!(all_validation_errors[0].to_string(), "exterior ring must have at least 3 distinct points");
/// assert_eq!(all_validation_errors[1].to_string(), "interior ring at index 0 is not contained within the polygon's exterior");
/// ```
pub trait Validation {
    type Error: std::error::Error;

    /// 检查几何体是否有效。
    fn is_valid(&self) -> bool {
        self.check_validation().is_ok()
    }

    /// 返回几何体无效的原因。
    ///
    /// 尽管我们尝试返回几何体的*所有*问题，但之前的错误可能会掩盖随后的错误。例如，一个 MultiPolygon 要求其所有元素都有效且不重叠。如果其中一个多边形无效，我们不能保证其“重叠”检查的正确性，因为它假设输入是有效的。因此，在尝试纠正任何验证错误后，您应该重新验证。
    fn validation_errors(&self) -> Vec<Self::Error> {
        let mut validation_errors = Vec::new();

        self.visit_validation(Box::new(|problem| {
            validation_errors.push(problem);
            Ok::<(), Self::Error>(())
        }))
        .expect("no errors are returned");

        validation_errors
    }

    /// 返回几何体无效的第一个原因。
    fn check_validation(&self) -> Result<(), Self::Error> {
        self.visit_validation(Box::new(Err))
    }

    /// 访问几何体的验证。
    ///
    /// 闭包 `handle_validation_error` 会在每个验证错误时被调用。
    fn visit_validation<T>(
        &self,
        handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T>;
}

/// [`Polygon`](crate::Polygon) 中环的角色。
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RingRole {
    Exterior,
    Interior(usize),
}

impl fmt::Display for RingRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RingRole::Exterior => write!(f, "exterior ring"),
            RingRole::Interior(idx) => write!(f, "interior ring at index {}", idx),
        }
    }
}

/// 多几何体中问题的位置，从0开始。
#[derive(Debug, PartialEq, Clone)]
pub struct GeometryIndex(pub usize);

/// 几何体中坐标的索引
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CoordIndex(pub usize);

#[cfg(test)]
pub(crate) use test_macros::*;

#[cfg(test)]
mod test_macros {
    macro_rules! assert_valid {
        ($to_validate:expr) => {
            assert!(
                $to_validate.is_valid(),
                "Validation errors: {:?}",
                $to_validate.validation_errors()
            );
        };
    }
    pub(crate) use assert_valid;

    macro_rules! assert_validation_errors {
        ($to_validate:expr, $errors:expr) => {
            assert!(!$to_validate.is_valid());
            assert!(
                !$errors.is_empty(),
                "Use `assert_valid!` instead to verify there are no errors."
            );
            assert_eq!($errors, $to_validate.validation_errors());
        };
    }
    pub(crate) use assert_validation_errors;
}
