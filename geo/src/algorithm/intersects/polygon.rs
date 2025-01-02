use super::{has_disjoint_bboxes, Intersects};
use crate::coordinate_position::CoordPos;
use crate::{BoundingRect, CoordinatePosition};
use crate::{
    Coord, CoordNum, GeoNum, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect,
    Triangle,
};

impl<T> Intersects<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, p: &Coord<T>) -> bool {
        // 检查点是否不在多边形外部
        self.coordinate_position(p) != CoordPos::Outside
    }
}
symmetric_intersects_impl!(Coord<T>, Polygon<T>);
symmetric_intersects_impl!(Polygon<T>, Point<T>);

impl<T> Intersects<Line<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        // 检查线是否与多边形的外部或内部相交或是否与线段的端点相交
        self.exterior().intersects(line)
            || self.interiors().iter().any(|inner| inner.intersects(line))
            || self.intersects(&line.start)
            || self.intersects(&line.end)
    }
}
symmetric_intersects_impl!(Line<T>, Polygon<T>);
symmetric_intersects_impl!(Polygon<T>, LineString<T>);
symmetric_intersects_impl!(Polygon<T>, MultiLineString<T>);

impl<T> Intersects<Rect<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rect: &Rect<T>) -> bool {
        // 判断多边形是否与矩形转换而来的多边形相交
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Rect<T>, Polygon<T>);

impl<T> Intersects<Triangle<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rect: &Triangle<T>) -> bool {
        // 判断多边形是否与三角形转换而来的多边形相交
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T>, Polygon<T>);

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false; // 如果边界框不相交，则直接返回false
        }

        // 检查self是否与多边形的任何线相交或self是否包含在多边形中
        self.intersects(polygon.exterior())
            || polygon
                .interiors()
                .iter()
                .any(|inner_line_string| self.intersects(inner_line_string))
            || polygon.intersects(self.exterior())
    }
}

// 对于MultiPolygon的实现

impl<G, T> Intersects<G> for MultiPolygon<T>
where
    T: GeoNum,
    Polygon<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false; // 如果边界框不相交，则直接返回false
        }
        // 检查任何MultiPolygon中的多边形是否和rhs相交
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Point<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Line<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Rect<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Triangle<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Polygon<T>, MultiPolygon<T>);

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    // 测试两个Geometry对象是否相交
    fn geom_intersects_geom() {
        let a = Geometry::<f64>::from(polygon![]);
        let b = Geometry::from(polygon![]);
        assert!(!a.intersects(&b));
    }
}
