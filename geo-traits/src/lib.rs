//! Rust中用于地理空间矢量数据交换的基于特征的接口。
//!
//! 这个crate包含一组基于简单要素标准的特征，用于地理空间矢量数据。
//! 这些特征旨在使在Rust生态系统中操作和使用几何图形变得容易，
//! 而无需了解特定库的API或内存布局。
//!
//! 预期从几何图形中访问任何单个坐标或值都是**常数时间**的。
//! 这意味着当在像WKB这样需要线性时间搜索来定位坐标的格式上实现这些特征时，
//! WKB包装器应该已经进行了初始传递，以找到坐标序列开始和结束的相关字节偏移量。
//!
//! 这个接口通常但并不总是零拷贝的。坐标访问预期是常数时间的，但不一定是_免费_的。
//! 例如，WKB不是对齐的，可能使用与当前机器不同的字节序，
//! 因此在读取时可能需要克隆单个值。

#![deny(missing_docs)]

pub use coord::{CoordTrait, UnimplementedCoord};
pub use dimension::Dimensions;
pub use geometry::{GeometryTrait, GeometryType, UnimplementedGeometry};
pub use geometry_collection::{GeometryCollectionTrait, UnimplementedGeometryCollection};
pub use line::{LineTrait, UnimplementedLine};
pub use line_string::{LineStringTrait, UnimplementedLineString};
pub use multi_line_string::{MultiLineStringTrait, UnimplementedMultiLineString};
pub use multi_point::{MultiPointTrait, UnimplementedMultiPoint};
pub use multi_polygon::{MultiPolygonTrait, UnimplementedMultiPolygon};
pub use point::{PointTrait, UnimplementedPoint};
pub use polygon::{PolygonTrait, UnimplementedPolygon};
pub use rect::{RectTrait, UnimplementedRect};
pub use triangle::{TriangleTrait, UnimplementedTriangle};

// 坐标模块
mod coord;
// 维度模块
mod dimension;
// 几何图形模块
mod geometry;
// 几何图形集合模块
mod geometry_collection;
// 迭代器模块
mod iterator;
// 线模块
mod line;
// 线串模块
mod line_string;
// 多线串模块
mod multi_line_string;
// 多点模块
mod multi_point;
// 多多边形模块
mod multi_polygon;
// 点模块
mod point;
// 多边形模块
mod polygon;
// 矩形模块
mod rect;
// 当启用"geo-types"特性时，包含到geo类型的转换模块
#[cfg(feature = "geo-types")]
pub mod to_geo;
// 三角形模块
mod triangle;
