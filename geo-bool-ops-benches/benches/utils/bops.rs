#![allow(dead_code)] // 允许未使用的代码

use geo_types::{Coord, LineString, MultiPolygon, Polygon}; // 导入geo_types库中的几何类型

/// 将一个Polygon<f64>类型的多边形转换为gt_prev::Polygon<f64>类型
pub fn convert_poly(poly: &Polygon<f64>) -> gt_prev::Polygon<f64> {
    // 提取外环顶点并转换为gt_prev::Coordinate类型
    let ext: Vec<_> = poly
        .exterior()
        .0
        .iter()
        .map(|c| gt_prev::Coordinate { x: c.x, y: c.y })
        .collect();
    gt_prev::Polygon::new(gt_prev::LineString(ext), vec![]) // 创建新的gt_prev::Polygon对象
}

/// 将一个MultiPolygon<f64>类型的多多边形转换为gt_prev::MultiPolygon<f64>类型
pub fn convert_mpoly(mpoly: &MultiPolygon<f64>) -> gt_prev::MultiPolygon<f64> {
    mpoly.0.iter().map(convert_poly).collect() // 对每个Polygon进行转换
}

/// 将一个gt_prev::Polygon<f64>类型的多边形转换回Polygon<f64>类型
pub fn convert_back_poly(poly: &gt_prev::Polygon<f64>) -> Polygon<f64> {
    // 提取外环顶点并转换回Coord类型
    let ext: Vec<_> = poly
        .exterior()
        .0
        .iter()
        .map(|c| Coord { x: c.x, y: c.y })
        .collect();
    Polygon::new(LineString(ext), vec![]) // 创建新的Polygon对象
}

/// 将一个gt_prev::MultiPolygon<f64>类型的多多边形转换回MultiPolygon<f64>类型
pub fn convert_back_mpoly(mpoly: &gt_prev::MultiPolygon<f64>) -> MultiPolygon<f64> {
    mpoly.0.iter().map(convert_back_poly).collect() // 对每个gt_prev::Polygon进行转换
}
