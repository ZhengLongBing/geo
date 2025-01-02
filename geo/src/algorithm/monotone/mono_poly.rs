use geo_types::{private_utils::get_bounding_rect, Line};

use crate::{
    coordinate_position::CoordPos, sweep::SweepPoint, BoundingRect, Coord, CoordinatePosition,
    GeoNum, Intersects, Kernel, LineString, Orientation, Polygon, Rect,
};

/// 单调多边形
///
/// 单调多边形是可以分解为两个单调链（沿X轴）的多边形。这意味着任何垂直线与多边形最多相交两次（或完全不相交）。
/// 这些多边形支持`O(log n)`时间复杂度的点在多边形内的查询；使用`Intersects<Coord>`特性进行查询。
///
/// 这种结构不能被直接构造。使用`crate::algorithm::monotone_subdivision`算法获得`Vec<MonoPoly>`。
/// 如果不关心单个单调多边形，可以考虑使用`MonotonicPolygons`。
#[derive(Clone, PartialEq)]
pub struct MonoPoly<T: GeoNum> {
    top: LineString<T>,
    bot: LineString<T>,
    bounds: Rect<T>,
}

impl<T: GeoNum> BoundingRect<T> for MonoPoly<T> {
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounds
    }
}
impl<T: GeoNum> std::fmt::Debug for MonoPoly<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top: Vec<SweepPoint<T>> = self.top.0.iter().map(|c| (*c).into()).collect();
        let bot: Vec<SweepPoint<T>> = self.bot.0.iter().map(|c| (*c).into()).collect();
        f.debug_struct("MonoPoly")
            .field("top", &top)
            .field("bot", &bot)
            .finish()
    }
}

impl<T: GeoNum> MonoPoly<T> {
    /// 从顶部和底部链创建单调多边形。
    ///
    /// 注意：每个链必须是严格递增的序列（按字典顺序），具有相同的起点和终点。
    /// 此外，顶部链除了在终点之外，必须严格高于底部链。
    /// 并非所有这些条件都经过检查，如果不满足条件，算法可能会崩溃。
    pub(super) fn new(top: LineString<T>, bot: LineString<T>) -> Self {
        debug_assert_eq!(top.0.first(), bot.0.first());
        debug_assert_eq!(top.0.last(), bot.0.last());
        debug_assert_ne!(top.0.first(), top.0.last());
        let bounds = get_bounding_rect(top.0.iter().chain(bot.0.iter()).cloned()).unwrap();
        Self { top, bot, bounds }
    }

    /// 获取单调多边形顶部链的引用。
    #[must_use]
    pub fn top(&self) -> &LineString<T> {
        &self.top
    }

    /// 获取单调多边形底部链的引用。
    #[must_use]
    pub fn bot(&self) -> &LineString<T> {
        &self.bot
    }

    /// 将本结构转化为(顶部, 底部)链对。
    pub fn into_ls_pair(self) -> (LineString<T>, LineString<T>) {
        (self.top, self.bot)
    }

    /// 获取与给定x坐标平行于Y轴的直线相交的链中的线段对。
    /// 如果有多个交点，则选择索引较低的线段，即距离链起点较近的段。
    pub fn bounding_segment(&self, x: T) -> Option<(Line<T>, Line<T>)> {
        // 二分搜索包含x坐标的线段。
        let tl_idx = self.top.0.partition_point(|c| c.x < x);
        if tl_idx == 0 && self.top.0[0].x != x {
            return None;
        }
        let bl_idx = self.bot.0.partition_point(|c| c.x < x);
        if bl_idx == 0 {
            debug_assert_eq!(tl_idx, 0);
            debug_assert_eq!(self.bot.0[0].x, x);
            return Some((
                Line::new(self.top.0[0], self.top.0[1]),
                Line::new(self.bot.0[0], self.bot.0[1]),
            ));
        } else {
            debug_assert_ne!(tl_idx, 0);
        }

        Some((
            Line::new(self.top.0[tl_idx - 1], self.top.0[tl_idx]),
            Line::new(self.bot.0[bl_idx - 1], self.bot.0[bl_idx]),
        ))
    }

    /// 将本结构转化为[`Polygon`]。
    pub fn into_polygon(self) -> Polygon<T> {
        let mut down = self.bot.0;
        let mut top = self.top.0;

        down.reverse();
        assert_eq!(down.first(), top.last());
        top.extend(down.drain(1..));

        let geom = LineString(top);
        debug_assert!(geom.is_closed());

        Polygon::new(geom, vec![])
    }
}

impl<T: GeoNum> CoordinatePosition for MonoPoly<T> {
    type Scalar = T;

    fn calculate_coordinate_position(
        &self,
        coord: &Coord<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if !self.bounds.intersects(coord) {
            return;
        }
        let (top, bot) = if let Some(t) = self.bounding_segment(coord.x) {
            t
        } else {
            return;
        };

        match T::Ker::orient2d(top.start, *coord, top.end) {
            Orientation::Clockwise => return,
            Orientation::Collinear => {
                *is_inside = true;
                *boundary_count += 1;
                return;
            }
            _ => {}
        }
        match T::Ker::orient2d(bot.start, *coord, bot.end) {
            Orientation::CounterClockwise => (),
            Orientation::Collinear => {
                *is_inside = true;
                *boundary_count += 1;
            }
            _ => {
                *is_inside = true;
            }
        }
    }
}
impl<T: GeoNum> Intersects<Coord<T>> for MonoPoly<T> {
    fn intersects(&self, other: &Coord<T>) -> bool {
        self.coordinate_position(other) != CoordPos::Outside
    }
}
