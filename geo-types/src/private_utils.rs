// 为了在 geo-types crates 中实现 RStar 的特性，我们需要访问一些地理空间算法，
// 这些算法包含在这个隐藏模块中。这个隐藏模块是公开的，以便 geo crate 可以重用这些算法
// 以防止重复。这些函数 _不_ 是为公共使用而设计的。

use crate::{Coord, CoordFloat, CoordNum, Line, LineString, Point, Rect};

pub fn line_string_bounding_rect<T>(line_string: &LineString<T>) -> Option<Rect<T>>
where
    T: CoordNum,
{
    get_bounding_rect(line_string.coords().cloned())
}

pub fn line_bounding_rect<T>(line: Line<T>) -> Rect<T>
where
    T: CoordNum,
{
    Rect::new(line.start, line.end)
}

pub fn get_bounding_rect<I, T>(collection: I) -> Option<Rect<T>>
where
    T: CoordNum,
    I: IntoIterator<Item = Coord<T>>,
{
    let mut iter = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        for pnt in iter {
            let (px, py) = pnt.x_y();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }

        return Some(Rect::new(
            coord! {
                x: xrange.0,
                y: yrange.0,
            },
            coord! {
                x: xrange.1,
                y: yrange.1,
            },
        ));
    }
    None
}

fn get_min_max<T: PartialOrd>(p: T, min: T, max: T) -> (T, T) {
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}

pub fn line_segment_distance<T, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    let point = point.into();
    let start = start.into();
    let end = end.into();

    if start == end {
        return line_euclidean_length(Line::new(point, start));
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let d_squared = dx * dx + dy * dy;
    let r = ((point.x - start.x) * dx + (point.y - start.y) * dy) / d_squared;
    if r <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new(point, end));
    }
    let s = ((start.y - point.y) * dx - (start.x - point.x) * dy) / d_squared;
    s.abs() * dx.hypot(dy)
}

pub fn line_euclidean_length<T>(line: Line<T>) -> T
where
    T: CoordFloat,
{
    line.dx().hypot(line.dy())
}

pub fn point_line_string_euclidean_distance<T>(p: Point<T>, l: &LineString<T>) -> T
where
    T: CoordFloat,
{
    // 如果点在 LineString 上或 LineString 为空，则无需继续
    if line_string_contains_point(l, p) || l.0.is_empty() {
        return T::zero();
    }
    l.lines()
        .map(|line| line_segment_distance(p.0, line.start, line.end))
        .fold(T::max_value(), |accum, val| accum.min(val))
}

pub fn point_line_euclidean_distance<C, T>(p: C, l: Line<T>) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    line_segment_distance(p.into(), l.start, l.end)
}

pub fn point_contains_point<T>(p1: Point<T>, p2: Point<T>) -> bool
where
    T: CoordFloat,
{
    let distance = line_euclidean_length(Line::new(p1, p2)).to_f32().unwrap();
    approx::relative_eq!(distance, 0.0)
}

pub fn line_string_contains_point<T>(line_string: &LineString<T>, point: Point<T>) -> bool
where
    T: CoordFloat,
{
    // 没有点的 LineString
    if line_string.0.is_empty() {
        return false;
    }
    // 只有一个点等于 p 的 LineString
    if line_string.0.len() == 1 {
        return point_contains_point(Point::from(line_string[0]), point);
    }
    // 检查点是否是顶点
    if line_string.0.contains(&point.0) {
        return true;
    }
    for line in line_string.lines() {
        // 这是 "intersects" 模块中线包含点逻辑的副本
        let tx = if line.dx() == T::zero() {
            None
        } else {
            Some((point.x() - line.start.x) / line.dx())
        };
        let ty = if line.dy() == T::zero() {
            None
        } else {
            Some((point.y() - line.start.y) / line.dy())
        };
        let contains = match (tx, ty) {
            (None, None) => {
                // 退化线
                point.0 == line.start
            }
            (Some(t), None) => {
                // 水平线
                point.y() == line.start.y && T::zero() <= t && t <= T::one()
            }
            (None, Some(t)) => {
                // 垂直线
                point.x() == line.start.x && T::zero() <= t && t <= T::one()
            }
            (Some(t_x), Some(t_y)) => {
                // 所有其他线
                (t_x - t_y).abs() <= T::epsilon() && T::zero() <= t_x && t_x <= T::one()
            }
        };
        if contains {
            return true;
        }
    }
    false
}
