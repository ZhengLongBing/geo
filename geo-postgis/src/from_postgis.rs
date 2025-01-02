use geo_types::{
    Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

use postgis::ewkb::{GeometryCollectionT, GeometryT};

#[cfg_attr(docsrs, doc(cfg(feature = "postgis")))]
/// 从PostGIS类型创建几何对象。
///
/// 请注意，PostGIS数据库可以在任何空间参考系统下存储数据 —— 不仅仅是WGS84。
/// 不会尝试在参考系统之间转换数据。
pub trait FromPostgis<T> {
    fn from_postgis(_: T) -> Self;
}

impl<'a, T> FromPostgis<&'a T> for Point
where
    T: postgis::Point,
{
    fn from_postgis(pt: &'a T) -> Self {
        Point::new(pt.x(), pt.y())
    }
}
impl<'a, T> FromPostgis<&'a T> for LineString
where
    T: postgis::LineString<'a>,
{
    fn from_postgis(ls: &'a T) -> Self {
        let ret: Vec<Point> = ls.points().map(Point::from_postgis).collect();
        LineString::from(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for Option<Polygon<f64>>
where
    T: postgis::Polygon<'a>,
{
    /// 这将返回一个`Option`，因为一个PostGIS`Polygon`可能不包含任何环，
    /// 这会导致一个无效的`geo::Polygon`。
    fn from_postgis(poly: &'a T) -> Self {
        let mut rings = poly
            .rings()
            .map(LineString::from_postgis)
            .collect::<Vec<_>>();
        if rings.is_empty() {
            return None;
        }
        let exterior = rings.remove(0);
        Some(Polygon::new(exterior, rings))
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiPoint
where
    T: postgis::MultiPoint<'a>,
{
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.points().map(Point::from_postgis).collect();
        MultiPoint::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiLineString
where
    T: postgis::MultiLineString<'a>,
{
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.lines().map(LineString::from_postgis).collect();
        MultiLineString::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiPolygon
where
    T: postgis::MultiPolygon<'a>,
{
    /// 此实现丢弃不能转换的PostGIS多边形
    /// （当调用`from_postgis()`时返回`None`）。
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.polygons().filter_map(Option::from_postgis).collect();
        MultiPolygon::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a GeometryCollectionT<T>> for GeometryCollection
where
    T: postgis::Point + postgis::ewkb::EwkbRead,
{
    /// 此实现丢弃不能转换的几何体
    /// （当调用`from_postgis()`时返回`None`）。
    fn from_postgis(gc: &'a GeometryCollectionT<T>) -> Self {
        let geoms = gc
            .geometries
            .iter()
            .filter_map(Option::from_postgis)
            .collect();
        GeometryCollection::new_from(geoms)
    }
}
impl<'a, T> FromPostgis<&'a GeometryT<T>> for Option<Geometry>
where
    T: postgis::Point + postgis::ewkb::EwkbRead,
{
    /// 返回一个`Option`，因为提供的几何体可能是一个无效的`Polygon`。
    fn from_postgis(geo: &'a GeometryT<T>) -> Self {
        Some(match *geo {
            GeometryT::Point(ref p) => Geometry::Point(Point::from_postgis(p)),
            GeometryT::LineString(ref ls) => Geometry::LineString(LineString::from_postgis(ls)),
            GeometryT::Polygon(ref p) => match Option::from_postgis(p) {
                Some(p) => Geometry::Polygon(p),
                None => return None,
            },
            GeometryT::MultiPoint(ref p) => Geometry::MultiPoint(MultiPoint::from_postgis(p)),
            GeometryT::MultiLineString(ref p) => {
                Geometry::MultiLineString(MultiLineString::from_postgis(p))
            }
            GeometryT::MultiPolygon(ref p) => Geometry::MultiPolygon(MultiPolygon::from_postgis(p)),
            GeometryT::GeometryCollection(ref p) => {
                Geometry::GeometryCollection(GeometryCollection::from_postgis(p))
            }
        })
    }
}
