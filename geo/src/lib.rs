// Start of Selection
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]

//! `geo` crate 提供地理空间基本类型和算法。
//!
//! # 类型
//!
//! - **[`Coord`]**: 二维坐标。所有几何类型都由[`Coord`]组成，尽管[`Coord`]本身不是[`Geometry`]类型
//! - **[`Point`]**: 由一个[`Coord`]表示的单点
//! - **[`MultiPoint`]**: 多个[`Point`]的集合
//! - **[`Line`]**: 由两个[`Coord`]表示的线段
//! - **[`LineString`]**: 由两个或多个[`Coord`]表示的连续线段序列
//! - **[`MultiLineString`]**: 多个[`LineString`]的集合
//! - **[`Polygon`]**: 由一个[`LineString`]外环和零个或多个[`LineString`]内环表示的有界区域
//! - **[`MultiPolygon`]**: 多个[`Polygon`]的集合
//! - **[`Rect`]**: 由最小和最大[`Coord`]表示的轴对齐的有界矩形
//! - **[`Triangle`]**: 由三个[`Coord`]顶点表示的有界区域
//! - **[`GeometryCollection`]**: 多个[`Geometry`]的集合
//! - **[`Geometry`]**: 所有几何类型（除[`Coord`]外）的枚举类型
//!
//! 上述类型从[`geo-types`] crate重新导出。如果您只需要访问这些类型而不需要其他`geo`功能，可以考虑使用该crate。
//!
//! ## 语义
//!
//! 此处提供的地理空间类型旨在符合[OpenGIS简单要素访问][OGC-SFA]标准。因此，这些类型可以与其他标准的实现互操作：[JTS]、[GEOS]等。
//!
//! # 算法
//!
//! ## 度量
//!
//! 沿线度量的算法，以及线的度量方式。
//!
//! ### 度量空间
//!
//! - **[`Euclidean`]**: [Euclidean plane]使用毕达哥拉斯公式来测量距离。不适用于经纬度几何。
//! - **[`Haversine`]**: [Haversine公式]测量球体上的距离。仅适用于经纬度几何。
//! - **[`Geodesic`]**: 基于[Karney (2013)]的测地方法更准确地反映地球的形状，但比Haversine慢。仅适用于经纬度几何。
//! - **[`Rhumb`]**: [店位置线]（又称loxodrome）度量在需要保持恒定方位或方向的导航应用中可能有用。仅适用于经纬度几何。
//!
//! ### 度量空间的操作
//!
//! - **[`Distance`]**: 计算两个几何体之间的最小距离。
//! - **[`Length`]**: 计算`Line`、`LineString`或`MultiLineString`的长度。
//! - **[`Bearing`]**: 计算两点之间的方位。
//!
//! - **[`Destination`]**: 给定方位和距离，从起始点计算目的地点。
//! - **[`InterpolatePoint`]**: 沿着直线插入点。
//! - **[`Densify`]**: 向几何体中插入点，以便两个点之间从不超过`max_segment_length`。
//!
//! ### 杂项度量
//!
//! - **[`HausdorffDistance`]**: 计算“从任何一个集合中的一点到另一个集合中最近一点的距离的最大值。”（Rote, 1991）
//! - **[`VincentyDistance`]**: 使用Vincenty公式计算几何体之间的最小测地距离
//! - **[`VincentyLength`]**: 使用Vincenty公式计算几何体的测地长度
//! - **[`FrechetDistance`]**: 使用弗雷歇距离计算[`LineString`]之间的相似性
//!
//! ## 面积
//!
//! - **[`Area`]**: 计算几何体的平面区域
//! - **[`ChamberlainDuquetteArea`]**: 使用Chamberlain和Duquette（2007）在_球面上的多边形的一些算法_中提出的算法计算几何体在球体上的测地面积
//! - **[`GeodesicArea`]**: 使用Charles Karney（2013）在_测地算法_中提出的算法计算几何体在椭球体上的测地面积和周长
//!
//! ## 布尔运算
//!
//! - **[`BooleanOps`]**: 使用交集、联合、异或或差运算组合或拆分（Multi）多边形
//! - **[`unary_union`]**: 高效地联合多个[`Polygon`]或[`MultiPolygon`]。
//!
//! ## 异常值检测
//!
//! - **[`OutlierDetection`]**: 使用[LOF](https://en.wikipedia.org/wiki/Local_outlier_factor)检测一组点中的异常值
//!
//! ## 简化
//!
//! - **[`Simplify`]**: 使用Ramer-Douglas-Peucker算法简化几何体
//! - **[`SimplifyIdx`]**: 使用Ramer-Douglas-Peucker算法计算简化的几何体，返回坐标索引
//! - **[`SimplifyVw`]**: 使用Visvalingam-Whyatt算法简化几何体
//! - **[`SimplifyVwPreserve`]**: 使用Visvalingam-Whyatt算法的拓扑保存变体简化几何体
//! - **[`SimplifyVwIdx`]**: 使用Visvalingam-Whyatt算法计算简化的几何体，返回坐标索引
//!
//! ## 查询
//!
//! - **[`ClosestPoint`]**: 找到几何体上最接近给定点的点
//! - **[`HaversineClosestPoint`]**: 使用球面坐标和线为大圆弧找到几何体上最接近给定点的点
//! - **[`IsConvex`]**: 计算[`LineString`]的凸性
//! - **[`LineInterpolatePoint`]**: 生成一个在给定线段上位于给定比例的位置的点
//! - **[`LineLocatePoint`]**: 计算线段总长的一部分代表从线段到给定点最近点的位置
//! - **[`InteriorPoint`]**: 计算几何体内的一个代表点
//!
//! ## 拓扑
//!
//! - **[`Contains`]**: 计算一个几何是否包含另一个几何
//! - **[`CoordinatePosition`]**: 计算一个坐标相对几何的位置
//! - **[`HasDimensions`]**: 确定几何的维度
//! - **[`Intersects`]**: 计算一个几何是否与另一个几何相交
//! - **[`line_intersection`]**: 计算两条线之间的交点（如果有的话）
//! - **[`Relate`]**: 基于[DE-9IM](https://en.wikipedia.org/wiki/DE-9IM)语义拓扑关系两个几何
//! - **[`Within`]**: 计算一个几何是否完全位于另一个几何内
//!
//! ## 三角剖分
//!
//! - **[`TriangulateEarcut`](triangulate_earcut)**: 使用earcut算法三角剖分多边形。需要启用默认启用的`"earcutr"`功能
//!
//! ## 绕线
//!
//! - **[`Orient`]**: 对[`Polygon`]的内部和外部环应用指定的绕线[`Direction`](orient::Direction)
//! - **[`Winding`]**: 计算并操作[`LineString`]的[`WindingOrder`](winding_order::WindingOrder)
//!
//! ## 迭代
//!
//! - **[`CoordsIter`]**: 迭代几何的坐标
//! - **[`MapCoords`]**: 在几何的所有坐标上映射一个函数，返回一个新几何体
//! - **[`MapCoordsInPlace`]**: 就地在几何的所有坐标上映射一个函数
//! - **[`LinesIter`]**: 迭代几何的线条
//!
//! ## 边界
//!
//! - **[`BoundingRect`]**: 计算几何的轴对齐边界矩形
//! - **[`MinimumRotatedRect`]**: 计算几何的最小边界盒
//! - **[`ConcaveHull`]**: 计算几何的凹壳
//! - **[`ConvexHull`]**: 计算几何的凸壳
//! - **[`Extremes`]**: 计算几何的极值坐标和索引
//!
//! ## 仿射变换
//!
//! - **[`Rotate`]**: 围绕几何的质心旋转几何
//! - **[`Scale`]**: 按因子缩放几何
//! - **[`Skew`]**: 沿`x`和`y`维度倾斜几何
//! - **[`Translate`]**: 沿轴平移几何
//! - **[`AffineOps`]**: 广义可组合的仿射操作
//!
//! ## 转换
//!
//! - **[`Convert`]**: 转换（无错误）几何坐标值的数值类型
//! - **[`TryConvert`]**: 转换（可能有错误）几何坐标值的数值类型
//! - **[`ToDegrees`]**: 将给定几何体的坐标从弧度转换为角度
//! - **[`ToRadians`]**: 将给定几何体的坐标从角度转换为弧度
//!
//! ## 杂项
//!
//! - **[`Centroid`]**: 计算几何体的质心
//! - **[`ChaikinSmoothing`]**: 使用Chaikin算法平滑`LineString`、`Polygon`、`MultiLineString`和`MultiPolygon`
//! - **[`proj`]**: 使用`proj` crate投影几何体（需要启用`use-proj`功能）
//! - **[`LineStringSegmentize`]**: 将LineString分割为`n`段
//! - **[`LineStringSegmentizeHaversine`]**: 使用Haversine距离分割LineString
//! - **[`Transform`]**: 使用Proj变换几何体
//! - **[`RemoveRepeatedPoints`]**: 从几何体中移除重复的点
//! - **[`Validation`]**: 检测几何体是否结构正确。一些算法可能无法正确处理无效几何体
//!
//! # 空间索引
//!
//! `geo`几何（[`Point`]、[`Line`]、[`LineString`]、[`Polygon`]、[`MultiPolygon`]）可与[rstar](https://docs.rs/rstar/0.12.0/rstar/struct.RTree.html#usage) R*-tree crate一起使用以进行快速距离和最近邻查询。多几何可以通过迭代其成员并添加它们来添加到树中。特别请注意[`bulk_load`](https://docs.rs/rstar/0.12.0/rstar/struct.RTree.html#method.bulk_load)方法和[`GeomWithData`](https://docs.rs/rstar/0.12.0/rstar/primitives/struct.GeomWithData.html)结构的可用性。
//!
//! # 功能
//!
//! 以下可选[Cargo features]可用：
//!
//! - `earcutr`:
//!     - 启用`earcutr` crate，它提供使用earcut算法对多边形进行三角剖分
//!     - ☑ 默认启用
//! - `proj-network`:
//!     - 为[`proj` crate]启用[网络网格]支持
//!     - 启用此功能后，[需要进一步配置][proj crate file download]以使用网络网格。
//!     - ☐ 默认禁用
//! - `use-proj`:
//!     - 启用使用[`proj` crate]对`Point`几何体进行坐标转换和变换
//!     - ☐ 默认禁用
//! - `use-serde`:
//!     - 允许使用[Serde]对几何类型进行序列化和反序列化
//!     - ☐ 默认禁用
//! - `multithreading`:
//!     - 启用多线程支持（通过Rayon），并激活`geo-types`中的`multithreading`标志，支持对`Multi*`几何体的多线程迭代
//!     - ☑ 默认启用
//!
//! # 生态系统
//!
//! 在`geo` crate生态系统中，有许多兼容`geo`的crate提供了`geo` crate中未包含的功能，包括：
//!
//! * 读取和写入文件格式（例如[GeoJSON][geojson crate]、[WKT][wkt crate]、[shapefile][shapefile crate]）
//! * [经纬度解析][latlng crate]
//! * [标签放置][polylabel crate]
//! * [地理编码][geocoding crate]
//! * [以及更多...][georust website]
//!
//! [Euclidean plane]: https://en.wikipedia.org/wiki/Euclidean_plane
//! [`geo-types`]: https://crates.io/crates/geo-types
//! [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula//
//! [`proj` crate]: https://github.com/georust/proj
//! [geojson crate]: https://crates.io/crates/geojson
//! [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
//! [wkt crate]: https://crates.io/crates/wkt
//! [shapefile crate]: https://crates.io/crates/shapefile
//! [latlng crate]: https://crates.io/crates/latlon
//! [polylabel crate]: https://crates.io/crates/polylabel
//! [geocoding crate]: https://crates.io/crates/geocoding
//! [georust website]: https://georust.org
//! [Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
//! [GEOS]: https://trac.osgeo.org/geos
//! [JTS]: https://github.com/locationtech/jts
//! [network grid]: https://proj.org/usage/network.html
//! [OGC-SFA]: https://www.ogc.org/standards/sfa
//! [proj crate file download]: https://docs.rs/proj/*/proj/#grid-file-download
//! [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
//! [Serde]: https://serde.rs/

