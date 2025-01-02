use super::{utils, CoordIndex, RingRole, Validation};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::{GeoFloat, HasDimensions, Polygon, Relate};

use std::fmt;

/// 多边形必须遵循以下规则才能有效：
/// - [x] 多边形的边界环（外部壳环和内部孔环）必须是简单的（不交叉或自相交）。因此，多边形不能有断线、尖点或环。这意味着多边形的孔必须表示为内部环，而不是通过外部环自相交（即所谓的“反向孔”）。
/// - [x] 边界环不能交叉
/// - [x] 边界环可以在个点接触，但只能是切线接触（即不在一条线上）
/// - [x] 内部环包含在外部环中
/// - [ ] 多边形的内部是简单连接的（即环不能以分拆多边形为多部分的方式接触）
///
/// 注意：此实现未检查内部的简单连接性。
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidPolygon {
    /// 一个环必须至少有4个点才能有效。请注意，为了闭合环，第一个和最后一个点都将是相同的。
    TooFewPointsInRing(RingRole),
    /// 一个环有自相交。
    SelfIntersection(RingRole),
    /// 多边形的一个坐标是非有限的。
    NonFiniteCoord(RingRole, CoordIndex),
    /// 多边形的内部必须完全包含在其外部环内。
    InteriorRingNotContainedInExteriorRing(RingRole),
    /// 一个有效的多边形的环不能相互交叉。在这种情况下，交叉是1维的。
    IntersectingRingsOnALine(RingRole, RingRole),
    /// 一个有效的多边形的环不能相互交叉。在这种情况下，交叉是2维的。
    IntersectingRingsOnAnArea(RingRole, RingRole),
}

impl fmt::Display for InvalidPolygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidPolygon::TooFewPointsInRing(ring) => {
                write!(f, "{ring} 必须至少有3个不同的点")
            }
            InvalidPolygon::SelfIntersection(ring) => {
                write!(f, "{ring} 有自相交")
            }
            InvalidPolygon::NonFiniteCoord(ring, idx) => {
                write!(f, "{ring} 在索引 {} 处有一个非有限的坐标", idx.0)
            }
            InvalidPolygon::InteriorRingNotContainedInExteriorRing(ring) => {
                write!(f, "{ring} 未包含在多边形的外部之内")
            }
            InvalidPolygon::IntersectingRingsOnALine(ring_1, ring_2) => {
                write!(f, "{ring_1} 和 {ring_2} 在一条线上交叉")
            }
            InvalidPolygon::IntersectingRingsOnAnArea(ring_1, ring_2) => {
                write!(f, "{ring_1} 和 {ring_2} 在一个区域上交叉")
            }
        }
    }
}

impl std::error::Error for InvalidPolygon {}

// 实现对 Polygon 的验证逻辑
impl<F: GeoFloat> Validation for Polygon<F> {
    // 设置错误类型为 InvalidPolygon
    type Error = InvalidPolygon;

    // 定义验证方法
    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        // 检查多边形是否为空
        if self.is_empty() {
            return Ok(());
        }

        // 迭代外部环和内部环
        for (ring_idx, ring) in std::iter::once(self.exterior())
            .chain(self.interiors().iter())
            .enumerate()
        {
            if ring.is_empty() {
                continue;
            }
            let ring_role = if ring_idx == 0 {
                RingRole::Exterior // 外部环
            } else {
                RingRole::Interior(ring_idx - 1) // 内部环
            };

            // 执行各种检查
            if utils::check_too_few_points(ring, true) {
                handle_validation_error(InvalidPolygon::TooFewPointsInRing(ring_role))?;
            }

            if utils::linestring_has_self_intersection(ring) {
                handle_validation_error(InvalidPolygon::SelfIntersection(ring_role))?;
            }

            for (coord_idx, coord) in ring.0.iter().enumerate() {
                if utils::check_coord_is_not_finite(coord) {
                    handle_validation_error(InvalidPolygon::NonFiniteCoord(
                        ring_role,
                        CoordIndex(coord_idx),
                    ))?;
                }
            }
        }

        // 创建只有外部环的多边形
        let polygon_exterior = Polygon::new(self.exterior().clone(), vec![]);

        // 验证内部环
        for (interior_1_idx, interior_1) in self.interiors().iter().enumerate() {
            let ring_role_1 = RingRole::Interior(interior_1_idx);
            if interior_1.is_empty() {
                continue;
            }
            let exterior_vs_interior = polygon_exterior.relate(interior_1);

            // 检查内部环是否包含在外部环内
            if !exterior_vs_interior.is_contains() {
                handle_validation_error(InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                    ring_role_1,
                ))?;
            }

