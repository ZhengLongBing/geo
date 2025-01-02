use crate::{Coord, GeoFloat, Line};
use geo_types::coord;

use crate::BoundingRect;
use crate::Intersects;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LineIntersection<F: GeoFloat> {
    /// 直线在单个点上相交
    SinglePoint {
        intersection: Coord<F>,
        /// 对于在单个点上相交的直线，该点可以是任一点或每条直线的内部。
        /// 如果该点位于两条直线的内部，我们称之为“适当”的交点。
        ///
        /// # 注意
        ///
        /// 由于大多数浮点数据类型的有限精度，即使两条直线的所有端点都是不同点，
        /// 计算出的交点可能会被固定到其中一个端点上。在这种情况下，此字段仍然设置为 `true`。
        /// 请参阅测试案例： `test_central_endpoint_heuristic_failure_1` 以了解此类示例。
        is_proper: bool,
    },

    /// 重叠的直线在一个线段上相交
    Collinear { intersection: Line<F> },
}

impl<F: GeoFloat> LineIntersection<F> {
    pub fn is_proper(&self) -> bool {
        match self {
            Self::Collinear { .. } => false,
            Self::SinglePoint { is_proper, .. } => *is_proper,
        }
    }
}

/// 返回两个 [`Lines`](Line) 之间的交点。
///
/// 直线可以在一个点上相交，或对于重合的直线，在一个直线上相交。有关结果的更多详细信息，请参阅 [`LineIntersection`]。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
/// use geo::{Line, Coord};
/// use geo::line_intersection::{line_intersection, LineIntersection};
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 0.0, y: 5.0}, coord! { x: 5.0, y: 0.0 } );
/// let expected = LineIntersection::SinglePoint { intersection: coord! { x: 2.5, y: 2.5 }, is_proper: true };
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 0.0, y: 1.0}, coord! { x: 5.0, y: 6.0 } );
/// assert_eq!(line_intersection(line_1, line_2), None);
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 5.0, y: 5.0}, coord! { x: 5.0, y: 0.0 } );
/// let expected = LineIntersection::SinglePoint { intersection: coord! { x: 5.0, y: 5.0 }, is_proper: false };
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 3.0, y: 3.0}, coord! { x: 6.0, y: 6.0 } );
/// let expected = LineIntersection::Collinear { intersection: Line::new(coord! { x: 3.0, y: 3.0 }, coord! { x: 5.0, y: 5.0 })};
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
/// ```
/// 强烈受启发于并旨在产生与 [JTS的 RobustLineIntersector](https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/algorithm/RobustLineIntersector.java#L26) 相同的结果。
pub fn line_intersection<F>(p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>>
where
    F: GeoFloat,
{
    if !p.bounding_rect().intersects(&q.bounding_rect()) {
        return None;
    }

    use crate::kernels::{Kernel, Orientation::*, RobustKernel};
    let p_q1 = RobustKernel::orient2d(p.start, p.end, q.start);
    let p_q2 = RobustKernel::orient2d(p.start, p.end, q.end);
    if matches!(
        (p_q1, p_q2),
        (Clockwise, Clockwise) | (CounterClockwise, CounterClockwise)
    ) {
        return None;
    }

    let q_p1 = RobustKernel::orient2d(q.start, q.end, p.start);
    let q_p2 = RobustKernel::orient2d(q.start, q.end, p.end);
    if matches!(
        (q_p1, q_p2),
        (Clockwise, Clockwise) | (CounterClockwise, CounterClockwise)
    ) {
        return None;
    }

    if matches!(
        (p_q1, p_q2, q_p1, q_p2),
        (Collinear, Collinear, Collinear, Collinear)
    ) {
        return collinear_intersection(p, q);
    }

    // 此时我们知道只有一个交点（因为两条线不共线）。
    //
    // 检查两个终点是否相交。如果是，则复制终点作为交点。
    // 复制该点而不是计算它可确保该点具有精确值，这对于稳健性很重要。
    // 只需检查其他线段上的端点即可，因为此时我们知道输入直线必须相交。
    if p_q1 == Collinear || p_q2 == Collinear || q_p1 == Collinear || q_p2 == Collinear {
        // 检查两个相等的端点。
        // 这不是通过下面的方位测试进行的，而是为了提高稳健性。
        //
        // [方位测试不一致的示例如下（其中
        // 真正的交点在共享端点
        // 点 (19.850257749638203 46.29709338043669)
        //
        // 线段 ( 19.850257749638203 46.29709338043669, 20.31970698357233 46.76654261437082 )
        // 和
        // 线段 ( -48.51001596420236 -22.063180333403878, 19.850257749638203 46.29709338043669 )
        //
        // 此前产生错误结果： (20.31970698357233, 46.76654261437082, NaN)

        let intersection: Coord<F>;
        // 对于这个过度古板的 Clippy 提高的假阳性警告 https://github.com/rust-lang/rust-clippy/issues/6747
        #[allow(clippy::suspicious_operation_groupings)]
        if p.start == q.start || p.start == q.end {
            intersection = p.start;
        } else if p.end == q.start || p.end == q.end {
            intersection = p.end;
            // 现在检查是否有任一端点位于另一线段内部。
        } else if p_q1 == Collinear {
            intersection = q.start;
        } else if p_q2 == Collinear {
            intersection = q.end;
        } else if q_p1 == Collinear {
            intersection = p.start;
        } else {
            assert_eq!(q_p2, Collinear);
            intersection = p.end;
        }
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        })
    } else {
        let intersection = proper_intersection(p, q);
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: true,
        })
    }
}

