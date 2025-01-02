use super::{LineIntersection, LineIntersector};
use crate::kernels::{Kernel, Orientation, RobustKernel};
use crate::{BoundingRect, Contains, Intersects};
use crate::{Coord, GeoFloat, Line, Rect};
use num_traits::Zero;

/// [LineIntersector](traits.LineIntersector) 的一个稳健版本
#[derive(Clone)]
pub(crate) struct RobustLineIntersector;

impl RobustLineIntersector {
    pub fn new() -> RobustLineIntersector {
        RobustLineIntersector
    }
}

impl<F: GeoFloat> LineIntersector<F> for RobustLineIntersector {
    fn compute_intersection(&mut self, p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>> {
        crate::line_intersection::line_intersection(p, q)
    }
}

impl RobustLineIntersector {
    /// 计算交点 p 在线段上的“边缘距离”
    ///
    /// 边缘距离是沿边缘的点的度量。
    /// 使用的度量是一个稳健且易于计算的度量函数。
    /// 它 _不_ 等于通常的欧几里得度量。
    /// 该方法依赖于边缘中的点的 x 或 y 坐标是唯一的，具体取决于边缘在横向或纵向哪个更长。
    ///
    /// 注意：对于 p 不完全位于 p1-p2 的输入，这个函数可能会产生不正确的距离
    /// （例如：p = (139,9)，p1 = (139,10)，p2 = (280,1)，返回的距离是 0.0，这是不正确的）。
    ///
    /// 我的假设是，该函数对于那些精确位于直线上的点的 _舍入_ 结果是安全的，
    /// 但对于 _截断_ 的点不安全。
    pub fn compute_edge_distance<F: GeoFloat>(intersection: Coord<F>, line: Line<F>) -> F {
        let dx = (line.end.x - line.start.x).abs();
        let dy = (line.end.y - line.start.y).abs();

        let mut dist: F;
        if intersection == line.start {
            dist = F::zero();
        } else if intersection == line.end {
            if dx > dy {
                dist = dx;
            } else {
                dist = dy;
            }
        } else {
            let intersection_dx = (intersection.x - line.start.x).abs();
            let intersection_dy = (intersection.y - line.start.y).abs();
            if dx > dy {
                dist = intersection_dx;
            } else {
                dist = intersection_dy;
            }
            // 确保非端点始终具有非零距离的修正
            if dist == F::zero() && intersection != line.start {
                dist = intersection_dx.max(intersection_dy);
            }
        }
        debug_assert!(
            !(dist == F::zero() && intersection != line.start),
            "距离计算错误"
        );
        dist
    }
}
