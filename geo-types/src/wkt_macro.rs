/// 从[WKT](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry)字面量创建一个[`crate::geometry`]。
///
/// 这在编译时进行评估，因此您无需担心无效WKT语法导致的运行时错误。
///
/// 注意，不接受`POINT EMPTY`，因为它无法表示为`geo_types::Point`。
///
/// ```
/// use geo_types::wkt;
/// let point = wkt! { POINT(1.0 2.0) };
/// assert_eq!(point.x(), 1.0);
/// assert_eq!(point.y(), 2.0);
///
/// let geometry_collection = wkt! {
///     GEOMETRYCOLLECTION(
///         POINT(1.0 2.0),
///         LINESTRING EMPTY,
///         POLYGON((0.0 0.0,1.0 0.0,1.0 1.0,0.0 0.0))
///     )
/// };
/// assert_eq!(geometry_collection.len(), 3);
/// ```
#[macro_export]
macro_rules! wkt {
    // 在生成的rustdoc中隐藏分散注意力的实现细节。
    ($($wkt:tt)+) => {
        {
            $crate::wkt_internal!($($wkt)+)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! wkt_internal {
    (POINT EMPTY) => {
        compile_error!("geo-types不支持EMPTY点")
    };
    (POINT($x: literal $y: literal)) => {
        $crate::point!(x: $x, y: $y)
    };
    (POINT $($tail: tt)*) => {
        compile_error!("无效的POINT wkt");
    };
    (LINESTRING EMPTY) => {
        $crate::line_string![]
    };
    (LINESTRING ($($x: literal $y: literal),+)) => {
        $crate::line_string![
            $($crate::coord!(x: $x, y: $y)),*
        ]
    };
    (LINESTRING ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (LINESTRING $($tail: tt)*) => {
        compile_error!("无效的LINESTRING wkt");
    };
    (POLYGON EMPTY) => {
        $crate::polygon![]
    };
    (POLYGON ( $exterior_tt: tt )) => {
        $crate::Polygon::new($crate::wkt!(LINESTRING $exterior_tt), $crate::_alloc::vec![])
    };
    (POLYGON( $exterior_tt: tt, $($interiors_tt: tt),+ )) => {
        $crate::Polygon::new(
            $crate::wkt!(LINESTRING $exterior_tt),
            $crate::_alloc::vec![
               $($crate::wkt!(LINESTRING $interiors_tt)),*
            ]
        )
    };
    (POLYGON ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (POLYGON $($tail: tt)*) => {
        compile_error!("无效的POLYGON wkt");
    };
    (MULTIPOINT EMPTY) => {
        $crate::MultiPoint($crate::_alloc::vec![])
    };
    (MULTIPOINT ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (MULTIPOINT ($($x: literal $y: literal),* )) => {
        $crate::MultiPoint(
            $crate::_alloc::vec![$($crate::point!(x: $x, y: $y)),*]
        )
    };
    (MULTIPOINT $($tail: tt)*) => {
        compile_error!("无效的MULTIPOINT wkt");
    };
    (MULTILINESTRING EMPTY) => {
        $crate::MultiLineString($crate::_alloc::vec![])
    };
    (MULTILINESTRING ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (MULTILINESTRING ( $($line_string_tt: tt),* )) => {
        $crate::MultiLineString($crate::_alloc::vec![
           $($crate::wkt!(LINESTRING $line_string_tt)),*
        ])
    };
    (MULTILINESTRING $($tail: tt)*) => {
        compile_error!("无效的MULTILINESTRING wkt");
    };
    (MULTIPOLYGON EMPTY) => {
        $crate::MultiPolygon($crate::_alloc::vec![])
    };
    (MULTIPOLYGON ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (MULTIPOLYGON ( $($polygon_tt: tt),* )) => {
        $crate::MultiPolygon($crate::_alloc::vec![
           $($crate::wkt!(POLYGON $polygon_tt)),*
        ])
    };
    (MULTIPOLYGON $($tail: tt)*) => {
        compile_error!("无效的MULTIPOLYGON wkt");
    };
    (GEOMETRYCOLLECTION EMPTY) => {
        $crate::GeometryCollection($crate::_alloc::vec![])
    };
    (GEOMETRYCOLLECTION ()) => {
        compile_error!("对于空集合，请使用`EMPTY`而不是()")
    };
    (GEOMETRYCOLLECTION ( $($el_type:tt $el_tt: tt),* )) => {
        $crate::GeometryCollection($crate::_alloc::vec![
           $($crate::Geometry::from($crate::wkt!($el_type $el_tt))),*
        ])
    };
    (GEOMETRYCOLLECTION $($tail: tt)*) => {
        compile_error!("无效的GEOMETRYCOLLECTION wkt");
    };
    ($name: ident ($($tail: tt)*)) => {
        compile_error!("未知类型。必须是POINT、LINESTRING、POLYGON、MULTIPOINT、MULTILINESTRING、MULTIPOLYGON或GEOMETRYCOLLECTION之一");
    };
}

#[cfg(test)]
mod test {
    use crate::geometry::*;
    use alloc::vec;

    #[test]
    fn point() {
        let point = wkt! { POINT(1.0 2.0) };
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);

        let point = wkt! { POINT(1.0   2.0) };
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);

        // 这（正确地）无法编译，因为geo-types不支持"空"点
        // wkt! { POINT EMPTY }
    }

    #[test]
    fn empty_line_string() {
        let line_string: LineString<f64> = wkt! { LINESTRING EMPTY };
        assert_eq!(line_string.0.len(), 0);

        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { LINESTRING() }
    }

    #[test]
    fn line_string() {
        let line_string = wkt! { LINESTRING(1.0 2.0,3.0 4.0) };
        assert_eq!(line_string.0.len(), 2);
        assert_eq!(line_string[0], coord! { x: 1.0, y: 2.0 });
    }

    #[test]
    fn empty_polygon() {
        let polygon: Polygon = wkt! { POLYGON EMPTY };
        assert_eq!(polygon.exterior().0.len(), 0);
        assert_eq!(polygon.interiors().len(), 0);

        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { POLYGON() }
    }

    #[test]
    fn polygon() {
        let polygon = wkt! { POLYGON((1.0 2.0)) };
        assert_eq!(polygon.exterior().0.len(), 1);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });

        let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0)) };
        // 注意：添加了一个额外的坐标以闭合线串
        assert_eq!(polygon.exterior().0.len(), 3);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
        assert_eq!(polygon.exterior().0[1], coord! { x: 3.0, y: 4.0 });
        assert_eq!(polygon.exterior().0[2], coord! { x: 1.0, y: 2.0 });

        let polygon = wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
        assert_eq!(polygon.exterior().0.len(), 1);
        assert_eq!(polygon.interiors().len(), 1);

        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
        assert_eq!(polygon.interiors()[0].0[0], coord! { x: 1.1, y: 2.1 });

        let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
        assert_eq!(polygon.exterior().0.len(), 3);
        assert_eq!(polygon.interiors().len(), 2);
        assert_eq!(polygon.interiors()[1][1], coord! { x: 3.2, y: 4.2 });
    }

    #[test]
    fn empty_multi_point() {
        let multipoint: MultiPoint = wkt! { MULTIPOINT EMPTY };
        assert!(multipoint.0.is_empty());
        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { MULTIPOINT() }
    }

    #[test]
    fn multi_point() {
        let multi_point = wkt! { MULTIPOINT(1.0 2.0) };
        assert_eq!(multi_point.0, vec![point! { x: 1.0, y: 2.0}]);

        let multi_point = wkt! { MULTIPOINT(1.0 2.0,3.0 4.0) };
        assert_eq!(
            multi_point.0,
            vec![point! { x: 1.0, y: 2.0}, point! { x: 3.0, y: 4.0}]
        );
    }

    #[test]
    fn empty_multi_line_string() {
        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING EMPTY };
        assert_eq!(multi_line_string.0, vec![]);
        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { MULTILINESTRING() }
    }
    #[test]
    fn multi_line_string() {
        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0)) };
        assert_eq!(multi_line_string.0.len(), 1);
        assert_eq!(multi_line_string.0[0].0[1], coord! { x: 3.0, y: 4.0 });
        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) };
        assert_eq!(multi_line_string.0.len(), 2);
        assert_eq!(multi_line_string.0[1].0[1], coord! { x: 7.0, y: 8.0 });

        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),EMPTY) };
        assert_eq!(multi_line_string.0.len(), 2);
        assert_eq!(multi_line_string.0[1].0.len(), 0);
    }

    #[test]
    fn empty_multi_polygon() {
        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON EMPTY };
        assert!(multi_polygon.0.is_empty());

        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { MULTIPOLYGON() }
    }

    #[test]
    fn multi_line_polygon() {
        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0))) };
        assert_eq!(multi_polygon.0.len(), 1);
        assert_eq!(multi_polygon.0[0].exterior().0[0], coord! { x: 1.0, y: 2.0});

        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)),((1.0 2.0))) };
        assert_eq!(multi_polygon.0.len(), 2);
        assert_eq!(
            multi_polygon.0[0].interiors()[1].0[0],
            coord! { x: 1.2, y: 2.2}
        );

        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)), EMPTY) };
        assert_eq!(multi_polygon.0.len(), 2);
        assert_eq!(
            multi_polygon.0[0].interiors()[1].0[0],
            coord! { x: 1.2, y: 2.2}
        );
        assert!(multi_polygon.0[1].exterior().0.is_empty());
    }

    #[test]
    fn empty_geometry_collection() {
        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION EMPTY };
        assert!(geometry_collection.is_empty());

        // 这（正确地）无法编译，因为它是无效的wkt
        // wkt! { MULTIPOLYGON() }
    }

    #[test]
    fn geometry_collection() {
        let geometry_collection = wkt! {
            GEOMETRYCOLLECTION (
                POINT (40.0 10.0),
                LINESTRING (10.0 10.0, 20.0 20.0, 10.0 40.0),
                POLYGON ((40.0 40.0, 20.0 45.0, 45.0 30.0, 40.0 40.0))
            )
        };
        assert_eq!(geometry_collection.len(), 3);

        let line_string = match &geometry_collection[1] {
            Geometry::LineString(line_string) => line_string,
            _ => panic!(
                "unexpected geometry: {geometry:?}",
                geometry = geometry_collection[1]
            ),
        };
        assert_eq!(line_string.0[1], coord! {x: 20.0, y: 20.0 });
    }

    #[test]
    fn other_numeric_types() {
        let point: Point<i32> = wkt!(POINT(1 2));
        assert_eq!(point.x(), 1i32);
        assert_eq!(point.y(), 2i32);

        let point: Point<u64> = wkt!(POINT(1 2));
        assert_eq!(point.x(), 1u64);
        assert_eq!(point.y(), 2u64);

        let point: Point<f32> = wkt!(POINT(1.0 2.0));
        assert_eq!(point.x(), 1.0f32);
        assert_eq!(point.y(), 2.0f32);
    }
}
