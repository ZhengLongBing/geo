use std::cmp::Ordering;

use crate::geometry::*;
use crate::intersects::{point_in_rect, value_in_between};
use crate::kernels::*;
use crate::{BoundingRect, HasDimensions, Intersects};
use crate::{GeoNum, GeometryCow};

/// `Coord` 相对于 `Geometry` 的位置
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CoordPos {
    OnBoundary,
    Inside,
    Outside,
}

/// 确定一个 `Coord` 是否在几何体内部、外部或者在边界上。
///
/// # 示例
///
/// ```rust
/// use geo::{polygon, coord};
/// use geo::coordinate_position::{CoordinatePosition, CoordPos};
///
/// let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];
///
/// let inside_coord = coord! { x: 1.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&inside_coord), CoordPos::Inside);
///
/// let boundary_coord = coord! { x: 0.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&boundary_coord), CoordPos::OnBoundary);
///
/// let outside_coord = coord! { x: 5.0, y: 5.0 };
/// assert_eq!(square_poly.coordinate_position(&outside_coord), CoordPos::Outside);
/// ```
pub trait CoordinatePosition {
    type Scalar: GeoNum;
    fn coordinate_position(&self, coord: &Coord<Self::Scalar>) -> CoordPos {
        // 初始化内部状态和边界计数
        let mut is_inside = false;
        let mut boundary_count = 0;

        self.calculate_coordinate_position(coord, &mut is_inside, &mut boundary_count);

        // “一个任意集合的几何体的边界（其内部互不相交）
        //  由通过 Mod 2 合并规则从元素几何体的边界构成”
        //
        // ― OpenGIS 简单功能访问 § 6.1.15.1
        if boundary_count % 2 == 1 {
            CoordPos::OnBoundary
        } else if is_inside {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        }
    }

    // 该特性的实现必须：
    //  1. 如果 `coord` 在任意组件的内部，则设置 `is_inside = true`。
    //  2. 对于每个包含 `coord` 的边界，增加 `boundary_count`。
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    );
}

impl<T> CoordinatePosition for Coord<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self == coord {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Point<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if &self.0 == coord {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Line<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        // 退化线就是一个点
        if self.start == self.end {
            self.start
                .calculate_coordinate_position(coord, is_inside, boundary_count);
            return;
        }

        if coord == &self.start || coord == &self.end {
            *boundary_count += 1;
        } else if self.intersects(coord) {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for LineString<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.0.len() < 2 {
            debug_assert!(false, "具有少于 2 个坐标的无效线字符串");
            return;
        }

        if self.0.len() == 2 {
            // 具有两个坐标的线字符串就是一条线
            Line::new(self.0[0], self.0[1]).calculate_coordinate_position(
                coord,
                is_inside,
                boundary_count,
            );
            return;
        }

        // 优化：如果没有相交的可能性，则尽早返回
        // 因为 self.0 是非空的，可以安全地 `unwrap`
        if !self.bounding_rect().unwrap().intersects(coord) {
            return;
        }

        // 根据 SFS，封闭的线字符串没有边界
        if !self.is_closed() {
            // 因为 self.0 是非空的，可以安全地 `unwrap`
            if coord == self.0.first().unwrap() || coord == self.0.last().unwrap() {
                *boundary_count += 1;
                return;
            }
        }

        if self.intersects(coord) {
            // 我们已经检查了“边界”条件，所以如果在这点上有相交，coord 必须在内部
            *is_inside = true
        }
    }
}

impl<T> CoordinatePosition for Triangle<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        *is_inside = self
            .to_lines()
            .map(|l| {
                let orientation = T::Ker::orient2d(l.start, l.end, *coord);
                if orientation == Orientation::Collinear
                    && point_in_rect(*coord, l.start, l.end)
                    && coord.x != l.end.x
                {
                    *boundary_count += 1;
                }
                orientation
            })
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear);
    }
}