#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;

pub use crate::algorithm::*;
pub use crate::types::Closest;
use std::cmp::Ordering;

pub use crate::relate::PreparedGeometry;
pub use geo_types::{coord, line_string, point, polygon, wkt, CoordFloat, CoordNum};

pub mod geometry;
pub use geometry::*;

/// 此模块包含所有几何计算的函数。
pub mod algorithm;
mod geometry_cow;
mod types;
mod utils;
use crate::kernels::{RobustKernel, SimpleKernel};
pub(crate) use geometry_cow::GeometryCow;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[macro_use]
extern crate log;

/// 地球的平均半径，单位为米
/// 这是国际大地测量协会推荐的值：
const MEAN_EARTH_RADIUS: f64 = 6371008.8;

// 地球在赤道的半径，单位为米（由WGS-84椭球体推导出）
const EQUATORIAL_EARTH_RADIUS: f64 = 6_378_137.0;

// 地球在极地的半径，单位为米（由WGS-84椭球体推导出）
const POLAR_EARTH_RADIUS: f64 = 6_356_752.314_245;

// WGS-84椭球体的扁率 - https://en.wikipedia.org/wiki/Flattening
const EARTH_FLATTENING: f64 =
    (EQUATORIAL_EARTH_RADIUS - POLAR_EARTH_RADIUS) / EQUATORIAL_EARTH_RADIUS;

