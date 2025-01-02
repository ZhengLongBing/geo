use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::Intersects;
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

// ┌──────────────────────────┐
// │ Line 的实现部分            │
// └──────────────────────────┘

impl<T> Contains<Coord<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        if self.start == self.end {
            // 如果线的起点和终点相同，只需检查起点与给定坐标是否相同
            &self.start == coord
        } else {
            // 如果坐标不是起点或终点，并且在该线段上，则包含该坐标
            coord != &self.start && coord != &self.end && self.intersects(coord)
        }
    }
}

impl<T> Contains<Point<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        // 使用坐标的实现来检查点是否在直线上
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            // 如果线的起点和终点相同，只需检查该点是否包含在当前线中
            self.contains(&line.start)
        } else {
            // 检查给定线的起点和终点是否都在当前线内进行包含判断
            self.intersects(&line.start) && self.intersects(&line.end)
        }
    }
}

impl<T> Contains<LineString<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // 如果 LineString 是空的，则无内点，任何几何体均不能包含它
        if linestring.0.is_empty() {
            return false;
        }

        // LineString 的内点应与当前线的内点有交集，两种情况：
        //
        // 1. LineString 至少有两个不同的点，则这两个点之间的内点必然与
        // 当前线有非空交集。
        //
        // 2. 否则，LineString 上所有点是相同的，在这种情况下，该点
        // 就是内点，它应该包含在当前线内。
        let first = linestring.0.first().unwrap();
        let mut all_equal = true;

        // 如果 linestring 的所有顶点都与当前线段相交，
        // 则 linestring 的边界不会与当前线的外部有非空交集。
        let all_intersects = linestring.0.iter().all(|c| {
            if c != first {
                all_equal = false;
            }
            self.intersects(c)
        });

        all_intersects && (!all_equal || self.contains(first))
    }
}

// 使用宏从 relate 方法生成包含实现
impl_contains_from_relate!(Line<T>, [Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Line<T>);
