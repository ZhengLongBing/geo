use geo::Point;
use geo_types::point;

fn main() {
    // 创建一个点
    let p = point! {
        x: 40.02f64,
        y: 116.34,
    };

    // 解构点对象并打印坐标
    let Point(coord) = p;
    println!("Point at ({}, {})", coord.x, coord.y);
}
