use super::{GeometryIndex, InvalidPolygon, Validation};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::{GeoFloat, MultiPolygon, Relate};

use std::fmt;

/// 一个 [`MultiPolygon`] 是有效的，如果：
/// - [x] 它的所有多边形都是有效的，
/// - [x] 元素之间不重叠（即它们的内部不相交）
/// - [x] 元素只能在点处接触
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidMultiPolygon {
    /// 对于一个 [`MultiPolygon`] 来说，要有效其中的每个 [`Polygon`](crate::Polygon) 必须是有效的。
    InvalidPolygon(GeometryIndex, InvalidPolygon),
    /// 一个有效的 [`MultiPolygon`] 中的任何 [`Polygon`](crate::Polygon) 不能重叠（二维交集）
    ElementsOverlaps(GeometryIndex, GeometryIndex),
    /// 一个有效的 [`MultiPolygon`] 中的任何 [`Polygon`](crate::Polygon) 不能在一条线上接触（一维交集）
    ElementsTouchOnALine(GeometryIndex, GeometryIndex),
}

impl fmt::Display for InvalidMultiPolygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidMultiPolygon::InvalidPolygon(idx, err) => {
                write!(f, "索引为 {} 的多边形无效: {}", idx.0, err)
            }
            InvalidMultiPolygon::ElementsOverlaps(idx1, idx2) => {
                write!(f, "索引为 {} 和 {} 的多边形重叠", idx1.0, idx2.0)
            }
            InvalidMultiPolygon::ElementsTouchOnALine(idx1, idx2) => {
                write!(f, "索引为 {} 和 {} 的多边形在一条线上接触", idx1.0, idx2.0)
            }
        }
    }
}

impl std::error::Error for InvalidMultiPolygon {}

impl<F: GeoFloat> Validation for MultiPolygon<F> {
    type Error = InvalidMultiPolygon;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        for (i, polygon) in self.0.iter().enumerate() {
            // 检查每个多边形是否有效
            polygon.visit_validation(Box::new(&mut |invalid_polygon| {
                handle_validation_error(InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(i),
                    invalid_polygon,
                ))
            }))?;

            // 特殊情况 MultiPolygon: 元素不能重叠且只能在点处接触
            for (j, pol2) in self.0.iter().enumerate().skip(i + 1) {
                let im = polygon.relate(pol2);
                if im.get(CoordPos::Inside, CoordPos::Inside) == Dimensions::TwoDimensional {
                    let err =
                        InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(i), GeometryIndex(j));
                    handle_validation_error(err)?;
                }
                if im.get(CoordPos::OnBoundary, CoordPos::OnBoundary) == Dimensions::OneDimensional
                {
                    let err = InvalidMultiPolygon::ElementsTouchOnALine(
                        GeometryIndex(i),
                        GeometryIndex(j),
                    );
                    handle_validation_error(err)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::assert_validation_errors;
    use super::*;
    use crate::algorithm::validation::RingRole;
    use crate::wkt;

    #[test]
    fn test_multipolygon_invalid() {
        // 以下多边形包含两个无效的多边形
        // 它本身也是无效的，因为多边形的两个多边形不是不相交的
        // （这里它们是相同的）
        let multi_polygon = wkt!(
            MULTIPOLYGON (
                (
                    (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                    (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
                ),
                (
                    (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                    (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
                )
            )
        );
        assert_validation_errors!(
            &multi_polygon,
            vec![
                InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(0),
                    InvalidPolygon::InteriorRingNotContainedInExteriorRing(RingRole::Interior(0))
                ),
                InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(0), GeometryIndex(1)),
                InvalidMultiPolygon::ElementsTouchOnALine(GeometryIndex(0), GeometryIndex(1)),
                InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(1),
                    InvalidPolygon::InteriorRingNotContainedInExteriorRing(RingRole::Interior(0))
                ),
            ]
        );
    }
}
