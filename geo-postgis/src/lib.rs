#![warn(missing_debug_implementations)]
//! 在[`geo-types`]和[`postgis`]类型之间进行转换。
//!
//! # 示例
//!
//! 将一个`postgis`点转换为一个`geo-types`点：
//!
//! [`geo-types`]: https://crates.io/crates/geo-types
//! [`postgis`]: https://crates.io/crates/postgis
//!
//! ```rust
//! use geo_postgis::FromPostgis; // 导入用于从PostGIS类型转换的trait
//!
//! let postgis_point = postgis::ewkb::Point { x: 1., y: -2., srid: None }; // 创建一个PostGIS点
//!
//! let geo_point = geo_types::Point::from_postgis(&postgis_point); // 将PostGIS点转换为Geo-types点
//!
//! assert_eq!(
//!     geo_types::point!(x: 1., y: -2.), // 创建一个Geo-types点用于比较
//!     geo_point, // 确保转换后的点与预期的Geo-types点一致
//! );
//! ```
//!
//! 将一个`geo-types`点转换为一个`postgis`点：
//!
//! ```rust
//! use geo_postgis::ToPostgis; // 导入用于转换为PostGIS类型的trait
//!
//! let geo_point = geo_types::point!(x: 1., y: -2.); // 创建一个Geo-types点
//!
//! let postgis_point = geo_point.to_postgis_with_srid(None); // 将Geo-types点转换为PostGIS点
//!
//! assert_eq!(
//!     postgis::ewkb::Point { x: 1., y: -2., srid: None }, // 创建一个PostGIS点用于比较
//!     postgis_point, // 确保转换后的点与预期的PostGIS点一致
//! );
//! ```

mod to_postgis; // 模块用于将geo-types转换为postgis
pub use to_postgis::ToPostgis; // 导出ToPostgis trait

mod from_postgis; // 模块用于将postgis转换为geo-types
pub use from_postgis::FromPostgis; // 导出FromPostgis trait
