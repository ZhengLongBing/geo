use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::Intersects;
use crate::geometry::*;
use crate::{CoordNum, GeoFloat, GeoNum, HasDimensions};

// ┌────────────────────────────────┐
// │ LineString 的实现 │
// └────────────────────────────────┘

impl<T> Contains<Coord<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        if self.0.is_empty() {
            // 如果 LineString 是空的，返回 false
            return false;
        }

        // 检查坐标是否在 LineString 的第一个或最后一个点
        if coord == &self.0[0] || coord == self.0.last().unwrap() {
            return self.is_closed(); // 如果是检查 LineString 是否闭合
        }

        // 检查直线段是否包含坐标
        self.lines()
            .enumerate()
            .any(|(i, line)| line.contains(coord) || (i > 0 && coord == &line.start))
    }
}

impl<T> Contains<Point<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0) // 使用坐标的实现来检查点
    }
}

impl<T> Contains<Line<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            // 如果线的起点和终点相同，只需检查点
            return self.contains(&line.start);
        }

        // 我们复制这条线，因为我们可能会在找到部分匹配时截断这条线
        let mut line = *line;
        let mut first_cut = None;

        let lines_iter = self.lines();
        let num_lines = lines_iter.len();

        // 我们需要重复逻辑以处理 LineString 在线段中间开始的情况
        for (i, segment) in self.lines().chain(lines_iter).enumerate() {
            if i >= num_lines {
                // 第一次循环完成后，如果从未截断线或者回到截断段，可退出
                if let Some(upto_i) = first_cut {
                    if i >= num_lines + upto_i {
                        break;
                    }
                } else {
                    break;
                }
            }
            // 寻找同时与线的一端相交的段
            let other = if segment.intersects(&line.start) {
                line.end
            } else if segment.intersects(&line.end) {
                line.start
            } else {
                continue; // 如果没有相交，检查下一个段
            };

            // 如果另一端也与此段相交，则完成
            let new_inside = if segment.intersects(&other) {
                return true;
            } else if line.contains(&segment.start) {
                segment.start // 截断线以保留外部部分
            } else if line.contains(&segment.end) {
                segment.end
            } else {
                continue; // 不相交则继续
            };

            first_cut = first_cut.or(Some(i));
            if other == line.start {
                line.end = new_inside;
            } else {
                line.start = new_inside;
            }
        }

        false
    }
}

impl<T> Contains<LineString<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, rhs: &LineString<T>) -> bool {
        // 如果任一线段为空，则返回 false
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        // 检查 LineString 的每一条线段是否被包含
        rhs.lines().all(|l| self.contains(&l))
    }
}

impl_contains_from_relate!(LineString<T>, [Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(LineString<T>);

// ┌─────────────────────────────────────┐
// │ MultiLineString 的实现 │
// └─────────────────────────────────────┘

impl_contains_from_relate!(MultiLineString<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(MultiLineString<T>);

impl<T> Contains<Point<T>> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Contains<Point<T>>,
{
    fn contains(&self, rhs: &Point<T>) -> bool {
        // 迭代检查任何 LineString 是否包含给定点
        self.iter().any(|ls| ls.contains(rhs))
    }
}
