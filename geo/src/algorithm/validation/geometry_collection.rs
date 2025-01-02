use super::{GeometryIndex, InvalidGeometry, Validation};
use crate::{GeoFloat, GeometryCollection};

use std::fmt;

/// 只有当所有元素均有效时，[`GeometryCollection`] 才是有效的。
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidGeometryCollection {
    /// 哪个元素无效，以及无效的原因。
    InvalidGeometry(GeometryIndex, Box<InvalidGeometry>),
}

impl std::error::Error for InvalidGeometryCollection {}

impl fmt::Display for InvalidGeometryCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidGeometryCollection::InvalidGeometry(idx, err) => {
                write!(f, "索引为 {} 的几何体无效：{}", idx.0, err)
            }
        }
    }
}

impl<F: GeoFloat> Validation for GeometryCollection<F> {
    type Error = InvalidGeometryCollection;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 遍历所有几何体，收集无效原因
        // 并更改问题位置以反映 GeometryCollection
        for (i, geometry) in self.0.iter().enumerate() {
            geometry.visit_validation(Box::new(&mut |geometry_err| {
                let err = InvalidGeometryCollection::InvalidGeometry(
                    GeometryIndex(i),
                    Box::new(geometry_err),
                );
                handle_validation_error(err)
            }))?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_validation_errors, InvalidLineString};
    use crate::wkt;

    #[test]
    fn test_geometrycollection_contain_invalid_element() {
        let gc = wkt!(
            GEOMETRYCOLLECTION(
                POINT(0. 0.),
                LINESTRING(0. 0.,1. 1.),
                LINESTRING(0. 0.,0. 0.)
            )
        );
        assert_validation_errors!(
            gc,
            vec![InvalidGeometryCollection::InvalidGeometry(
                GeometryIndex(2),
                Box::new(InvalidGeometry::InvalidLineString(
                    InvalidLineString::TooFewPoints
                )),
            )]
        );
    }

    #[test]
    fn test_display() {
        let gc = wkt!(
            GEOMETRYCOLLECTION(
                POINT(0. 0.),
                LINESTRING(0. 0.,1. 1.),
                LINESTRING(0. 0.,0. 0.),
                POLYGON(
                    (0. 0., 1. 1., 1. 0., 0. 0.),
                    (0. 0., 1. 1., 1. 0., 0. 0.)
                )
            )
        );
        let errors = gc.validation_errors();
        assert_eq!(
            errors[0].to_string(),
            "geometry at index 2 is invalid: line string must have at least 2 distinct points"
        );

        assert_eq!(
            errors[1].to_string(),
            "geometry at index 3 is invalid: interior ring at index 0 is not contained within the polygon's exterior"
        );

        assert_eq!(
            errors[2].to_string(),
            "geometry at index 3 is invalid: exterior ring and interior ring at index 0 intersect on a line"
        );
    }
}
