/// 用于计算各种谓词的内核
pub mod kernels;
pub use kernels::{Kernel, Orientation};

/// 计算几何图形表面的面积。
pub mod area;
pub use area::Area;

/// 布尔运算，如两个几何图形的并集、异或或差值。
pub mod bool_ops;
pub use bool_ops::{unary_union, BooleanOps, OpType};

/// 计算几何图形的边界矩形。
pub mod bounding_rect;
pub use bounding_rect::BoundingRect;

/// 计算几何图形的最小旋转矩形。
pub mod minimum_rotated_rect;
pub use minimum_rotated_rect::MinimumRotatedRect;

/// 计算几何对象的中心点。
pub mod centroid;
pub use centroid::Centroid;

/// 使用Chaikins算法平滑`LineString`、`Polygon`、`MultiLineString`和`MultiPolygon`。
pub mod chaikin_smoothing;
pub use chaikin_smoothing::ChaikinSmoothing;

/// 计算几何图形的签名近似测地面积。
pub mod chamberlain_duquette_area;
pub use chamberlain_duquette_area::ChamberlainDuquetteArea;

/// 计算几何图形与某个输入点之间的最近点。
pub mod closest_point;
pub use closest_point::ClosestPoint;

/// 计算几何图形的凹壳。
pub mod concave_hull;
pub use concave_hull::ConcaveHull;

/// 判断几何图形`A`是否完全包围几何图形`B`。
pub mod contains;
pub use contains::Contains;

/// 转换几何图形的坐标值类型。
pub mod convert;
pub use convert::{Convert, TryConvert};

/// 在弧度和度之间转换坐标角度单位。
pub mod convert_angle_unit;
pub use convert_angle_unit::{ToDegrees, ToRadians};

/// 计算几何图形的凸壳。
pub mod convex_hull;
pub use convex_hull::ConvexHull;

/// 跟踪距离
pub mod cross_track_distance;
pub use cross_track_distance::CrossTrackDistance;

/// 判断一个坐标是否位于几何图形的内部、外部或边界上。
pub mod coordinate_position;
pub use coordinate_position::CoordinatePosition;

/// 迭代几何图形的坐标。
pub mod coords_iter;
pub use coords_iter::CoordsIter;

/// 使球面几何组件密集化
pub mod densify_haversine;
#[allow(deprecated)]
pub use densify_haversine::DensifyHaversine;

/// 几何图形及其边界的维度，基于 OGC-SFA。
pub mod dimensions;
pub use dimensions::HasDimensions;

/// 计算两个`几何图形`之间的最小欧氏距离。
pub mod euclidean_distance;
#[allow(deprecated)]
pub use euclidean_distance::EuclideanDistance;

/// 计算两个`几何图形`间平面线的长度。
pub mod euclidean_length;
#[allow(deprecated)]
pub use euclidean_length::EuclideanLength;

/// 计算几何体的极值坐标和索引。
pub mod extremes;
pub use extremes::Extremes;

/// 计算两个`线串`之间的Fréchet距离。
pub mod frechet_distance;
pub use frechet_distance::FrechetDistance;

/// 计算到另一`点`的测地线方位角。
pub mod geodesic_bearing;
pub use geodesic_bearing::GeodesicBearing;

/// 使用测地线上的距离和方位角返回一个新点。
pub mod geodesic_destination;
#[allow(deprecated)]
pub use geodesic_destination::GeodesicDestination;

/// 计算两个`点`之间的测地线距离。
pub mod geodesic_distance;
#[allow(deprecated)]
pub use geodesic_distance::GeodesicDistance;

/// 计算多边形的测地面积和周长。
pub mod geodesic_area;
pub use geodesic_area::GeodesicArea;

/// 计算位于两个`点`之间的测地弧的一个新`点`。
pub mod geodesic_intermediate;
#[allow(deprecated)]
pub use geodesic_intermediate::GeodesicIntermediate;

/// 计算线的测地长度。
pub mod geodesic_length;
#[allow(deprecated)]
pub use geodesic_length::GeodesicLength;

/// 计算两个几何体之间的Hausdorff距离。
pub mod hausdorff_distance;
pub use hausdorff_distance::HausdorffDistance;

/// 计算到另一`点`的方位角，以度为单位。
pub mod haversine_bearing;
#[allow(deprecated)]
pub use haversine_bearing::HaversineBearing;

/// 给定距离和方位角，计算目标`点`。
pub mod haversine_destination;
#[allow(deprecated)]
pub use haversine_destination::HaversineDestination;

/// 计算两个`几何体`之间的 Haversine 距离。
pub mod haversine_distance;
#[allow(deprecated)]
pub use haversine_distance::HaversineDistance;

/// 计算位于两个`点`之间的大圆弧上的新`点`。
pub mod haversine_intermediate;
#[allow(deprecated)]
pub use haversine_intermediate::HaversineIntermediate;

/// 计算线条的 Haversine 长度。
pub mod haversine_length;
#[allow(deprecated)]
pub use haversine_length::HaversineLength;

/// 计算给定点在大圆弧几何体上的最近点。
pub mod haversine_closest_point;
pub use haversine_closest_point::HaversineClosestPoint;

/// 计算`几何体`内部的代表性`点`
pub mod interior_point;
pub use interior_point::InteriorPoint;

