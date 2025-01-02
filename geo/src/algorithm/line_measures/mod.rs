//! 线段测量，如[`Bearing`]和[`Distance`]，适用于各种度量空间，如[`Euclidean`]，[`Haversine`]，[`Geodesic`]和[`Rhumb`]。

// 包含方位角计算模块
mod bearing;
pub use bearing::Bearing;

// 包含目的地计算模块
mod destination;
pub use destination::Destination;

// 包含距离计算模块
mod distance;
pub use distance::Distance;

// 包含插值点计算模块
mod interpolate_point;
pub use interpolate_point::InterpolatePoint;

// 包含长度计算模块
mod length;
pub use length::Length;

// 包含加密线段模块
mod densify;
pub use densify::Densify;

// 包含度量空间相关模块
pub mod metric_spaces;
pub use metric_spaces::{Euclidean, Geodesic, Haversine, Rhumb};
