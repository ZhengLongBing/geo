use super::{GeometryIndex, InvalidPoint, Validation};
use crate::{GeoFloat, MultiPoint};

use std::fmt;

/// 当[`MultiPoint`]中的每个[`Point`](crate::Point)都是有效时，该[`MultiPoint`]是有效的。
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidMultiPoint {
    /// 指示哪个元素无效以及无效的具体原因。
    InvalidPoint(GeometryIndex, InvalidPoint),
}

impl fmt::Display for InvalidMultiPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidMultiPoint::InvalidPoint(idx, err) => {
                write!(f, "索引为 {} 的点无效: {}", idx.0, err)
            }
        }
    }
}

impl std::error::Error for InvalidMultiPoint {}

impl<F: GeoFloat> Validation for MultiPoint<F> {
    type Error = InvalidMultiPoint;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        for (i, point) in self.0.iter().enumerate() {
            point.visit_validation(Box::new(&mut |invalid_point| {
                let err = InvalidMultiPoint::InvalidPoint(GeometryIndex(i), invalid_point);
                handle_validation_error(err)
            }))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::{geometry::*, wkt};

    #[test]
    fn test_multipoint_valid() {
        let mp = wkt!(MULTIPOINT(0. 0.,1. 1.));
        assert_valid!(&mp);
    }

    #[test]
    fn test_multipoint_invalid() {
        let mp = MultiPoint(vec![
            Point::new(0., f64::INFINITY),
            Point::new(f64::NAN, 1.),
        ]);
        assert_validation_errors!(
            &mp,
            vec![
                InvalidMultiPoint::InvalidPoint(GeometryIndex(0), InvalidPoint::NonFiniteCoord),
                InvalidMultiPoint::InvalidPoint(GeometryIndex(1), InvalidPoint::NonFiniteCoord)
            ]
        );
    }
}
