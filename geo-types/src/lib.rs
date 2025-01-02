#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_debug_implementations)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! `geo-types` 库为 [GeoRust] 生态系统定义了几何类型。
//!
//! 在大多数情况下，只有当你是一个 crate 作者并且想要与其他 GeoRust crates 兼容时，
//! 你才需要使用这个 crate。否则，[`geo`](https://crates.io/crates/geo) crate
//! 重新导出了这些类型，并额外提供了地理空间算法。
//!
//! ## 几何体
//!
//! - **[`Point`]**: 由一个 [`Coord`] 表示的单点
//! - **[`MultiPoint`]**: [`Point`] 的集合
//! - **[`Line`]**: 由两个 [`Coord`] 表示的线段
//! - **[`LineString`]**: 由两个或更多 [`Coord`] 表示的一系列连续线段
//! - **[`MultiLineString`]**: [`LineString`] 的集合
//! - **[`Polygon`]**: 由一个 [`LineString`] 外环和零个或多个 [`LineString`] 内环表示的有界区域
//! - **[`MultiPolygon`]**: [`Polygon`] 的集合
//! - **[`Rect`]**: 由最小和最大 [`Coord`] 表示的轴对齐有界矩形
//! - **[`Triangle`]**: 由三个 [`Coord`] 顶点表示的有界区域
//! - **[`GeometryCollection`]**: [`Geometry`] 的集合
//! - **[`Geometry`]**: 所有几何类型的枚举，不包括 [`Coord`]
//!
//! ## 坐标和数值类型
//!
//! - **[`Coord`]**: 二维坐标。所有几何类型都由 [`Coord`] 组成，尽管 [`Coord`] 本身不是 [`Geometry`] 类型。
//! 对于单坐标几何体，请参见 [`Point`]。
//!
//! 默认情况下，坐标是 64 位浮点数，但这是泛型的，你可以指定任何实现了 [`CoordNum`] 或 [`CoordFloat`] 的数值类型。
//! 除了 [`f64`]，这还包括常见的数值类型如 [`f32`]、[`i32`]、[`i64`] 等。
//!
//! ```rust
//! use geo_types::Point;
//!
//! // 几何体默认使用 f64
//! let point: Point = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point>(), 64 * 2 / 8);
//!
//! // 你可以明确指定数值类型
//! let f64_point: Point<f64> = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point<f64>>(), 64 * 2 / 8);
//!
//! // 或者指定一些非默认的数值类型
//! let f32_point: Point<f32> = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point<f32>>(), 32 * 2 / 8);
//!
//! // 整数几何体也是支持的，尽管并非所有算法都会为所有数值类型实现
//! let i32_point: Point<i32> = Point::new(1, 2);
//! assert_eq!(std::mem::size_of::<Point<i32>>(), 32 * 2 / 8);
//! ```
//!
//! # 语义
//!
//! 这里提供的地理空间类型旨在遵守 [OpenGIS Simple feature access][OGC-SFA] 标准。
//! 因此，这里的类型与标准的其他实现是可互操作的：[JTS]、[GEOS] 等。
//!
//! # 特性
//!
//! 以下可选的 [Cargo 特性] 可用：
//!
//! - `std`: 启用完整 `std` 库的使用。默认启用。
//! - `multithreading`: 启用对 `Multi*` 几何体的多线程迭代。默认**禁用**，
//!    但被 `geo` 的默认特性**启用**。
//! - `approx`: 允许使用 [approx] 检查几何类型的近似相等性
//! - `arbitrary`: 允许使用 [arbitrary] 从非结构化输入创建几何类型
//! - `serde`: 允许使用 [Serde] 序列化和反序列化几何类型
//! - `use-rstar_0_8`: 允许将几何类型插入到 [rstar] R*-树中（`rstar v0.8`）
//! - `use-rstar_0_9`: 允许将几何类型插入到 [rstar] R*-树中（`rstar v0.9`）
//! - `use-rstar_0_10`: 允许将几何类型插入到 [rstar] R*-树中（`rstar v0.10`）
//! - `use-rstar_0_11`: 允许将几何类型插入到 [rstar] R*-树中（`rstar v0.11`）
//! - `use-rstar_0_12`: 允许将几何类型插入到 [rstar] R*-树中（`rstar v0.12`）
//!
//! 如果禁用默认的 `std` 特性，这个库可以在 `#![no_std]` 环境中使用。目前，
//! `arbitrary` 和 `use-rstar_0_8` 特性需要 `std`。这可能在未来的版本中改变。
//!
//! [approx]: https://github.com/brendanzab/approx
//! [arbitrary]: https://github.com/rust-fuzz/arbitrary
//! [Cargo 特性]: https://doc.rust-lang.org/cargo/reference/features.html
//! [GeoRust]: https://georust.org
//! [GEOS]: https://trac.osgeo.org/geos
//! [JTS]: https://github.com/locationtech/jts
//! [OGC-SFA]: https://www.ogc.org/standards/sfa
//! [rstar]: https://github.com/Stoeoef/rstar
//! [Serde]: https://serde.rs/
extern crate alloc;

