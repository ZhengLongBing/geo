use crate::{
    CoordNum, Geometry, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon, Rect, Triangle,
};
use geo_types::GeometryCollection;

/// 从 `MultiPoint` 中移除重复点，以及从 `LineString`、`Polygon`、`MultiLineString` 和 `MultiPolygon` 中移除重复的连续坐标。
///
/// 对于 `GeometryCollection`，单独移除集合中每个几何体的重复点。
///
/// 对于 `Point`、`Line`、`Rect` 和 `Triangle`，几何体保持不变。
pub trait RemoveRepeatedPoints<T: CoordNum> {
    /// 创建一个去除（连续）重复点的新几何对象。
    fn remove_repeated_points(&self) -> Self;
    /// 就地移除（连续）重复点。
    fn remove_repeated_points_mut(&mut self);
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for MultiPoint<T> {
    /// 创建一个去除重复点的 MultiPoint。
    fn remove_repeated_points(&self) -> Self {
        let mut points = vec![];
        for p in self.0.iter() {
            if !points.contains(p) {
                points.push(*p);
            }
        }
        MultiPoint(points)
    }

    /// 就地从 MultiPoint 中移除重复点。
    fn remove_repeated_points_mut(&mut self) {
        let mut points = vec![];
        for p in self.0.iter() {
            if !points.contains(p) {
                points.push(*p);
            }
        }
        self.0 = points;
    }
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for LineString<T> {
    /// 创建一个去除连续重复点的 LineString。
    fn remove_repeated_points(&self) -> Self {
        let mut coords = self.0.clone();
        coords.dedup();
        LineString(coords)
    }

    /// 就地移除 LineString 中连续的重复点。
    fn remove_repeated_points_mut(&mut self) {
        self.0.dedup();
    }
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for Polygon<T> {
    /// 创建一个去除连续重复点的 Polygon。
    fn remove_repeated_points(&self) -> Self {
        Polygon::new(
            self.exterior().remove_repeated_points(),
            self.interiors()
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }

    /// 就地移除 Polygon 中连续的重复点。
    fn remove_repeated_points_mut(&mut self) {
        self.exterior_mut(|exterior| exterior.remove_repeated_points_mut());
        self.interiors_mut(|interiors| {
            for interior in interiors {
                interior.remove_repeated_points_mut();
            }
        });
    }
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for MultiLineString<T> {
    /// 创建一个去除连续重复点的 MultiLineString。
    fn remove_repeated_points(&self) -> Self {
        MultiLineString::new(
            self.0
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }

    /// 就地移除 MultiLineString 中连续的重复点。
    fn remove_repeated_points_mut(&mut self) {
        for ls in self.0.iter_mut() {
            ls.remove_repeated_points_mut();
        }
    }
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for MultiPolygon<T> {
    /// 创建一个去除连续重复点的 MultiPolygon。
    fn remove_repeated_points(&self) -> Self {
        MultiPolygon::new(self.0.iter().map(|p| p.remove_repeated_points()).collect())
    }

    /// 就地移除 MultiPolygon 中连续的重复点。
    fn remove_repeated_points_mut(&mut self) {
        for p in self.0.iter_mut() {
            p.remove_repeated_points_mut();
        }
    }
}

// 对于不适合坐标移除的类型的实现
// （Point / Line / Triangle / Rect），`remove_repeated_points` 返回几何对象的克隆，
// 而 `remove_repeated_points_mut` 是一个无操作（no-op）。
macro_rules! impl_for_not_candidate_types {
    ($type:ident) => {
        impl<T: CoordNum> RemoveRepeatedPoints<T> for $type<T> {
            fn remove_repeated_points(&self) -> Self {
                self.clone()
            }

            fn remove_repeated_points_mut(&mut self) {
                // 无操作
            }
        }
    };
}

impl_for_not_candidate_types!(Point);
impl_for_not_candidate_types!(Rect);
impl_for_not_candidate_types!(Triangle);
impl_for_not_candidate_types!(Line);

impl<T: CoordNum> RemoveRepeatedPoints<T> for GeometryCollection<T> {
    /// 创建一个去除其几何体的（连续）重复点的 GeometryCollection。
    fn remove_repeated_points(&self) -> Self {
        GeometryCollection::new_from(self.0.iter().map(|g| g.remove_repeated_points()).collect())
    }

    /// 就地从 GeometryCollection 中移除其几何体的（连续）重复点。
    fn remove_repeated_points_mut(&mut self) {
        for g in self.0.iter_mut() {
            g.remove_repeated_points_mut();
        }
    }
}

impl<T: CoordNum> RemoveRepeatedPoints<T> for Geometry<T> {
    // 在实现 "impl<T: CoordNum> From<GeometryCollection<T>> for Geometry<T>" 之前，
    // 下面的内容无法用于实现 `remove_repeated_points`
    // （参见 geo-types/src/geometry/mod.rs，第 101-106 行），所以我们暂时手动实现
    //
    //   crate::geometry_delegate_impl! {
    //       fn remove_repeated_points(&self) -> Geometry<T>;
    //   }

    /// 创建一个去除连续重复点的几何体。
    fn remove_repeated_points(&self) -> Self {
        match self {
            Geometry::Point(p) => Geometry::Point(p.remove_repeated_points()),
            Geometry::Line(l) => Geometry::Line(l.remove_repeated_points()),
            Geometry::LineString(ls) => Geometry::LineString(ls.remove_repeated_points()),
            Geometry::Polygon(p) => Geometry::Polygon(p.remove_repeated_points()),
            Geometry::MultiPoint(mp) => Geometry::MultiPoint(mp.remove_repeated_points()),
            Geometry::MultiLineString(mls) => {
                Geometry::MultiLineString(mls.remove_repeated_points())
            }
            Geometry::MultiPolygon(mp) => Geometry::MultiPolygon(mp.remove_repeated_points()),
            Geometry::Rect(r) => Geometry::Rect(r.remove_repeated_points()),
            Geometry::Triangle(t) => Geometry::Triangle(t.remove_repeated_points()),
            Geometry::GeometryCollection(gc) => {
                Geometry::GeometryCollection(gc.remove_repeated_points())
            }
        }
    }

    /// 就地从几何体中移除连续重复点。
    fn remove_repeated_points_mut(&mut self) {
        match self {
            Geometry::Point(p) => p.remove_repeated_points_mut(),
            Geometry::Line(l) => l.remove_repeated_points_mut(),
            Geometry::LineString(ls) => ls.remove_repeated_points_mut(),
            Geometry::Polygon(p) => p.remove_repeated_points_mut(),
            Geometry::MultiPoint(mp) => mp.remove_repeated_points_mut(),
            Geometry::MultiLineString(mls) => mls.remove_repeated_points_mut(),
            Geometry::MultiPolygon(mp) => mp.remove_repeated_points_mut(),
            Geometry::Rect(r) => r.remove_repeated_points_mut(),
            Geometry::Triangle(t) => t.remove_repeated_points_mut(),
            Geometry::GeometryCollection(gc) => gc.remove_repeated_points_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::RemoveRepeatedPoints;
    use crate::{
        Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
        Polygon,
    };

    fn make_test_mp_integer() -> MultiPoint<i32> {
        MultiPoint(vec![
            Point::new(0, 0),
            Point::new(1, 1),
            Point::new(1, 1),
            Point::new(1, 1),
            Point::new(2, 2),
            Point::new(0, 0),
        ])
    }

    fn make_result_mp_integer() -> MultiPoint<i32> {
        MultiPoint(vec![Point::new(0, 0), Point::new(1, 1), Point::new(2, 2)])
    }

    fn make_test_mp1() -> MultiPoint {
        MultiPoint(vec![
            Point::new(0., 0.),
            Point::new(1., 1.),
            Point::new(1., 1.),
            Point::new(1., 1.),
            Point::new(2., 2.),
            Point::new(0., 0.),
        ])
    }

    fn make_result_mp1() -> MultiPoint {
        MultiPoint(vec![
            Point::new(0., 0.),
            Point::new(1., 1.),
            Point::new(2., 2.),
        ])
    }

    fn make_test_line1() -> LineString {
        LineString(vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 1. },
            Coord { x: 1., y: 1. },
            Coord { x: 1., y: 1. },
            Coord { x: 2., y: 2. },
            Coord { x: 2., y: 2. },
            Coord { x: 0., y: 0. },
        ])
    }

    fn make_result_line1() -> LineString {
        LineString(vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 1. },
            Coord { x: 2., y: 2. },
            Coord { x: 0., y: 0. },
        ])
    }

    fn make_test_line2() -> LineString {
        LineString(vec![
            Coord { x: 10., y: 10. },
            Coord { x: 11., y: 11. },
            Coord { x: 11., y: 11. },
            Coord { x: 11., y: 11. },
            Coord { x: 12., y: 12. },
            Coord { x: 12., y: 12. },
            Coord { x: 10., y: 10. },
        ])
    }

    fn make_result_line2() -> LineString {
        LineString(vec![
            Coord { x: 10., y: 10. },
            Coord { x: 11., y: 11. },
            Coord { x: 12., y: 12. },
            Coord { x: 10., y: 10. },
        ])
    }

    fn make_test_poly1() -> Polygon {
        Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        )
    }

    fn make_result_poly1() -> Polygon {
        Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        )
    }

    fn make_test_poly2() -> Polygon {
        Polygon::new(
            LineString(vec![
                Coord { x: 10., y: 10. },
                Coord { x: 11., y: 11. },
                Coord { x: 11., y: 11. },
                Coord { x: 11., y: 11. },
                Coord { x: 10., y: 12. },
                Coord { x: 10., y: 12. },
                Coord { x: 10., y: 10. },
            ]),
            vec![],
        )
    }

    fn make_result_poly2() -> Polygon {
        Polygon::new(
            LineString(vec![
                Coord { x: 10., y: 10. },
                Coord { x: 11., y: 11. },
                Coord { x: 10., y: 12. },
                Coord { x: 10., y: 10. },
            ]),
            vec![],
        )
    }

    #[test]
    fn test_remove_repeated_points_multipoint_integer() {
        let mp = make_test_mp_integer();
        let expected = make_result_mp_integer();

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipoint() {
        let mp = make_test_mp1();
        let expected = make_result_mp1();

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_linestring() {
        let ls = make_test_line1();
        let expected = make_result_line1();

        assert_eq!(ls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_polygon() {
        let poly = make_test_poly1();
        let expected = make_result_poly1();

        assert_eq!(poly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multilinestring() {
        let mls = MultiLineString(vec![make_test_line1(), make_test_line2()]);

        let expected = MultiLineString(vec![make_result_line1(), make_result_line2()]);

        assert_eq!(mls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipolygon() {
        let mpoly = MultiPolygon(vec![make_test_poly1(), make_test_poly2()]);

        let expected = MultiPolygon(vec![make_result_poly1(), make_result_poly2()]);

        assert_eq!(mpoly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_geometrycollection() {
        let gc = GeometryCollection::new_from(vec![
            make_test_mp1().into(),
            make_test_line1().into(),
            make_test_poly1().into(),
        ]);

        let expected = GeometryCollection::new_from(vec![
            make_result_mp1().into(),
            make_result_line1().into(),
            make_result_poly1().into(),
        ]);

        assert_eq!(gc.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipoint_integer() {
        let mut mp = make_test_mp_integer();
        mp.remove_repeated_points_mut();
        let expected = make_result_mp_integer();

        assert_eq!(mp, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipoint() {
        let mut mp = make_test_mp1();
        mp.remove_repeated_points_mut();
        let expected = make_result_mp1();

        assert_eq!(mp, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_linestring() {
        let mut ls = make_test_line1();
        ls.remove_repeated_points_mut();
        let expected = make_result_line1();

        assert_eq!(ls, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_polygon() {
        let mut poly = make_test_poly1();
        poly.remove_repeated_points_mut();
        let expected = make_result_poly1();

        assert_eq!(poly, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multilinestring() {
        let mut mls = MultiLineString(vec![make_test_line1(), make_test_line2()]);
        mls.remove_repeated_points_mut();
        let expected = MultiLineString(vec![make_result_line1(), make_result_line2()]);

        assert_eq!(mls, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipolygon() {
        let mut mpoly = MultiPolygon(vec![make_test_poly1(), make_test_poly2()]);
        mpoly.remove_repeated_points_mut();
        let expected = MultiPolygon(vec![make_result_poly1(), make_result_poly2()]);

        assert_eq!(mpoly, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_geometrycollection() {
        let mut gc = GeometryCollection::new_from(vec![
            make_test_mp1().into(),
            make_test_line1().into(),
            make_test_poly1().into(),
        ]);
        gc.remove_repeated_points_mut();

        let expected = GeometryCollection::new_from(vec![
            make_result_mp1().into(),
            make_result_line1().into(),
            make_result_poly1().into(),
        ]);

        assert_eq!(gc, expected);
    }
}