fn collinear_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>> {
    fn collinear<F: GeoFloat>(intersection: Line<F>) -> LineIntersection<F> {
        LineIntersection::Collinear { intersection }
    }

    fn improper<F: GeoFloat>(intersection: Coord<F>) -> LineIntersection<F> {
        LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        }
    }

    let p_bounds = p.bounding_rect();
    let q_bounds = q.bounding_rect();
    Some(
        match (
            p_bounds.intersects(&q.start),
            p_bounds.intersects(&q.end),
            q_bounds.intersects(&p.start),
            q_bounds.intersects(&p.end),
        ) {
            (true, true, _, _) => collinear(q),
            (_, _, true, true) => collinear(p),
            (true, false, true, false) if q.start == p.start => improper(q.start),
            (true, _, true, _) => collinear(Line::new(q.start, p.start)),
            (true, false, false, true) if q.start == p.end => improper(q.start),
            (true, _, _, true) => collinear(Line::new(q.start, p.end)),
            (false, true, true, false) if q.end == p.start => improper(q.end),
            (_, true, true, _) => collinear(Line::new(q.end, p.start)),
            (false, true, false, true) if q.end == p.end => improper(q.end),
            (_, true, _, true) => collinear(Line::new(q.end, p.end)),
            _ => return None,
        },
    )
}

/// 找到线段 P 和 Q 的端点，它最接近另一个线段。
/// 这是在条件不佳的情况下（如两条线段几乎重合，或一条线段的端点几乎在另一条线段上）合理的真交点替代。
///
/// 这替代了较旧的 CentralEndpoint 启发式方法，该方法在某些情况下选择了错误的端点，
/// 这些情况下，线段的斜率非常不同且一个端点几乎位于另一线段上。
///
/// `returns`离另一个线段最近的端点
fn nearest_endpoint<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Coord<F> {
    use geo_types::private_utils::point_line_euclidean_distance;

    let mut nearest_pt = p.start;
    let mut min_dist = point_line_euclidean_distance(p.start, q);

    let dist = point_line_euclidean_distance(p.end, q);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = p.end;
    }
    let dist = point_line_euclidean_distance(q.start, p);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = q.start;
    }
    let dist = point_line_euclidean_distance(q.end, p);
    if dist < min_dist {
        nearest_pt = q.end;
    }
    nearest_pt
}