/// 一个通常用来重新导出这个crate中操作对象的特性段。通常使用`use geo::prelude::*`进行导入。
pub mod prelude {
    pub use crate::algorithm::*;
}

/// 针对geo算法使用的一种常用数值特性。
///
/// 不同的数值类型有不同的权衡。`geo`努力使用泛型让用户选择他们的数值类型。如果你正在写一个希望对`geo`支持的所有数值类型都通用的函数，你可能需要限制函数输入为`GeoFloat`。针对整数而不仅仅是浮点数的方法，参见[`GeoNum`]。
///
/// # 例子
///
/// ```
/// use geo::{GeoFloat, MultiPolygon, Polygon, Point};
///
/// // 一个明显愚蠢的方法实现，但签名显示了如何使用GeoFloat特性
/// fn farthest_from<'a, T: GeoFloat>(point: &Point<T>, polygons: &'a MultiPolygon<T>) -> Option<&'a Polygon<T>> {
///     polygons.iter().fold(None, |accum, next| {
///         match accum {
///             None => Some(next),
///             Some(farthest) => {
///                 use geo::{euclidean_distance::EuclideanDistance};
///                 if next.euclidean_distance(point) > farthest.euclidean_distance(point) {
///                     Some(next)
///                 } else {
///                     Some(farthest)
///                 }
///             }
///         }
///     })
/// }
/// ```
pub trait GeoFloat:
    GeoNum + num_traits::Float + num_traits::Signed + num_traits::Bounded + float_next_after::NextAfter
{
}
impl<T> GeoFloat for T where
    T: GeoNum
        + num_traits::Float
        + num_traits::Signed
        + num_traits::Bounded
        + float_next_after::NextAfter
{
}

