#![no_main]

use geo::EuclideanLength;
use geo::Simplify;
use libfuzzer_sys::fuzz_target;

// 定义模糊测试目标
fuzz_target!(|tuple: (geo_types::Polygon<f32>, f32)| {
    // 解构传入的元组
    let (polygon, epsilon) = tuple;

    // 对多边形进行简化操作
    let simplified = polygon.simplify(&epsilon);

    // 检查简化结果
    check_result(polygon, simplified);
});

// 检查简化后的多边形是否符合预期
fn check_result(original: geo_types::Polygon<f32>, simplified: geo_types::Polygon<f32>) {
    // 检查外部轮廓点数是否符合预期
    assert!(simplified.exterior().0.len() <= original.exterior().0.len());
    // 检查欧几里得长度是否符合预期
    assert!(simplified.exterior().euclidean_length() <= original.exterior().euclidean_length());

    // 遍历每个内部轮廓
    for interior in simplified.interiors() {
        // 检查外部轮廓点数是否小于等于内部轮廓点数
        assert!(simplified.exterior().0.len() <= interior.0.len());
        // 检查外部轮廓的长度是否小于等于内部轮廓的长度
        assert!(simplified.exterior().euclidean_length() <= interior.euclidean_length());
    }
}