fn raw_line_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Option<Coord<F>> {
    let p_min_x = p.start.x.min(p.end.x);
    let p_min_y = p.start.y.min(p.end.y);
    let p_max_x = p.start.x.max(p.end.x);
    let p_max_y = p.start.y.max(p.end.y);

    let q_min_x = q.start.x.min(q.end.x);
    let q_min_y = q.start.y.min(q.end.y);
    let q_max_x = q.start.x.max(q.end.x);
    let q_max_y = q.start.y.max(q.end.y);

    let int_min_x = p_min_x.max(q_min_x);
    let int_max_x = p_max_x.min(q_max_x);
    let int_min_y = p_min_y.max(q_min_y);
    let int_max_y = p_max_y.min(q_max_y);

    let two = F::one() + F::one();
    let mid_x = (int_min_x + int_max_x) / two;
    let mid_y = (int_min_y + int_max_y) / two;

    // 通过减去中点调节纵坐标值
    let p1x = p.start.x - mid_x;
    let p1y = p.start.y - mid_y;
    let p2x = p.end.x - mid_x;
    let p2y = p.end.y - mid_y;
    let q1x = q.start.x - mid_x;
    let q1y = q.start.y - mid_y;
    let q2x = q.end.x - mid_x;
    let q2y = q.end.y - mid_y;

    // 利用齐次坐标公式展开计算
    let px = p1y - p2y;
    let py = p2x - p1x;
    let pw = p1x * p2y - p2x * p1y;

    let qx = q1y - q2y;
    let qy = q2x - q1x;
    let qw = q1x * q2y - q2x * q1y;

    let xw = py * qw - qy * pw;
    let yw = qx * pw - px * qw;
    let w = px * qy - qx * py;

    let x_int = xw / w;
    let y_int = yw / w;

    // 检查平行直线
    if (x_int.is_nan() || x_int.is_infinite()) || (y_int.is_nan() || y_int.is_infinite()) {
        None
    } else {
        // 去调节交点
        Some(coord! {
            x: x_int + mid_x,
            y: y_int + mid_y,
        })
    }
}