            // 内部环和外部环只能在一点上接触，不可线接或交叉
            if exterior_vs_interior.get(CoordPos::OnBoundary, CoordPos::Inside)
                == Dimensions::OneDimensional
            {
                handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                    RingRole::Exterior,
                    ring_role_1,
                ))?;
            }

            // 性能：考虑使用 PreparedGeometry
            let interior_1_as_poly = Polygon::new(interior_1.clone(), vec![]);

            // 验证内部环之间的交叉情况
            for (interior_2_idx, interior_2) in
                self.interiors().iter().enumerate().skip(interior_1_idx + 1)
            {
                let ring_role_2 = RingRole::Interior(interior_2_idx);
                let interior_2_as_poly = Polygon::new(interior_2.clone(), vec![]);
                let intersection_matrix = interior_1_as_poly.relate(&interior_2_as_poly);

                // 检查内部环之间是否在一个区域交叉
                if intersection_matrix.get(CoordPos::Inside, CoordPos::Inside)
                    == Dimensions::TwoDimensional
                {
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnAnArea(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
                // 检查内部环之间是否在一条线上交叉
                if intersection_matrix.get(CoordPos::OnBoundary, CoordPos::OnBoundary)
                    == Dimensions::OneDimensional
                {
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::wkt;

    #[test]
    fn test_polygon_valid() {
        // 未闭合的环会被 geo_types 自动闭合
        // 所以以下多边形应该是有效的
        let polygon = wkt!(
            POLYGON((0. 0., 1. 1., 0. 1.))
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_valid_interior_ring_touches_exterior_ring() {
        // 下列多边形包含一个在一个点上接触外部环的内部环。
        // 根据 OGC 规范，这是有效的。
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 1., 4. 4.,0. 4.,0. 0.),
                (0. 2., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_valid_interior_rings_touch_at_point() {
        // 下列多边形包含两个在一个点上接触的内部环。
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4.,0. 4.,0. 0.),
                (1. 2., 2. 1., 3. 2., 2. 3., 1. 2.),
                (3. 2., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_invalid_interior_rings_touch_at_line() {
        // 下列多边形包含两个在线上接触的内部环，这无效。
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4.,0. 4.,0. 0.),
                (1. 2., 2. 1., 3. 2., 2. 3., 1. 2.),
                (3. 2., 2. 1., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnALine(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_rings_crosses() {
        // 下列多边形包含两个交叉的内部环（它们共享一些公共区域），这无效。
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0.,  4. 4.,   0. 4.,  0. 0.),
                (1. 2., 2. 1.,  3. 2.,   2. 3.,  1. 2.),
                (2. 2., 2. 1., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_touches_exterior_ring_as_line() {
        // 下列多边形包含一个在一点上接触外部环的内部环。
        // 根据 OGC 规范，这是有效的。
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
                // 前两个点位于外部环上
                (0. 2., 0. 1., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnALine(
                RingRole::Exterior,
                RingRole::Interior(0)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_too_few_point_exterior_ring() {
        // 未闭合的环会被 geo_types 自动闭合
        // 但是这个环中仍有太少的点
        // 不能成为非空多边形
        let polygon = wkt!( POLYGON((0. 0., 1. 1.)) );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::TooFewPointsInRing(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_spike() {
        // 下列多边形包含一个尖点
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 2. 4., 2. 6., 2. 4., 0. 4., 0. 0.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_exterior_is_not_simple() {
        // 这个多边形的外部环不是简单的（即它有自相交）
        let polygon = wkt!(
            POLYGON((0. 0., 4. 0., 0. 2., 4. 2., 0. 0.))
        );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_not_fully_contained_in_exterior() {
        // 下列多边形的内部环没有完全包含在外部环内
        let polygon = wkt!(
            POLYGON (
                (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
            )
        );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                RingRole::Interior(0)
            ),]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_contained_in_interior_ring() {
        // 下列多边形包含一个位于另一个内部环中的内部环。
        let polygon_1 = wkt!(
            POLYGON(
                (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
                (1. 1.,  1. 9.,  9.  9., 9.  1., 1. 1.),
                (2. 2.,  2. 8.,  8.  8., 8.  2., 2. 2.)
            )
        );

        assert_validation_errors!(
            polygon_1,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );

        // 我们来看一下如果我们切换内部环的顺序会怎样
        // （这仍然是无效的）
        let polygon_2 = wkt!(
            POLYGON(
                (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
                (2. 2.,  2. 8.,  8.  8., 8.  2., 2. 2.),
                (1. 1.,  1. 9.,  9.  9., 9.  1., 1. 1.)
            )
        );

        assert_validation_errors!(
            polygon_2,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }
}
