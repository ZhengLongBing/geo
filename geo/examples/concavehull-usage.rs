use geo::ConcaveHull;
use geo::ConvexHull;
use geo::{Coord, Point};
use geo_types::MultiPoint;
use std::fs::File;
use std::io::Write;

// 生成多边形的字符串表示
fn generate_polygon_str(coords: &[Coord]) -> String {
    let mut points_str = String::from("");
    for coord in coords {
        points_str.push_str(format!("{},{} ", coord.x, coord.y).as_ref());
    }
    format!(
        "    <polygon points=\"{}\" fill=\"none\" stroke=\"black\"/>\n",
        points_str
    )
}

// 生成连续的圆形字符串表示
fn generate_consecutive_circles(coords: &[Coord]) -> String {
    let mut circles_str = String::from("");
    for coord in coords {
        circles_str.push_str(
            format!("<circle cx=\"{}\" cy=\"{}\" r=\"1\"/>\n", coord.x, coord.y).as_ref(),
        );
    }
    circles_str
}

// 生成文件内容的字符串
fn produce_file_content(start_str: &str, mid_str: &str) -> String {
    let mut overall_string = start_str.to_string();
    overall_string.push_str(mid_str);
    overall_string.push_str("</svg>");
    overall_string
}

// 移动点以便它们集中在图像中心
fn move_points_in_viewbox(width: f64, height: f64, points: Vec<Point>) -> Vec<Point> {
    let mut new_points = vec![];
    for point in points {
        new_points.push(Point::new(
            point.0.x + width / 2.0,
            point.0.y + height / 2.0,
        ));
    }
    new_points
}

// 将点映射到坐标
fn map_points_to_coords(points: Vec<Point>) -> Vec<Coord> {
    points.iter().map(|point| point.0).collect()
}

fn main() -> std::io::Result<()> {
    let mut points_file = File::create("points.svg")?; // 创建文件以存储点
    let mut concave_hull_file = File::create("concavehull.svg")?; // 创建文件以存储凹壳
    let mut convex_hull_file = File::create("convexhull.svg")?; // 创建文件以存储凸壳
    let width = 100;
    let height = 100;
    let svg_file_string = format!(
        "<svg viewBox=\"50 50 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        width, height
    );
    let norway = geo_test_fixtures::norway_main::<f64>();
    let v: Vec<_> = norway
        .0
        .into_iter()
        .map(|coord| Point::new(coord.x, coord.y))
        .collect();
    // 将点移动至观点框中
    let moved_v = move_points_in_viewbox(width as f64, height as f64, v);
    let multipoint = MultiPoint::from(moved_v);
    // 计算凹壳和凸壳
    let concave = multipoint.concave_hull(2.0);
    let convex = multipoint.convex_hull();
    // 生成凹壳和凸壳的多边形字符串
    let concave_polygon_str = generate_polygon_str(&concave.exterior().0);
    let convex_polygon_str = generate_polygon_str(&convex.exterior().0);
    // 获取点的坐标并生成圆形字符串
    let v_coords = map_points_to_coords(multipoint.0);
    let circles_str = generate_consecutive_circles(&v_coords);
    // 生成文件内容字符串
    let points_str = produce_file_content(&svg_file_string, &circles_str);
    let concave_hull_str = produce_file_content(&svg_file_string, &concave_polygon_str);
    let convex_hull_str = produce_file_content(&svg_file_string, &convex_polygon_str);

    // 写入到对应文件
    points_file.write_all(points_str.as_ref())?;
    concave_hull_file.write_all(concave_hull_str.as_ref())?;
    convex_hull_file.write_all(convex_hull_str.as_ref())?;
    Ok(())
}