/// 对整数**和**浮点数都有效的方法特性。
pub trait GeoNum: CoordNum {
    type Ker: Kernel<Self>;

    /// 返回self和other之间的排序。
    ///
    /// 对于整数，这应该和[`Ord`]一样。
    ///
    /// 对于浮点数，不像标准的浮点数部分比较，这种比较始终产生排序。
    ///
    /// 详见[f64::total_cmp](https://doc.rust-lang.org/src/core/num/f64.rs.html#1432)。
    fn total_cmp(&self, other: &Self) -> Ordering;
}

macro_rules! impl_geo_num_for_float {
    ($t: ident) => {
        impl GeoNum for $t {
            type Ker = RobustKernel;
            fn total_cmp(&self, other: &Self) -> Ordering {
                self.total_cmp(other)
            }
        }
    };
}
macro_rules! impl_geo_num_for_int {
    ($t: ident) => {
        impl GeoNum for $t {
            type Ker = SimpleKernel;
            fn total_cmp(&self, other: &Self) -> Ordering {
                self.cmp(other)
            }
        }
    };
}

// 这是我们支持的原始类型列表。
impl_geo_num_for_float!(f32);
impl_geo_num_for_float!(f64);
impl_geo_num_for_int!(i16);
impl_geo_num_for_int!(i32);
impl_geo_num_for_int!(i64);
impl_geo_num_for_int!(i128);
impl_geo_num_for_int!(isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_ord_float() {
        assert_eq!(GeoNum::total_cmp(&3.0f64, &2.0f64), Ordering::Greater);
        assert_eq!(GeoNum::total_cmp(&2.0f64, &2.0f64), Ordering::Equal);
        assert_eq!(GeoNum::total_cmp(&1.0f64, &2.0f64), Ordering::Less);
        assert_eq!(GeoNum::total_cmp(&1.0f64, &f64::NAN), Ordering::Less);
        assert_eq!(GeoNum::total_cmp(&f64::NAN, &f64::NAN), Ordering::Equal);
        assert_eq!(GeoNum::total_cmp(&f64::INFINITY, &f64::NAN), Ordering::Less);
    }

    #[test]
    fn total_ord_int() {
        assert_eq!(GeoNum::total_cmp(&3i32, &2i32), Ordering::Greater);
        assert_eq!(GeoNum::total_cmp(&2i32, &2i32), Ordering::Equal);
        assert_eq!(GeoNum::total_cmp(&1i32, &2i32), Ordering::Less);
    }

    #[test]
    fn numeric_types() {
        let _n_i16 = Point::new(1i16, 2i16);
        let _n_i32 = Point::new(1i32, 2i32);
        let _n_i64 = Point::new(1i64, 2i64);
        let _n_i128 = Point::new(1i128, 2i128);
        let _n_isize = Point::new(1isize, 2isize);
        let _n_f32 = Point::new(1.0f32, 2.0f32);
        let _n_f64 = Point::new(1.0f64, 2.0f64);
    }
}