/// 确定`几何体`A是否与`几何体`B相交。
pub mod intersects;
pub use intersects::Intersects;

/// 确定一个`线串`是否为凸的。
pub mod is_convex;
pub use is_convex::IsConvex;

/// 使用k近邻算法计算凹壳
pub mod k_nearest_concave_hull;
pub use k_nearest_concave_hull::KNearestConcaveHull;

/// 沿着`线`或`线串`插入一个点。
pub mod line_interpolate_point;
pub use line_interpolate_point::LineInterpolatePoint;

/// 计算两条线的交点。
pub mod line_intersection;
pub use line_intersection::LineIntersection;

/// 定位`线`或`线串`上的一个点。
pub mod line_locate_point;
pub use line_locate_point::LineLocatePoint;

/// 在几何体中迭代线。
pub mod lines_iter;
pub use lines_iter::LinesIter;

/// 线度量相关模块和对外接口，包括欧氏空间、测地空间及Haversine、Rhumb测地函数的接口。
pub mod line_measures;
pub use line_measures::metric_spaces::{Euclidean, Geodesic, Haversine, Rhumb};
pub use line_measures::{Bearing, Densify, Destination, Distance, InterpolatePoint, Length};

/// 将`线串`拆分为n段
pub mod linestring_segment;
pub use linestring_segment::{LineStringSegmentize, LineStringSegmentizeHaversine};

/// 对`几何体`的所有`坐标`应用一个函数。
pub mod map_coords;
pub use map_coords::{MapCoords, MapCoordsInPlace};

/// 定向化`多边形`的外部和内部环。
pub mod orient;
pub use orient::Orient;

/// 使用当前稳定版本的 [PROJ](http://proj.org) 进行坐标投影和转换。
#[cfg(feature = "use-proj")]
pub mod proj;

/// 基于 DE-9IM 关联两个几何形状
pub mod relate;
pub use relate::Relate;

/// 移除（连续的）重复点
pub mod remove_repeated_points;
pub use remove_repeated_points::RemoveRepeatedPoints;

/// 根据给定的角度旋转`几何体`。
pub mod rotate;
pub use rotate::Rotate;

/// 按比例放大或缩小`几何体`
pub mod scale;
pub use scale::Scale;

/// 通过在x和y维度上剪切它以使`几何体`倾斜
pub mod skew;
pub use skew::Skew;

/// 可组合仿射操作，例如旋转，缩放，倾斜和翻译
pub mod affine_ops;
pub use affine_ops::{AffineOps, AffineTransform};

/// 使用 Ramer-Douglas-Peucker 算法简化`几何体`。
pub mod simplify;
pub use simplify::{Simplify, SimplifyIdx};

/// 使用 Visvalingam-Whyatt 算法对`几何体`进行简化。包括拓扑保持的变体。
pub mod simplify_vw;
pub use simplify_vw::{SimplifyVw, SimplifyVwIdx, SimplifyVwPreserve};

/// 将邻边三角形缝合在一起。与通过 BooleanOps 结合三角形的替代方法。
#[allow(dead_code)]
pub(crate) mod stitch;
pub use stitch::StitchTriangles;

/// 使用PROJ转换几何体。
#[cfg(feature = "use-proj")]
pub mod transform;
#[cfg(feature = "use-proj")]
pub use transform::Transform;

/// 沿给定偏移量平移`几何体`。
pub mod translate;
pub use translate::Translate;

/// 使用[耳切算法](https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf)对多边形进行三角化。
///
/// 需要 `"earcutr"` 功能。
#[cfg(feature = "earcutr")]
pub mod triangulate_earcut;
#[cfg(feature = "earcutr")]
pub use triangulate_earcut::TriangulateEarcut;

/// 使用（无约束的或约束的）[Delaunay三角化](https://en.wikipedia.org/wiki/Delaunay_triangulation)算法对多边形进行三角化。
#[cfg(feature = "spade")]
pub mod triangulate_spade;
#[cfg(feature = "spade")]
pub use triangulate_spade::TriangulateSpade;

/// 二维坐标的向量操作
mod vector_ops;
pub use vector_ops::Vector2DOps;

/// 计算两个`点`之间的 Vincenty 距离。
pub mod vincenty_distance;
pub use vincenty_distance::VincentyDistance;

/// 计算`线串`的 Vincenty 长度。
pub mod vincenty_length;
pub use vincenty_length::VincentyLength;

/// 计算及处理`线串`的环绕顺序。
pub mod winding_order;
pub use winding_order::Winding;

/// 判断`几何体`A是否被完全包含在`几何体`B之内。
pub mod within;
pub use within::Within;

/// 平面扫描算法及相关工具
pub mod sweep;

/// 使用 [LOF](https://en.wikipedia.org/wiki/Local_outlier_factor) 检测一组点中的离群值
pub mod outlier_detection;

pub use outlier_detection::OutlierDetection;

/// 单调多边形细分
pub mod monotone;
pub use monotone::{monotone_subdivision, MonoPoly, MonotonicPolygons};

/// 航线相关算法和工具
pub mod rhumb;
#[allow(deprecated)]
pub use rhumb::{RhumbBearing, RhumbDestination, RhumbDistance, RhumbIntermediate, RhumbLength};

/// 验证模块和对外接口
pub mod validation;
pub use validation::Validation;