/// 此方法计算交点的实际值。
/// 为了从交叉计算中获得最大精度，坐标通过减去最小纵坐标值（绝对值）来归一化。
/// 这具有从计算中删除共同有效数字以保持更多精度位的效果。
fn proper_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Coord<F> {
    // 使用齐次坐标计算线段交点。
    // 圆误可能导致原始计算失败，
    // （通常是因为线段近似平行）。
    // 若发生这种情况，则计算一个合理的近似值。
    let mut int_pt = raw_line_intersection(p, q).unwrap_or_else(|| nearest_endpoint(p, q));

    // 注意：此时，JTS 确实进行了 `Envelope::contains(coord)` 检查，但令人困惑的是，
    // JTS 中的 Envelope::contains(coord) 实际上是 *交叉* 检查，而不是真正的 SFS
    // `contains`，因为它包括矩形的边界。
    if !(p.bounding_rect().intersects(&int_pt) && q.bounding_rect().intersects(&int_pt)) {
        // 计算一个更安全的结果
        // 复制坐标，因为它可能会在稍后进行舍入
        int_pt = nearest_endpoint(p, q);
    }
    int_pt
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::coord;

    /// 基于 JTS 测试 `testCentralEndpointHeuristicFailure`
    /// > 以下案例在使用 CentralEndpointIntersector 启发式方法时失败。
    /// > 这是因为一个线段相对于另一个具有显著角度，
    /// > 仅有一个端点接近于另一线段。
    /// > CE 启发式选择了错误的端点返回。
    /// > 修复方法是使用一种新的启发式方法，从 4 个端点中
    /// > 选择最接近另一线段的一个。
    /// > 这种方法在所有已知故障情况下有效。
    #[test]
    fn test_central_endpoint_heuristic_failure_1() {
        let line_1 = Line::new(
            coord! {
                x: 163.81867067,
                y: -211.31840378,
            },
            coord! {
                x: 165.9174252,
                y: -214.1665075,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 2.84139601,
                y: -57.95412726,
            },
            coord! {
                x: 469.59990601,
                y: -502.63851732,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 163.81867067,
                y: -211.31840378,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testCentralEndpointHeuristicFailure2`
    /// > Tomas Fa 的测试 - JTS 列表 6/13/2012
    /// >
    /// > 使用原始 JTS DeVillers 定位测试失败。
    /// > 使用 DD 和 Shewchuk 方向测试成功
    #[test]
    fn test_central_endpoint_heuristic_failure_2() {
        let line_1 = Line::new(
            coord! {
                x: -58.00593335955,
                y: -1.43739086465,
            },
            coord! {
                x: -513.86101637525,
                y: -457.29247388035,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: -215.22279674875,
                y: -158.65425425385,
            },
            coord! {
                x: -218.1208801283,
                y: -160.68343590235,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: -215.22279674875,
                y: -158.65425425385,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testTomasFa_1`
    /// > Tomas Fa 的测试 - JTS 列表 6/13/2012
    /// >
    /// > 使用原始 JTS DeVillers 定位测试失败。
    /// > 使用 DD 和 Shewchuk 方向测试成功
    #[test]
    fn test_tomas_fa_1() {
        let line_1 = Line::new(coord! { x: -42.0, y: 163.2 }, coord! { x: 21.2, y: 265.2 });
        let line_2 = Line::new(coord! { x: -26.2, y: 188.7 }, coord! { x: 37.0, y: 290.7 });
        let actual = line_intersection(line_1, line_2);
        let expected = None;
        assert_eq!(actual, expected);
    }

    /// 基于 JTS 测试 `testTomasFa_2`
    ///
    /// > Tomas Fa 的测试 - JTS 列表 6/13/2012
    /// >
    /// > 使用原始 JTS DeVillers 定位测试失败。
    #[test]
    fn test_tomas_fa_2() {
        let line_1 = Line::new(coord! { x: -5.9, y: 163.1 }, coord! { x: 76.1, y: 250.7 });
        let line_2 = Line::new(coord! { x: 14.6, y: 185.0 }, coord! { x: 96.6, y: 272.6 });
        let actual = line_intersection(line_1, line_2);
        let expected = None;
        assert_eq!(actual, expected);
    }

    /// 基于 JTS 测试 `testLeduc_1`
    ///
    /// > 涉及两条非几乎平行线的测试。
    /// > 似乎不会导致基本直线交叉算法出现问题。
    #[test]
    fn test_leduc_1() {
        let line_1 = Line::new(
            coord! {
                x: 305690.0434123494,
                y: 254176.46578338774,
            },
            coord! {
                x: 305601.9999843455,
                y: 254243.19999846347,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 305689.6153764265,
                y: 254177.33102743194,
            },
            coord! {
                x: 305692.4999844298,
                y: 254171.4999983967,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 305690.0434123494,
                y: 254176.46578338774,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testGEOS_1()`
    ///
    /// > 来自 strk 的测试，在 GEOS 中不良（2009-04-14）。
    #[test]
    fn test_geos_1() {
        let line_1 = Line::new(
            coord! {
                x: 588750.7429703881,
                y: 4518950.493668233,
            },
            coord! {
                x: 588748.2060409798,
                y: 4518933.9452804085,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 588745.824857241,
                y: 4518940.742239175,
            },
            coord! {
                x: 588748.2060437313,
                y: 4518933.9452791475,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 588748.2060416829,
                y: 4518933.945284994,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testGEOS_2()`
    ///
    /// > 来自 strk 的测试，在 GEOS 中不良（2009-04-14）。
    #[test]
    fn test_geos_2() {
        let line_1 = Line::new(
            coord! {
                x: 588743.626135934,
                y: 4518924.610969561,
            },
            coord! {
                x: 588732.2822865889,
                y: 4518925.4314047815,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 588739.1191384895,
                y: 4518927.235700594,
            },
            coord! {
                x: 588731.7854614238,
                y: 4518924.578370095,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 588733.8306132929,
                y: 4518925.319423238,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testDaveSkeaCase()`
    ///
    /// > 这曾经是个故障案例（异常），但现在似乎已修复。
    /// > 可能是规范化修复了该问题？
    #[test]
    fn test_dave_skea_case() {
        let line_1 = Line::new(
            coord! {
                x: 2089426.5233462777,
                y: 1180182.387733969,
            },
            coord! {
                x: 2085646.6891757075,
                y: 1195618.7333999649,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 1889281.8148903656,
                y: 1997547.0560044837,
            },
            coord! {
                x: 2259977.3672236,
                y: 483675.17050843034,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 2087536.6062609926,
                y: 1187900.560566967,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// 基于 JTS 测试 `testCmp5CaseWKT()`
    ///
    /// > 使用 HCoordinate 方法在包络之外。
    #[test]
    fn test_cmp_5_cask_wkt() {
        let line_1 = Line::new(
            coord! {
                x: 4348433.262114629,
                y: 5552595.478385733,
            },
            coord! {
                x: 4348440.849387404,
                y: 5552599.272022122,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 4348433.26211463,
                y: 5552595.47838573,
            },
            coord! {
                x: 4348440.8493874,
                y: 5552599.27202212,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 4348440.8493874,
                y: 5552599.27202212,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }
}
