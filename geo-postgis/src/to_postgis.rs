use geo_types::{
    Coord, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use postgis::ewkb;

/// 将几何体转换为 PostGIS 类型。
///
/// 请注意，PostGIS 数据库可以包含存储在其中的几何体的 SRID（空间参考系统标识符）。
/// 在转换时，您应该指定几何体的 SRID，使用 `to_postgis_with_srid()`，
/// 或者如果您的数据是标准的 WGS84，则使用 `to_postgis_wgs84()`。
pub trait ToPostgis<T> {
    /// 使用提供的 SRID 将此几何体转换为 PostGIS 类型。
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> T;
    /// 将此 WGS84 几何体转换为 PostGIS 类型。
    fn to_postgis_wgs84(&self) -> T {
        self.to_postgis_with_srid(Some(4326))
    }
}

impl ToPostgis<ewkb::Point> for Coord {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Point {
        ewkb::Point::new(self.x, self.y, srid)
    }
}

impl ToPostgis<ewkb::Point> for Point {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Point {
        ewkb::Point::new(self.x(), self.y(), srid)
    }
}

impl ToPostgis<ewkb::LineString> for Line {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::LineString {
        let points = vec![
            self.start_point().to_postgis_with_srid(srid),
            self.end_point().to_postgis_with_srid(srid),
        ];
        ewkb::LineString { points, srid }
    }
}

impl ToPostgis<ewkb::Polygon> for Polygon<f64> {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Polygon {
        let rings = ::std::iter::once(self.exterior())
            .chain(self.interiors().iter())
            .map(|x| (*x).to_postgis_with_srid(srid))
            .collect();
        ewkb::Polygon { rings, srid }
    }
}

macro_rules! to_postgis_impl {
    ($from:ident, $to:path, $name:ident) => {
        impl ToPostgis<$to> for $from<f64> {
            fn to_postgis_with_srid(&self, srid: Option<i32>) -> $to {
                let $name = self
                    .0
                    .iter()
                    .map(|x| x.to_postgis_with_srid(srid))
                    .collect();
                $to { $name, srid }
            }
        }
    };
}

to_postgis_impl!(GeometryCollection, ewkb::GeometryCollection, geometries);
to_postgis_impl!(MultiPolygon, ewkb::MultiPolygon, polygons);
to_postgis_impl!(MultiLineString, ewkb::MultiLineString, lines);
to_postgis_impl!(MultiPoint, ewkb::MultiPoint, points);
to_postgis_impl!(LineString, ewkb::LineString, points);

impl ToPostgis<ewkb::Geometry> for Geometry {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Geometry {
        match *self {
            Geometry::Point(ref p) => ewkb::GeometryT::Point(p.to_postgis_with_srid(srid)),
            Geometry::Line(ref p) => ewkb::GeometryT::LineString(p.to_postgis_with_srid(srid)),
            Geometry::LineString(ref p) => {
                ewkb::GeometryT::LineString(p.to_postgis_with_srid(srid))
            }
            Geometry::Polygon(ref p) => ewkb::GeometryT::Polygon(p.to_postgis_with_srid(srid)),
            Geometry::MultiPoint(ref p) => {
                ewkb::GeometryT::MultiPoint(p.to_postgis_with_srid(srid))
            }
            Geometry::MultiLineString(ref p) => {
                ewkb::GeometryT::MultiLineString(p.to_postgis_with_srid(srid))
            }
            Geometry::MultiPolygon(ref p) => {
                ewkb::GeometryT::MultiPolygon(p.to_postgis_with_srid(srid))
            }
            Geometry::GeometryCollection(ref p) => {
                ewkb::GeometryT::GeometryCollection(p.to_postgis_with_srid(srid))
            }
            Geometry::Rect(ref p) => {
                ewkb::GeometryT::Polygon(p.to_polygon().to_postgis_with_srid(srid))
            }
            Geometry::Triangle(ref p) => {
                ewkb::GeometryT::Polygon(p.to_polygon().to_postgis_with_srid(srid))
            }
        }
    }
}
