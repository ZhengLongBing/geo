//! 此模块提供恒向线（亦称罗经线）几何操作。
//! 距离、目的地和方位角实现部分改编自[Turf.js](https://turfjs.org/)中的等效实现，
//! 而Turf.js又是改编自Movable Type的
//! [球面大地测量工具](https://www.movable-type.co.uk/scripts/latlong.html)。
//! Turf.js的版权归其作者所有，球面大地测量工具的版权归Chris Veness所有；两者均在MIT许可证下提供。

use crate::{point, utils::normalize_longitude, CoordFloat, Point};
use num_traits::FromPrimitive;

mod distance;
#[allow(deprecated)]
pub use distance::RhumbDistance;

mod bearing;
#[allow(deprecated)]
pub use bearing::RhumbBearing;

mod destination;
#[allow(deprecated)]
pub use destination::RhumbDestination;

mod intermediate;
#[allow(deprecated)]
pub use intermediate::RhumbIntermediate;

mod length;
#[allow(deprecated)]
pub use length::RhumbLength;

pub(crate) struct RhumbCalculations<T: CoordFloat + FromPrimitive> {
    from: Point<T>,
    to: Point<T>,
    phi1: T,
    delta_lambda: T,
    delta_phi: T,
    delta_psi: T,
}

impl<T: CoordFloat + FromPrimitive> RhumbCalculations<T> {
    pub(crate) fn new(from: &Point<T>, to: &Point<T>) -> Self {
        let pi = T::from(std::f64::consts::PI).unwrap();
        let two = T::one() + T::one();
        let four = two + two;

        let phi1 = from.y().to_radians();
        let phi2 = to.y().to_radians();
        let mut delta_lambda = (to.x() - from.x()).to_radians();
        // 如果delta_lambda超过180°，则穿越反子午线取较短的恒向线
        if delta_lambda > pi {
            delta_lambda = delta_lambda - (two * pi);
        }
        if delta_lambda < -pi {
            delta_lambda = delta_lambda + (two * pi);
        }

        let delta_psi = ((phi2 / two + pi / four).tan() / (phi1 / two + pi / four).tan()).ln();
        let delta_phi = phi2 - phi1;

        RhumbCalculations {
            from: *from,
            to: *to,
            phi1,
            delta_lambda,
            delta_phi,
            delta_psi,
        }
    }

    pub(crate) fn theta(&self) -> T {
        self.delta_lambda.atan2(self.delta_psi)
    }

    pub(crate) fn delta(&self) -> T {
        let threshold = T::from(10.0e-12).unwrap();
        let q = if self.delta_psi > threshold {
            self.delta_phi / self.delta_psi
        } else {
            self.phi1.cos()
        };

        (self.delta_phi * self.delta_phi + q * q * self.delta_lambda * self.delta_lambda).sqrt()
    }

    pub(crate) fn intermediate(&self, fraction: T) -> Point<T> {
        let delta = fraction * self.delta();
        let theta = self.theta();
        let lambda1 = self.from.x().to_radians();
        calculate_destination(delta, lambda1, self.phi1, theta)
    }

    pub(crate) fn intermediate_fill(&self, max_delta: T, include_ends: bool) -> Vec<Point<T>> {
        let theta = self.theta();
        let lambda1 = self.from.x().to_radians();

        let total_delta = self.delta();

        if total_delta <= max_delta {
            return if include_ends {
                vec![self.from, self.to]
            } else {
                vec![]
            };
        }

        let number_of_points = (total_delta / max_delta).ceil();
        let interval = T::one() / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends {
            vec![self.from]
        } else {
            vec![]
        };

        while current_step < T::one() {
            let delta = total_delta * current_step;
            let point = calculate_destination(delta, lambda1, self.phi1, theta);
            points.push(point);
            current_step = current_step + interval;
        }

        if include_ends {
            points.push(self.to);
        }

        points
    }
}

pub(crate) fn calculate_destination<T: CoordFloat + FromPrimitive>(
    delta: T,
    lambda1: T,
    phi1: T,
    theta: T,
) -> Point<T> {
    let pi = T::from(std::f64::consts::PI).unwrap();
    let two = T::one() + T::one();
    let four = two + two;
    let threshold = T::from(10.0e-12).unwrap();

    let delta_phi = delta * theta.cos();
    let mut phi2 = phi1 + delta_phi;

    // 检查超出极点的情况，并规范化纬度
    if phi2.abs() > pi / two {
        phi2 = if phi2 > T::zero() {
            pi - phi2
        } else {
            -pi - phi2
        };
    }

    let delta_psi = ((phi2 / two + pi / four).tan() / (phi1 / two + pi / four).tan()).ln();
    // 东西航向0/0导致的不良状态处理
    let q = if delta_psi.abs() > threshold {
        delta_phi / delta_psi
    } else {
        phi1.cos()
    };

    let delta_lambda = (delta * theta.sin()) / q;
    let lambda2 = lambda1 + delta_lambda;

    point! {
        x: normalize_longitude(lambda2.to_degrees()),
        y: phi2.to_degrees(),
    }
}
