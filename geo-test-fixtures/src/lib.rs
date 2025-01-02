use std::{fs, iter::FromIterator, path::PathBuf, str::FromStr};

use geo_types::{LineString, MultiPolygon, Point, Polygon};
use wkt::{Geometry, Wkt, WktFloat};

/// 返回路易斯安那州的线串
pub fn louisiana<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("louisiana.wkt")
}

/// 返回巴吞鲁日的点
pub fn baton_rouge<T>() -> Point<T>
where
    T: WktFloat + Default + FromStr,
{
    let x = T::from(-91.147385).unwrap();
    let y = T::from(30.471165).unwrap();
    Point::new(x, y)
}

/// 返回东巴吞鲁日的多边形
pub fn east_baton_rouge<T>() -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    polygon("east_baton_rouge.wkt")
}

/// 返回挪威主要地区的线串
pub fn norway_main<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_main.wkt")
}

/// 返回挪威凹边界的线串
pub fn norway_concave_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_concave_hull.wkt")
}

/// 返回挪威凸包的线串
pub fn norway_convex_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_convex_hull.wkt")
}

/// 返回挪威非凸边界的线串
pub fn norway_nonconvex_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_nonconvex_hull.wkt")
}

/// 返回原始 VW 数据的线串
pub fn vw_orig<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("vw_orig.wkt")
}

/// 返回简化后的 VW 数据的线串
pub fn vw_simplified<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("vw_simplified.wkt")
}

/// 返回多边形1的线串
pub fn poly1<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly1.wkt")
}

/// 返回多边形1的凸包线串
pub fn poly1_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly1_hull.wkt")
}

/// 返回多边形2的线串
pub fn poly2<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly2.wkt")
}

/// 返回多边形2的凸包线串
pub fn poly2_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly2_hull.wkt")
}

/// 返回环内的多边形线串
pub fn poly_in_ring<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly_in_ring.wkt")
}

/// 返回单个环的线串
pub fn ring<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("ring.wkt")
}

/// 返回壳的线串
pub fn shell<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("shell.wkt")
}

/// 从特定 URL 获取的荷兰区域多多边形
pub fn nl_zones<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_zones.wkt")
}

/// 从特定 URL 获取的荷兰地块多多边形（使用 WGS84 坐标系）
pub fn nl_plots_wgs84<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_plots.wkt")
}

/// 从特定 URL 获取的荷兰地块多多边形（使用 EPSG:28992 坐标系）
pub fn nl_plots_epsg_28992<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_plots_epsg_28992.wkt")
}

/// 从 WKT 文件中读取线串
fn line_string<T>(name: &str) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::LineString(line_string) => wkt_line_string_to_geo(&line_string),
        _ => unreachable!(),
    }
}

/// 从 WKT 文件中读取多边形
pub fn polygon<T>(name: &str) -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::Polygon(wkt_polygon) => wkt_polygon_to_geo(&wkt_polygon),
        _ => unreachable!(),
    }
}

/// 从 WKT 文件中读取多多边形
pub fn multi_polygon<T>(name: &str) -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::MultiPolygon(multi_polygon) => wkt_multi_polygon_to_geo(&multi_polygon),
        _ => unreachable!(),
    }
}

/// 将 WKT 线串转换为 geo_types 线串
fn wkt_line_string_to_geo<T>(line_string: &wkt::types::LineString<T>) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::from_iter(line_string.0.iter().map(|coord| (coord.x, coord.y)))
}

/// 将 WKT 多边形转换为 geo_types 多边形
fn wkt_polygon_to_geo<T>(polygon: &wkt::types::Polygon<T>) -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let exterior: LineString<T> = wkt_line_string_to_geo(&polygon.0[0]);
    let interiors: Vec<LineString<T>> = polygon.0[1..].iter().map(wkt_line_string_to_geo).collect();

    Polygon::new(exterior, interiors)
}

/// 将 WKT 多多边形转换为 geo_types 多多边形
fn wkt_multi_polygon_to_geo<T>(multi_polygon: &wkt::types::MultiPolygon<T>) -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let polygons: Vec<Polygon<T>> = multi_polygon.0.iter().map(wkt_polygon_to_geo).collect();
    MultiPolygon(polygons)
}

/// 从文件中读取 WKT 字符串
pub fn file(name: &str) -> String {
    let mut res = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("fixtures");
    res.push(name);
    fs::read_to_string(res).unwrap()
}