use core::fmt::Debug;
use num_traits::{Float, Num, NumCast};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[deprecated(since = "0.7.0", note = "使用 `CoordFloat` 或 `CoordNum` 代替")]
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd + Debug {}
#[allow(deprecated)]
impl<T: Num + Copy + NumCast + PartialOrd + Debug> CoordinateType for T {}

/// 对于可以使用整数和浮点数 `Point`/`Coord` 的算法
///
/// 浮点数（`f32` 和 `f64`）和整数（`u8`、`i32` 等）实现了这个特征。
///
/// 对于只适用于浮点数的算法，如面积或长度计算，
/// 请参见 [CoordFloat](trait.CoordFloat.html)。
#[allow(deprecated)]
pub trait CoordNum: CoordinateType + Debug {}
#[allow(deprecated)]
impl<T: CoordinateType + Debug> CoordNum for T {}

/// 对于只能使用浮点数 `Point`/`Coord` 的算法，如面积或长度计算
pub trait CoordFloat: CoordNum + Float {}
impl<T: CoordNum + Float> CoordFloat for T {}

pub mod geometry;
pub use geometry::*;

pub use geometry::line_string::PointsIter;

#[allow(deprecated)]
pub use geometry::rect::InvalidRectCoordinatesError;

mod error;
pub use error::Error;

#[macro_use]
mod macros;

#[macro_use]
mod wkt_macro;

#[cfg(feature = "arbitrary")]
mod arbitrary;

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
#[doc(hidden)]
pub mod private_utils;

#[doc(hidden)]
pub mod _alloc {
    //! 当 std 特性被禁用且调用上下文缺少 `extern crate alloc` 时，
    //! 需要从 `alloc` 访问这些类型。这些**不**是为公共使用而设计的。
    pub use ::alloc::vec;
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn type_test() {
        let c = coord! {
            x: 40.02f64,
            y: 116.34,
        };

        let p = Point::from(c);

        let Point(c2) = p;
        assert_eq!(c, c2);
        assert_relative_eq!(c.x, c2.x);
        assert_relative_eq!(c.y, c2.y);

        let p: Point<f32> = (0f32, 1f32).into();
        assert_relative_eq!(p.x(), 0.);
        assert_relative_eq!(p.y(), 1.);
    }

    #[test]
    fn convert_types() {
        let p: Point<f32> = Point::new(0., 0.);
        let p1 = p;
        let g: Geometry<f32> = p.into();
        let p2 = Point::try_from(g).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn polygon_new_test() {
        let exterior = LineString::new(vec![
            coord! { x: 0., y: 0. },
            coord! { x: 1., y: 1. },
            coord! { x: 1., y: 0. },
            coord! { x: 0., y: 0. },
        ]);
        let interiors = vec![LineString::new(vec![
            coord! { x: 0.1, y: 0.1 },
            coord! { x: 0.9, y: 0.9 },
            coord! { x: 0.9, y: 0.1 },
            coord! { x: 0.1, y: 0.1 },
        ])];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior(), &exterior);
        assert_eq!(p.interiors(), &interiors[..]);
    }

    #[test]
    fn iters() {
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        let mut l1: LineString<_> = vec![(0., 0.), (1., 2.)].into();
        assert_eq!(l1[1], coord! { x: 1., y: 2. }); // 索引到线串
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        // 可变索引到线串
        l1[0] = coord! { x: 1., y: 1. };
        assert_eq!(l1, vec![(1., 1.), (1., 2.)].into());
    }

    #[test]
    fn test_coordinate_types() {
        let p: Point<u8> = Point::new(0, 0);
        assert_eq!(p.x(), 0u8);

        let p: Point<i64> = Point::new(1_000_000, 0);
        assert_eq!(p.x(), 1_000_000i64);
    }

    #[cfg(feature = "rstar_0_8")]
    #[test]
    /// 确保 Line 的 SpatialObject 实现是正确的
    fn line_test() {
        use rstar_0_8::primitives::Line as RStarLine;
        use rstar_0_8::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // 第15位小数有差异
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[cfg(feature = "rstar_0_9")]
    #[test]
    /// 确保 Line 的 SpatialObject 实现是正确的
    fn line_test_0_9() {
        use rstar_0_9::primitives::Line as RStarLine;
        use rstar_0_9::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // 第15位小数有差异
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[cfg(feature = "rstar_0_10")]
    #[test]
    /// 确保 Line 的 SpatialObject 实现是正确的
    fn line_test_0_10() {
        use rstar_0_10::primitives::Line as RStarLine;
        use rstar_0_10::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[cfg(feature = "rstar_0_11")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test_0_11() {
        use rstar_0_11::primitives::Line as RStarLine;
        use rstar_0_11::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[cfg(feature = "rstar_0_12")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test_0_12() {
        use rstar_0_12::primitives::Line as RStarLine;
        use rstar_0_12::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[test]
    fn test_rects() {
        let r = Rect::new(coord! { x: -1., y: -1. }, coord! { x: 1., y: 1. });
        let p: Polygon<_> = r.into();
        assert_eq!(
            p,
            Polygon::new(
                vec![(-1., -1.), (1., -1.), (1., 1.), (-1., 1.), (-1., -1.)].into(),
                vec![]
            )
        );
    }
}
