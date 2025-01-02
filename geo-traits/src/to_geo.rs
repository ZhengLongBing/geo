//! 将实现geo-traits的结构体转换为[geo-types]对象。

use geo_types::{
    Coord, CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

use crate::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    TriangleTrait,
};

/// 将任何坐标转换为[`Coord`]。
///
/// 仅保留前两个维度。
pub trait ToGeoCoord<T: CoordNum> {
    /// 转换为geo_types [`Coord`]。
    fn to_coord(&self) -> Coord<T>;
}

impl<T: CoordNum, G: CoordTrait<T = T>> ToGeoCoord<T> for G {
    fn to_coord(&self) -> Coord<T> {
        Coord {
            x: self.x(),
            y: self.y(),
        }
    }
}

/// 将任何点转换为[`Point`]。
///
/// 仅保留前两个维度。
pub trait ToGeoPoint<T: CoordNum> {
    /// 转换为geo_types [`Point`]。
    ///
    /// # 可能的崩溃
    ///
    /// 空点将导致崩溃。
    fn to_point(&self) -> Point<T> {
        self.try_to_point().expect("geo-types 不支持空点。")
    }

    /// 转换为geo_types [`Point`]。
    ///
    /// 空点将返回`None`。
    fn try_to_point(&self) -> Option<Point<T>>;
}

impl<T: CoordNum, G: PointTrait<T = T>> ToGeoPoint<T> for G {
    fn try_to_point(&self) -> Option<Point<T>> {
        self.coord().map(|coord| Point(coord.to_coord()))
    }
}

/// 将任何线串转换为[`LineString`]。
///
/// 仅保留前两个维度。
pub trait ToGeoLineString<T: CoordNum> {
    /// 转换为geo_types [`LineString`]。
    fn to_line_string(&self) -> LineString<T>;
}

impl<T: CoordNum, G: LineStringTrait<T = T>> ToGeoLineString<T> for G {
    fn to_line_string(&self) -> LineString<T> {
        LineString::new(self.coords().map(|coord| coord.to_coord()).collect())
    }
}

/// 将任何多边形转换为[`Polygon`]。
///
/// 仅保留前两个维度。
pub trait ToGeoPolygon<T: CoordNum> {
    /// 转换为geo_types [`Polygon`]。
    fn to_polygon(&self) -> Polygon<T>;
}

impl<T: CoordNum, G: PolygonTrait<T = T>> ToGeoPolygon<T> for G {
    fn to_polygon(&self) -> Polygon<T> {
        let exterior = if let Some(exterior) = self.exterior() {
            exterior.to_line_string()
        } else {
            LineString::new(vec![])
        };
        let interiors = self
            .interiors()
            .map(|interior| interior.to_line_string())
            .collect();
        Polygon::new(exterior, interiors)
    }
}

/// 将任何多点转换为[`MultiPoint`]。
///
/// 仅保留前两个维度。
pub trait ToGeoMultiPoint<T: CoordNum> {
    /// 转换为geo_types [`MultiPoint`]。
    ///
    /// # 可能的崩溃
    ///
    /// 如果任何包含的点为空，将导致崩溃。
    fn to_multi_point(&self) -> MultiPoint<T> {
        self.try_to_multi_point()
            .expect("geo-types 不支持包含空点的MultiPoint。")
    }

    /// 转换为geo_types [`MultiPoint`]。
    ///
    /// 如果任何包含的点为空，将返回`None`。
    fn try_to_multi_point(&self) -> Option<MultiPoint<T>>;
}

impl<T: CoordNum, G: MultiPointTrait<T = T>> ToGeoMultiPoint<T> for G {
    fn try_to_multi_point(&self) -> Option<MultiPoint<T>> {
        let mut geo_points = vec![];
        for point in self.points() {
            if let Some(geo_point) = point.try_to_point() {
                geo_points.push(geo_point);
            } else {
                // 如果任何点为空，则返回None
                return None;
            }
        }
        Some(MultiPoint::new(geo_points))
    }
}

/// 将任何多线串转换为[`MultiLineString`]。
///
/// 仅保留前两个维度。
pub trait ToGeoMultiLineString<T: CoordNum> {
    /// 转换为geo_types [`MultiLineString`]。
    fn to_multi_line_string(&self) -> MultiLineString<T>;
}

impl<T: CoordNum, G: MultiLineStringTrait<T = T>> ToGeoMultiLineString<T> for G {
    fn to_multi_line_string(&self) -> MultiLineString<T> {
        MultiLineString::new(
            self.line_strings()
                .map(|line_string| line_string.to_line_string())
                .collect(),
        )
    }
}

/// 将任何多边形转换为[`MultiPolygon`]。
///
/// 仅保留前两个维度。
pub trait ToGeoMultiPolygon<T: CoordNum> {
    /// 转换为geo_types [`MultiPolygon`]。
    fn to_multi_polygon(&self) -> MultiPolygon<T>;
}