impl<T> CoordinatePosition for Rect<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let mut boundary = false;

        let min = self.min();

        match coord.x.partial_cmp(&min.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match coord.y.partial_cmp(&min.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }

        let max = self.max();

        match max.x.partial_cmp(&coord.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match max.y.partial_cmp(&coord.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }

        if boundary {
            *boundary_count += 1;
        } else {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for MultiPoint<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self.0.iter().any(|p| &p.0 == coord) {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Polygon<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.is_empty() {
            return;
        }

        match coord_pos_relative_to_ring(*coord, self.exterior()) {
            CoordPos::Outside => {}
            CoordPos::OnBoundary => {
                *boundary_count += 1;
            }
            CoordPos::Inside => {
                for hole in self.interiors() {
                    match coord_pos_relative_to_ring(*coord, hole) {
                        CoordPos::Outside => {}
                        CoordPos::OnBoundary => {
                            *boundary_count += 1;
                            return;
                        }
                        CoordPos::Inside => {
                            return;
                        }
                    }
                }
                // coord 在内部孔的外部，所以它在多边形内部
                *is_inside = true;
            }
        }
    }
}

impl<T> CoordinatePosition for MultiLineString<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for line_string in &self.0 {
            line_string.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for MultiPolygon<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for polygon in &self.0 {
            polygon.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for GeometryCollection<T>
where
    T: GeoNum,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for geometry in self {
            geometry.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for Geometry<T>
where
    T: GeoNum,
{
    type Scalar = T;
    crate::geometry_delegate_impl! {
        fn calculate_coordinate_position(
            &self,
            coord: &Coord<T>,
            is_inside: &mut bool,
            boundary_count: &mut usize) -> ();
    }
}

impl<T: GeoNum> CoordinatePosition for GeometryCow<'_, T> {
    type Scalar = T;
    crate::geometry_cow_delegate_impl! {
        fn calculate_coordinate_position(
            &self,
            coord: &Coord<T>,
            is_inside: &mut bool,
            boundary_count: &mut usize) -> ();
    }
}

/// 计算 `Coord` 相对于封闭 `LineString` 的位置。
pub fn coord_pos_relative_to_ring<T>(coord: Coord<T>, linestring: &LineString<T>) -> CoordPos
where
    T: GeoNum,
{
    debug_assert!(linestring.is_closed());

    // 没有点的 LineString
    if linestring.0.is_empty() {
        return CoordPos::Outside;
    }
    if linestring.0.len() == 1 {
        // 如果 LineString 只有一个点，则不会生成任何线。
        // 因此，我们需要单独处理这种边缘情况。
        return if coord == linestring.0[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    // 使用卷绕数算法并在边界进行短路
    // 参见：https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm
    let mut winding_number = 0;
    for line in linestring.lines() {
        // 边交叉规则：
        //   1. 向上的边包括其起始端点，排除其结束端点；
        //   2. 向下的边排除其起始端点，包括其结束端点；
        //   3. 水平边被排除
        //   4. 边光线相交点必须严格位于 coord 的右边。
        if line.start.y <= coord.y {
            if line.end.y >= coord.y {
                let o = T::Ker::orient2d(line.start, line.end, coord);
                if o == Orientation::CounterClockwise && line.end.y != coord.y {
                    winding_number += 1
                } else if o == Orientation::Collinear
                    && value_in_between(coord.x, line.start.x, line.end.x)
                {
                    return CoordPos::OnBoundary;
                }
            };
        } else if line.end.y <= coord.y {
            let o = T::Ker::orient2d(line.start, line.end, coord);
            if o == Orientation::Clockwise {
                winding_number -= 1
            } else if o == Orientation::Collinear
                && value_in_between(coord.x, line.start.x, line.end.x)
            {
                return CoordPos::OnBoundary;
            }
        }
    }
    if winding_number == 0 {
        CoordPos::Outside
    } else {
        CoordPos::Inside
    }
}

#[cfg(test)]
mod test {
    use geo_types::coord;

    use super::*;
    use crate::{line_string, point, polygon};

    #[test]
    fn test_empty_poly() {
        let square_poly: Polygon<f64> = Polygon::new(LineString::new(vec![]), vec![]);
        assert_eq!(
            square_poly.coordinate_position(&Coord::zero()),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_simple_poly() {
        let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];

        let inside_coord = coord! { x: 1.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&inside_coord),
            CoordPos::Inside
        );

        let vertex_coord = coord! { x: 0.0, y: 0.0 };
        assert_eq!(
            square_poly.coordinate_position(&vertex_coord),
            CoordPos::OnBoundary
        );

        let boundary_coord = coord! { x: 0.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&boundary_coord),
            CoordPos::OnBoundary
        );

        let outside_coord = coord! { x: 5.0, y: 5.0 };
        assert_eq!(
            square_poly.coordinate_position(&outside_coord),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_poly_interior() {
        let poly = polygon![
            exterior: [
                (x: 11., y: 11.),
                (x: 20., y: 11.),
                (x: 20., y: 20.),
                (x: 11., y: 20.),
                (x: 11., y: 11.),
            ],
            interiors: [
                [
                    (x: 13., y: 13.),
                    (x: 13., y: 17.),
                    (x: 17., y: 17.),
                    (x: 17., y: 13.),
                    (x: 13., y: 13.),
                ]
            ],
        ];

        let inside_hole = coord! { x: 14.0, y: 14.0 };
        assert_eq!(poly.coordinate_position(&inside_hole), CoordPos::Outside);

        let outside_poly = coord! { x: 30.0, y: 30.0 };
        assert_eq!(poly.coordinate_position(&outside_poly), CoordPos::Outside);

        let on_outside_border = coord! { x: 20.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_outside_border),
            CoordPos::OnBoundary
        );

        let on_inside_border = coord! { x: 13.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_inside_border),
            CoordPos::OnBoundary
        );

        let inside_coord = coord! { x: 12.0, y: 12.0 };
        assert_eq!(poly.coordinate_position(&inside_coord), CoordPos::Inside);
    }

    #[test]
    fn test_simple_line() {
        use crate::point;
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 10.0, y: 10.0]);

        let start = coord! { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::OnBoundary);

        let end = coord! { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&end), CoordPos::OnBoundary);

        let interior = coord! { x: 5.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&interior), CoordPos::Inside);

        let outside = coord! { x: 6.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line() {
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 0.0, y: 0.0]);

        let start = coord! { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::Inside);

        let outside = coord! { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_point() {
        let p1 = point![x: 2.0, y: 0.0];

        let c1 = coord! { x: 2.0, y: 0.0 };
        let c2 = coord! { x: 3.0, y: 3.0 };

        assert_eq!(p1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(p1.coordinate_position(&c2), CoordPos::Outside);

        assert_eq!(c1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(c1.coordinate_position(&c2), CoordPos::Outside);
    }

    #[test]
    fn test_simple_line_string() {
        let line_string =
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 2.0, y: 0.0), (x: 3.0, y: 0.0)];

        let start = Coord::zero();
        assert_eq!(
            line_string.coordinate_position(&start),
            CoordPos::OnBoundary
        );

        let midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let vertex = coord! { x: 2.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&vertex), CoordPos::Inside);

        let end = coord! { x: 3.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&end), CoordPos::OnBoundary);

        let outside = coord! { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line_strings() {
        let line_string = line_string![(x: 0.0, y: 0.0), (x: 0.0, y: 0.0)];

        let start = Coord::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0)];

        let start = Coord::zero();
        assert_eq!(
            line_string.coordinate_position(&start),
            CoordPos::OnBoundary
        );
    }

    #[test]
    fn test_closed_line_string() {
        let line_string = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 2.0, y: 0.0), (x: 3.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];

        // 检查封闭性
        assert!(line_string.is_closed());

        // 封闭的线字符串没有边界
        let start = Coord::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let outside = coord! { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_boundary_rule() {
        let multi_line_string = MultiLineString::new(vec![
            // 第一和第二条线有相同的起点但不同的终点
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)],
            line_string![(x: 0.0, y: 0.0), (x: -1.0, y: -1.0)],
            // 第三条线有自己的起点，但终点接触到第一条线的中间
            line_string![(x: 0.0, y: 1.0), (x: 0.5, y: 0.5)],
            // 第四条和第五条线有独立的起点，但都在第二条线的中间终结
            line_string![(x: 0.0, y: -1.0), (x: -0.5, y: -0.5)],
            line_string![(x: 0.0, y: -2.0), (x: -0.5, y: -0.5)],
        ]);

        let outside_of_all = coord! { x: 123.0, y: 123.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&outside_of_all),
            CoordPos::Outside
        );

        let end_of_one_line = coord! { x: -1.0, y: -1.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&end_of_one_line),
            CoordPos::OnBoundary
        );

        // 第一条和第二条线的边界中，所以根据 mod 2 规则不被认为在边界中
        let shared_start = Coord::zero();
        assert_eq!(
            multi_line_string.coordinate_position(&shared_start),
            CoordPos::Outside
        );

        // 在第一条线*中*，在第三条线的边界上
        let one_end_plus_midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&one_end_plus_midpoint),
            CoordPos::OnBoundary
        );

        // 在第一条线*中*，在第四条和第五条线的*边界上*
        let two_ends_plus_midpoint = coord! { x: -0.5, y: -0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&two_ends_plus_midpoint),
            CoordPos::Inside
        );
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        assert_eq!(
            rect.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            rect.coordinate_position(&coord! { x: 0.0, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            rect.coordinate_position(&coord! { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_triangle() {
        let triangle = Triangle::new((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 2.49, y: 5.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_collection() {
        let triangle = Triangle::new((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        let collection = GeometryCollection::new_from(vec![triangle.into(), rect.into()]);

        // 在两个几何体的外部
        assert_eq!(
            collection.coordinate_position(&coord! { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );

        // 在两个几何体的内部
        assert_eq!(
            collection.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );

        // 在一个几何体的内部，在另一个几何体的边界上
        assert_eq!(
            collection.coordinate_position(&coord! { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );

        // 在两个几何体的边界上
        assert_eq!(
            collection.coordinate_position(&coord! { x: 5.0, y: 10.0 }),
            CoordPos::Outside
        );
    }
}
