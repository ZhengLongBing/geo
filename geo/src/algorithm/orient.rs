use crate::{GeoNum, MultiPolygon, Polygon};

use crate::winding_order::{Winding, WindingOrder};

pub trait Orient {
    /// 根据约定对多边形的外环和内环进行定向
    ///
    /// 默认情况下，多边形的外环是逆时针定向的，而任何内环是顺时针定向的。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::orient::{Direction, Orient};
    /// use geo::polygon;
    ///
    /// // 一个菱形
    /// let polygon = polygon![
    ///     // 顺时针定向的外环
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     // 逆时针定向的内环
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// let oriented = polygon.orient(Direction::Default);
    ///
    /// // 一个菱形
    /// let expected = polygon![
    ///     // 逆时针定向的外环
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     // 顺时针定向的内环
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// assert_eq!(expected, oriented);
    /// ```
    fn orient(&self, orientation: Direction) -> Self;
}

impl<T> Orient for Polygon<T>
where
    T: GeoNum,
{
    fn orient(&self, direction: Direction) -> Polygon<T> {
        // 根据方向对多边形进行定向
        orient(self, direction)
    }
}

impl<T> Orient for MultiPolygon<T>
where
    T: GeoNum,
{
    fn orient(&self, direction: Direction) -> MultiPolygon<T> {
        // 对每个多边形进行定向，然后收集成新的MultiPolygon
        MultiPolygon::new(self.iter().map(|poly| poly.orient(direction)).collect())
    }
}

/// 默认情况下，一个正确定向的多边形的外环为逆时针方向，
/// 内环为顺时针方向。选择 `Reversed` 将使外环为顺时针方向，
/// — 内环为逆时针方向。
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    /// 外环为逆时针方向，内环为顺时针方向
    Default,
    /// 外环为顺时针方向，内环为逆时针方向
    Reversed,
}

/// 按照惯例定向一个多边形
/// 默认情况下，外环会被定向为逆时针
/// 内环会被定向为顺时针
fn orient<T>(poly: &Polygon<T>, direction: Direction) -> Polygon<T>
where
    T: GeoNum,
{
    let interiors = poly
        .interiors()
        .iter()
        .map(|l| {
            // 根据方向匹配适当的绕行顺序并复制到内环
            l.clone_to_winding_order(match direction {
                Direction::Default => WindingOrder::Clockwise,
                Direction::Reversed => WindingOrder::CounterClockwise,
            })
        })
        .collect();

    let ext_ring = poly.exterior().clone_to_winding_order(match direction {
        // 将外环复制为适当的绕行顺序
        Direction::Default => WindingOrder::CounterClockwise,
        Direction::Reversed => WindingOrder::Clockwise,
    });

    // 创建一个新的多边形
    Polygon::new(ext_ring, interiors)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{LineString, Polygon};
    #[test]
    fn test_polygon_orientation() {
        // 一个顺时针方向的菱形外环
        let points_ext = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 1.0), (1.0, 0.0)];
        // 逆时针方向的内环
        let points_int = vec![(1.0, 0.5), (1.5, 1.0), (1.0, 1.5), (0.5, 1.0), (1.0, 0.5)];
        let poly1 = Polygon::new(
            LineString::from(points_ext),
            vec![LineString::from(points_int)],
        );
        // 一个逆时针方向的菱形外环
        let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let oriented_ext_ls = LineString::from(oriented_ext);
        // 顺时针方向的内环
        let oriented_int_raw = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
        let oriented_int_ls = LineString::from(oriented_int_raw);
        // 构建定向正确的多边形
        let oriented = orient(&poly1, Direction::Default);
        assert_eq!(oriented.exterior().0, oriented_ext_ls.0);
        assert_eq!(oriented.interiors()[0].0, oriented_int_ls.0);
    }
}
