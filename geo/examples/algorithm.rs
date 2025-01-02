// 开始选择
use geo::{line_string, Centroid};

fn main() {
    // 创建线串
    let linestring = geo::line_string![
        (x: 40.02f64, y: 116.34), // 第一个点
        (x: 41.02f64, y: 116.34), // 第二个点
    ];
    // 计算并打印线串的质心
    println!("Centroid {:?}", linestring.centroid());
}
// 结束选择