impl<T: CoordNum, G: MultiPolygonTrait<T = T>> ToGeoMultiPolygon<T> for G {
    fn to_multi_polygon(&self) -> MultiPolygon<T> {
        MultiPolygon::new(
            self.polygons()
                .map(|polygon| polygon.to_polygon())
                .collect(),
        )
    }
}

/// 将任何矩形转换为[`Rect`]。
///
/// 仅保留前两个维度。
pub trait ToGeoRect<T: CoordNum> {
    /// 转换为geo_types [`Rect`]。
    fn to_rect(&self) -> Rect<T>;
}

impl<T: CoordNum, G: RectTrait<T = T>> ToGeoRect<T> for G {
    fn to_rect(&self) -> Rect<T> {
        let c1 = self.min().to_coord();
        let c2 = self.max().to_coord();
        Rect::new(c1, c2)
    }
}

/// 将任何线段转换为[`Line`]。
///
/// 仅保留前两个维度。
pub trait ToGeoLine<T: CoordNum> {
    /// 转换为geo_types [`Line`]。
    fn to_line(&self) -> Line<T>;
}

impl<T: CoordNum, G: LineTrait<T = T>> ToGeoLine<T> for G {
    fn to_line(&self) -> Line<T> {
        let start = self.start().to_coord();
        let end = self.end().to_coord();
        Line::new(start, end)
    }
}

/// 将任何三角形转换为[`Triangle`]。
///
/// 仅保留前两个维度。
pub trait ToGeoTriangle<T: CoordNum> {
    /// 转换为geo_types [`Triangle`]。
    fn to_triangle(&self) -> Triangle<T>;
}

impl<T: CoordNum, G: TriangleTrait<T = T>> ToGeoTriangle<T> for G {
    fn to_triangle(&self) -> Triangle<T> {
        let v1 = self.first().to_coord();
        let v2 = self.second().to_coord();
        let v3 = self.third().to_coord();
        Triangle::new(v1, v2, v3)
    }
}

/// 将任何几何体转换为[`Geometry`]。
///
/// 仅保留前两个维度。
pub trait ToGeoGeometry<T: CoordNum> {
    /// 转换为geo_types [`Geometry`]。
    ///
    /// # 可能的崩溃
    ///
    /// 空点或包含空点的MultiPoint将导致崩溃。
    fn to_geometry(&self) -> Geometry<T> {
        self.try_to_geometry()
            .expect("geo-types 不支持空点或包含空点的MultiPoint。")
    }

    /// 转换为geo_types [`Geometry`]。
    ///
    /// 空几何体将返回`None`。
    fn try_to_geometry(&self) -> Option<Geometry<T>>;
}

impl<T: CoordNum, G: GeometryTrait<T = T>> ToGeoGeometry<T> for G {
    fn try_to_geometry(&self) -> Option<Geometry<T>> {
        use GeometryType::*;

        match self.as_type() {
            Point(geom) => geom.try_to_point().map(Geometry::Point),
            LineString(geom) => Some(Geometry::LineString(geom.to_line_string())),
            Polygon(geom) => Some(Geometry::Polygon(geom.to_polygon())),
            MultiPoint(geom) => geom.try_to_multi_point().map(Geometry::MultiPoint),
            MultiLineString(geom) => Some(Geometry::MultiLineString(geom.to_multi_line_string())),
            MultiPolygon(geom) => Some(Geometry::MultiPolygon(geom.to_multi_polygon())),
            GeometryCollection(geom) => geom
                .try_to_geometry_collection()
                .map(Geometry::GeometryCollection),
            Rect(geom) => Some(Geometry::Rect(geom.to_rect())),
            Line(geom) => Some(Geometry::Line(geom.to_line())),
            Triangle(geom) => Some(Geometry::Triangle(geom.to_triangle())),
        }
    }
}

/// 将任何几何体集合转换为[`GeometryCollection`]。
///
/// 仅保留前两个维度。
pub trait ToGeoGeometryCollection<T: CoordNum> {
    /// 转换为geo_types [`GeometryCollection`]。
    ///
    /// # 可能的崩溃
    ///
    /// 空点或包含空点的MultiPoint将导致崩溃。
    fn to_geometry_collection(&self) -> GeometryCollection<T> {
        self.try_to_geometry_collection()
            .expect("geo-types 不支持空GeometryCollections。")
    }

    /// 转换为geo_types [`GeometryCollection`]。
    ///
    /// 空点或包含空点的MultiPoint将返回`None`。
    fn try_to_geometry_collection(&self) -> Option<GeometryCollection<T>>;
}

impl<T: CoordNum, G: GeometryCollectionTrait<T = T>> ToGeoGeometryCollection<T> for G {
    fn try_to_geometry_collection(&self) -> Option<GeometryCollection<T>> {
        let mut geo_geometries = vec![];
        for geom in self.geometries() {
            if let Some(geo_geom) = geom.try_to_geometry() {
                geo_geometries.push(geo_geom);
            } else {
                // 如果任何点为空，则返回None
                return None;
            }
        }
        Some(GeometryCollection::new_from(geo_geometries))
    }
}
