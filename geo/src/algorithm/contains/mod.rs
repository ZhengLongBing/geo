/// 检查 `rhs` 是否完全包含在 `self` 内部。
/// 更正式地说，`rhs` 的内部与 `self` 形成非空的（集合理论上的）交集，
/// 但 `rhs` 的内部和边界都不与 `self` 的外部相交。
/// 换句话说，`(rhs, self)` 的 [DE-9IM] 交集矩阵为 `T*F**F***`。
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # 示例
///
/// ```
/// use geo::Contains;
/// use geo::{line_string, point, Polygon};
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 2., y: 0.),
///     (x: 2., y: 2.),
///     (x: 0., y: 2.),
///     (x: 0., y: 0.),
/// ];
///
/// let polygon = Polygon::new(line_string.clone(), vec![]);
///
/// // 点包含在点中
/// assert!(point!(x: 2., y: 0.).contains(&point!(x: 2., y: 0.)));
///
/// // 点包含在线串中
/// assert!(line_string.contains(&point!(x: 2., y: 0.)));
///
/// // 点包含在多边形中
/// assert!(polygon.contains(&point!(x: 1., y: 1.)));
/// ```
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

mod geometry;
mod geometry_collection;
mod line;
mod line_string;
mod point;
mod polygon;
mod rect;
mod triangle;

macro_rules! impl_contains_from_relate {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T> Contains<$target> for $for
            where
                T: GeoFloat
            {
                fn contains(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    self.relate(target).is_contains()
                }
            }
        )*
    };
}
pub(crate) use impl_contains_from_relate;

macro_rules! impl_contains_geometry_for {
    ($geom_type: ty) => {
        impl<T> Contains<Geometry<T>> for $geom_type
        where
            T: GeoFloat,
        {
            fn contains(&self, geometry: &Geometry<T>) -> bool {
                match geometry {
                    Geometry::Point(g) => self.contains(g),
                    Geometry::Line(g) => self.contains(g),
                    Geometry::LineString(g) => self.contains(g),
                    Geometry::Polygon(g) => self.contains(g),
                    Geometry::MultiPoint(g) => self.contains(g),
                    Geometry::MultiLineString(g) => self.contains(g),
                    Geometry::MultiPolygon(g) => self.contains(g),
                    Geometry::GeometryCollection(g) => self.contains(g),
                    Geometry::Rect(g) => self.contains(g),
                    Geometry::Triangle(g) => self.contains(g),
                }
            }
        }
    };
}
pub(crate) use impl_contains_geometry_for;

// ┌───────┐
// │ 测试  │
// └───────┘

#[cfg(test)]
mod test {
    use crate::line_string;
    use crate::Contains;
    use crate::Relate;
    use crate::{coord, Coord, Line, LineString, MultiPolygon, Point, Polygon, Rect, Triangle};

