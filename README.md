[![geo](https://avatars1.githubusercontent.com/u/10320338?v=4&s=50)](https://github.com/georust)

[![geo 在 Crates.io 上](https://img.shields.io/crates/v/geo.svg?color=brightgreen)](https://crates.io/crates/geo)
[![覆盖率状态](https://img.shields.io/coverallsCoverage/github/georust/geo.svg)](https://coveralls.io/github/georust/geo?branch=trying)
[![文档](https://img.shields.io/docsrs/geo/latest.svg)](https://docs.rs/geo)
[![Discord](https://img.shields.io/discord/598002550221963289)](https://discord.gg/Fp2aape)

# geo

## 地理空间基本类型、算法和工具

### 在 [Discord](https://discord.gg/Fp2aape) 上聊天或提问

`geo` crate 提供了诸如 `Point`、`LineString` 和 `Polygon` 这样的地理空间基本类型，并提供了诸如以下的算法和操作：
- 面积和中心点计算
- 简化和凸包操作
- 欧几里得和 Haversine 距离测量
- 相交检查
- 仿射变换，如旋转和位移
- 所有 DE-9IM 空间谓词，例如 contains、crosses 和 touches。

请参考[文档](https://docs.rs/geo)以获取完整的列表。

基本类型也为 `Geo` 生态系统中的其他功能提供基础，包括：

- [坐标转换和投影](https://github.com/georust/proj)
- 从 [GeoJSON](https://github.com/georust/geojson) 和 [WKT](https://github.com/georust/wkt) 进行序列化和反序列化
- [地理编码](https://github.com/georust/geocoding)
- [处理 GPS 数据](https://github.com/georust/gpx)

## 示例

```rust
// 基本类型
use geo::{line_string, polygon};

// 算法
use geo::ConvexHull;

// 一个 L 形状
let poly = polygon![
    (x: 0.0, y: 0.0),
    (x: 4.0, y: 0.0),
    (x: 4.0, y: 1.0),
    (x: 1.0, y: 1.0),
    (x: 1.0, y: 4.0),
    (x: 0.0, y: 4.0),
    (x: 0.0, y: 0.0),
];

// 计算多边形的凸包
let hull = poly.convex_hull();

assert_eq!(
    hull.exterior(),
    &line_string![
        (x: 4.0, y: 0.0),
        (x: 4.0, y: 1.0),
        (x: 1.0, y: 4.0),
        (x: 0.0, y: 4.0),
        (x: 0.0, y: 0.0),
        (x: 4.0, y: 0.0),
    ]
);
```

## 贡献

欢迎贡献！看看这些[问题](https://github.com/georust/geo/issues)，如果你想添加算法或功能，可以发起一个拉取请求。

## 许可

可以选择使用以下许可之一：

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0)
 * MIT 许可证 ([LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT)

### 贡献内容

除非您明确声明，否则您有意提交的任何用于包含在此作品中的贡献，将依据 Apache-2.0 许可证进行上述双重许可，且无任何附加条款或条件。