    #[test]
    // 查看 https://github.com/georust/geo/issues/452
    fn linestring_contains_point() {
        let line_string = LineString::from(vec![(0., 0.), (3., 3.)]);
        let point_on_line = Point::new(1., 1.);
        assert!(line_string.contains(&point_on_line));
    }
    #[test]
    // V 不包含 rect，因为它的两条边与 V 的外部边界相交
    fn polygon_does_not_contain_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (250., 310.),
                (150., 310.),
                (150., 280.),
                (250., 280.),
                (250., 310.),
            ]
            .into(),
            vec![],
        );
        assert!(!v.contains(&rect));
    }
    #[test]
    // V 包含 rect，因为 rect 的所有顶点都被包含，并且没有它的边与 V 的边界相交
    fn polygon_contains_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (185., 237.),
                (220., 237.),
                (220., 220.),
                (185., 220.),
                (185., 237.),
            ]
            .into(),
            vec![],
        );
        assert!(v.contains(&rect));
    }
    #[test]
    // LineString 被完全包含
    fn linestring_fully_contained_in_polygon() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![],
        );
        let ls = LineString::from(vec![(3.0, 0.5), (3.0, 3.5)]);
        assert!(poly.contains(&ls));
    }
    /// 测试：点在 LineString 中
    #[test]
    fn empty_linestring_test() {
        let linestring = LineString::new(Vec::new());
        assert!(!linestring.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn linestring_point_is_vertex_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        // 注意：LineString 的终点不被视为“包含”
        assert!(linestring.contains(&Point::new(2., 0.)));
        assert!(!linestring.contains(&Point::new(0., 0.)));
        assert!(!linestring.contains(&Point::new(2., 2.)));
    }
    #[test]
    fn linestring_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        assert!(linestring.contains(&Point::new(1., 0.)));
    }
    /// 测试：点在多边形中
    #[test]
    fn empty_polygon_test() {
        let linestring = LineString::new(Vec::new());
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_one_point_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(1., 0.)));
        assert!(!poly.contains(&Point::new(2., 1.)));
        assert!(!poly.contains(&Point::new(1., 2.)));
        assert!(!poly.contains(&Point::new(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_test() {
        let linestring = LineString::from(vec![(1., 0.), (0., 1.), (-1., 0.), (0., -1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(0., 0.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_and_not_crossing() {
        let linestring = LineString::from(vec![
            (0., 0.),
            (2., 0.),
            (3., 1.),
            (4., 0.),
            (4., 2.),
            (0., 2.),
            (0., 0.),
        ]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2.1, 1.)));
        assert!(!poly.contains(&Point::new(1., 2.1)));
        assert!(!poly.contains(&Point::new(2.1, 2.1)));
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let inner_linestring = LineString::from(vec![
            [0.5, 0.5],
            [1.5, 0.5],
            [1.5, 1.5],
            [0.0, 1.5],
            [0.0, 0.0],
        ]);
        let poly = Polygon::new(linestring, vec![inner_linestring]);
        assert!(!poly.contains(&Point::new(0.25, 0.25)));
        assert!(!poly.contains(&Point::new(1., 1.)));
        assert!(!poly.contains(&Point::new(1.5, 1.5)));
        assert!(!poly.contains(&Point::new(1.5, 1.)));
    }

    /// 测试：多边形中包含点
    #[test]
    fn empty_multipolygon_test() {
        let multipoly = MultiPolygon::new(Vec::new());
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.), (0., 0.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(2., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 0.)]),
            Vec::new(),
        );
        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 1.),
            ])],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(9., 0.), (14., 0.), (14., 4.), (9., 4.), (9., 0.)]),
            Vec::new(),
        );

        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(3., 5.)));
        assert!(multipoly.contains(&Point::new(12., 2.)));
        assert!(!multipoly.contains(&Point::new(3., 2.)));
        assert!(!multipoly.contains(&Point::new(7., 2.)));
    }
    /// 测试：LineString 在多边形中
    #[test]
    fn linestring_in_polygon_with_linestring_is_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring.clone(), Vec::new());
        assert!(!poly.contains(&linestring));
        assert!(!poly.contains(&LineString::from(vec![(0., 0.), (2., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 0.), (2., 2.)])));
        assert!(!poly.contains(&LineString::from(vec![(0., 2.), (0., 0.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&LineString::from(vec![(1., 1.), (3., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.), (5., 2.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 4.),
                (1., 1.),
            ])],
        );
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (3., 3.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (2., 5.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.5), (3., 5.)])));
    }
    #[test]
    fn bounding_rect_in_inner_bounding_rect_test() {
        let bounding_rect_xl =
            Rect::new(coord! { x: -100., y: -200. }, coord! { x: 100., y: 200. });
        let bounding_rect_sm = Rect::new(coord! { x: -10., y: -20. }, coord! { x: 10., y: 20. });
        assert!(bounding_rect_xl.contains(&bounding_rect_sm));
        assert!(!bounding_rect_sm.contains(&bounding_rect_xl));
    }
    #[test]
    fn point_in_line_test() {
        let c = |x, y| coord! { x: x, y: y };
        let p0 = c(2., 4.);
        // 垂直线
        let line1 = Line::new(c(2., 0.), c(2., 5.));
        // 点在直线上，但在线段之外
        let line2 = Line::new(c(0., 6.), c(1.5, 4.5));
        // 点在直线上
        let line3 = Line::new(c(0., 6.), c(3., 3.));
        assert!(line1.contains(&Point::from(p0)));
        assert!(!line2.contains(&Point::from(p0)));
        assert!(line3.contains(&Point::from(p0)));
    }
    #[test]
    fn line_in_line_test() {
        let c = |x, y| coord! { x: x, y: y };
        let line0 = Line::new(c(0., 1.), c(3., 4.));
        // 第一个点在线 line0 上，第二个不在
        let line1 = Line::new(c(1., 2.), c(2., 2.));
        // 共线，但延伸超出 line0 的末端
        let line2 = Line::new(c(1., 2.), c(4., 5.));
        // 包含在 line0 中
        let line3 = Line::new(c(1., 2.), c(3., 4.));
        assert!(!line0.contains(&line1));
        assert!(!line0.contains(&line2));
        assert!(line0.contains(&line3));
    }
    #[test]
    fn linestring_in_line_test() {
        let line = Line::from([(0, 10), (30, 40)]);
        // linestring0 在 line 中
        let linestring0 = LineString::from(vec![(1, 11), (10, 20), (15, 25)]);
        // linestring1 的起点和终点在线上，但中间部分偏离
        let linestring1 = LineString::from(vec![(1, 11), (20, 20), (15, 25)]);
        // linestring2 是共线的，但超出了 line 的范围
        let linestring2 = LineString::from(vec![(1, 11), (10, 20), (40, 50)]);
        // linestring3 的任何部分都不在线上
        let linestring3 = LineString::from(vec![(11, 11), (20, 20), (25, 25)]);
        // 一个内点在直线上边界的 linestring
        let linestring4 = LineString::from(vec![(0, 10), (0, 10), (0, 10)]);
        // 一个内点包含在直线内的 linestring
        let linestring5 = LineString::from(vec![(1, 11), (1, 11), (1, 11)]);
        assert!(line.contains(&linestring0));
        assert!(!line.contains(&linestring1));
        assert!(!line.contains(&linestring2));
        assert!(!line.contains(&linestring3));
        assert!(!line.contains(&linestring4));
        assert!(line.contains(&linestring5));
    }
    #[test]
    fn line_in_polygon_test() {
        let c = |x, y| coord! { x: x, y: y };
        let line = Line::new(c(0.0, 10.0), c(30.0, 40.0));
        let linestring0 = line_string![
            c(-10.0, 0.0),
            c(50.0, 0.0),
            c(50.0, 50.0),
            c(0.0, 50.0),
            c(-10.0, 0.0)
        ];
        let poly0 = Polygon::new(linestring0, Vec::new());
        let linestring1 = line_string![
            c(0.0, 0.0),
            c(0.0, 20.0),
            c(20.0, 20.0),
            c(20.0, 0.0),
            c(0.0, 0.0)
        ];
        let poly1 = Polygon::new(linestring1, Vec::new());
        assert!(poly0.contains(&line));
        assert!(!poly1.contains(&line));
    }
    #[test]
    fn line_in_polygon_edgecases_test() {
        // 某些 DE-9IM 边界情形用于检查线是否在多边形中 线的终点可以
        // 在多边形的边界上。
        let c = |x, y| coord! { x: x, y: y };
        // 非凸多边形
        let linestring0 = line_string![
            c(0.0, 0.0),
            c(1.0, 1.0),
            c(1.0, -1.0),
            c(-1.0, -1.0),
            c(-1.0, 1.0)
        ];
        let poly = Polygon::new(linestring0, Vec::new());

        assert!(poly.contains(&Line::new(c(0.0, 0.0), c(1.0, -1.0))));
        assert!(poly.contains(&Line::new(c(-1.0, 1.0), c(1.0, -1.0))));
        assert!(!poly.contains(&Line::new(c(-1.0, 1.0), c(1.0, 1.0))));
    }
    #[test]
    fn line_in_linestring_edgecases() {
        let c = |x, y| coord! { x: x, y: y };
        use crate::line_string;
        let mut ls = line_string![c(0, 0), c(1, 0), c(0, 1), c(-1, 0)];
        assert!(!ls.contains(&Line::from([(0, 0), (0, 0)])));
        ls.close();
        assert!(ls.contains(&Line::from([(0, 0), (0, 0)])));
        assert!(ls.contains(&Line::from([(-1, 0), (1, 0)])));
    }
    #[test]
    fn line_in_linestring_test() {
        let line0 = Line::from([(1., 1.), (2., 2.)]);
        // line0 完全包含在第二段中
        let linestring0 = LineString::from(vec![(0., 0.5), (0.5, 0.5), (3., 3.)]);
        // line0 包含在最后三段中
        let linestring1 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 1.5),
            (3., 3.),
        ]);
        // line0 的端点在 linestring 中，但第四点不在线上
        let linestring2 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 0.),
            (2., 2.),
            (3., 3.),
        ]);
        assert!(linestring0.contains(&line0));
        assert!(linestring1.contains(&line0));
        assert!(!linestring2.contains(&line0));
    }

    #[test]
    fn integer_bounding_rects() {
        let p: Point<i32> = Point::new(10, 20);
        let bounding_rect: Rect<i32> = Rect::new(coord! { x: 0, y: 0 }, coord! { x: 100, y: 100 });
        assert!(bounding_rect.contains(&p));
        assert!(!bounding_rect.contains(&Point::new(-10, -10)));

        let smaller_bounding_rect: Rect<i32> =
            Rect::new(coord! { x: 10, y: 10 }, coord! { x: 20, y: 20 });
        assert!(bounding_rect.contains(&smaller_bounding_rect));
    }

    #[test]
    fn triangle_not_contains_point_on_edge() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_on_vertex() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(2.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_point_inside() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.5);
        assert!(t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_above() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 1.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_below() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(-1.0, 0.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_neg_point() {
        let t = Triangle::from([(0.0, 0.0), (-2.0, 0.0), (-2.0, -2.0)]);
        let p = Point::new(-1.0, -0.5);
        assert!(t.contains(&p));
    }

    #[test]
    // https://github.com/georust/geo/issues/473
    fn triangle_contains_collinear_points() {
        let origin: Coord = (0., 0.).into();
        let tri = Triangle::new(origin, origin, origin);
        let pt: Point = (0., 1.23456).into();
        assert!(!tri.contains(&pt));
        let pt: Point = (0., 0.).into();
        assert!(!tri.contains(&pt));
        let origin: Coord = (0., 0.).into();
        let tri = Triangle::new((1., 1.).into(), origin, origin);
        let pt: Point = (1., 1.).into();
        assert!(!tri.contains(&pt));
        let pt: Point = (0.5, 0.5).into();
        assert!(!tri.contains(&pt));
    }

    #[test]
    fn rect_contains_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 100., y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(rect.contains(&poly), rect.relate(&poly).is_contains());
    }

    #[test]
    fn rect_contains_touching_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let touching_poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 90.,  y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&touching_poly),
            rect.relate(&touching_poly).is_contains()
        );

        let touching_rect = Rect::new(coord! { x: 90., y: 200. }, coord! { x: 200., y: 300. });
        assert_eq!(
            rect.contains(&touching_rect),
            rect.relate(&touching_rect).is_contains()
        );
    }

    #[test]
    fn rect_contains_empty_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let empty_poly = Polygon::new(line_string![], vec![]);
        assert_eq!(
            rect.contains(&empty_poly),
            rect.relate(&empty_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_empty_area() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let empty_poly = Polygon::new(
            line_string![
                (x: 100., y: 200.),
                (x: 100., y: 200.),
                (x: 100., y: 200.),
                (x: 100., y: 200.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&empty_poly),
            rect.relate(&empty_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_rect_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let rect_poly = rect.to_polygon();
        assert_eq!(
            rect.contains(&rect_poly),
            rect.relate(&rect_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_in_boundary() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 300., y: 360. });
        let poly_one_border =
            Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 90., y: 360. }).to_polygon();
        assert_eq!(
            rect.contains(&poly_one_border),
            rect.relate(&poly_one_border).is_contains()
        );

        let poly_two_borders = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&poly_two_borders),
            rect.relate(&poly_two_borders).is_contains()
        );

        let poly_two_borders_triangle = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&poly_two_borders_triangle),
            rect.relate(&poly_two_borders_triangle).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_in_boundary_with_hole() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 300., y: 360. });
        let poly_two_borders_triangle_with_hole = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ]],
        );
        assert_eq!(
            rect.contains(&poly_two_borders_triangle_with_hole),
            rect.relate(&poly_two_borders_triangle_with_hole)
                .is_contains()
        );
    }

    #[test]
    fn rect_empty_contains_polygon() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 90., y: 150. });
        let poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 100., y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(rect.contains(&poly), rect.relate(&poly).is_contains());

        let rect_poly = rect.to_polygon();
        assert_eq!(
            rect.contains(&rect_poly),
            rect.relate(&rect_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_point() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });

        let point1 = Point::new(100., 200.);
        assert_eq!(rect.contains(&point1), rect.relate(&point1).is_contains());

        let point2 = Point::new(90., 200.);
        assert_eq!(rect.contains(&point2), rect.relate(&point2).is_contains());
    }
}
