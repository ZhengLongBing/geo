use super::*;
use crate::geometry::*;
use crate::{coord, line_string, polygon};

#[test]
fn test_zero_points() {
    // 测试空点集
    let mut v: Vec<Coord<i64>> = vec![];
    let correct = vec![];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_zero_points_include_on_hull() {
    // 测试空点集且包含在凸包中
    let mut v: Vec<Coord<i64>> = vec![];
    let correct = vec![];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_one_point() {
    // 测试单个点
    let mut v = vec![coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_one_point_include_on_hull() {
    // 测试单个点且包含在凸包中
    let mut v = vec![coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points() {
    // 测试两个点
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_include_on_hull() {
    // 测试两个点且包含在凸包中
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_duplicated() {
    // 测试两个重复点
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_duplicated_include_on_hull() {
    // 测试两个重复点且包含在凸包中
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_ccw() {
    // 测试三个点逆时针
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_cw() {
    // 测试三个点顺时针
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 1, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_two_duplicated() {
    // 测试三个点有两个重复
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_two_duplicated_include_on_hull() {
    // 测试三个点有两个重复且包含在凸包中
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_duplicated() {
    // 测试三个点全部重复
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_duplicated_include_on_hull() {
    // 测试三个点全部重复且包含在凸包中
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_collinear_points() {
    // 测试三个共线点
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 2, y: 2 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_collinear_points_include_on_hull() {
    // 测试三个共线点且包含在凸包中
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn convex_hull_multipoint_test() {
    // 测试多点的凸包
    let v = vec![
        Point::new(0, 10),
        Point::new(1, 1),
        Point::new(10, 0),
        Point::new(1, -1),
        Point::new(0, -10),
        Point::new(-1, -1),
        Point::new(-10, 0),
        Point::new(-1, 1),
        Point::new(0, 10),
    ];
    let mp = MultiPoint::new(v);
    let correct = vec![
        Coord::from((0, -10)),
        Coord::from((10, 0)),
        Coord::from((0, 10)),
        Coord::from((-10, 0)),
        Coord::from((0, -10)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}

#[test]
fn convex_hull_linestring_test() {
    // 测试线串的凸包
    let mp = line_string![
        (x: 0.0, y: 10.0),
        (x: 1.0, y: 1.0),
        (x: 10.0, y: 0.0),
        (x: 1.0, y: -1.0),
        (x: 0.0, y: -10.0),
        (x: -1.0, y: -1.0),
        (x: -10.0, y: 0.0),
        (x: -1.0, y: 1.0),
        (x: 0.0, y: 10.0),
    ];
    let correct = vec![
        Coord::from((0.0, -10.0)),
        Coord::from((10.0, 0.0)),
        Coord::from((0.0, 10.0)),
        Coord::from((-10.0, 0.0)),
        Coord::from((0.0, -10.0)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}

#[test]
fn convex_hull_multilinestring_test() {
    // 测试多线串的凸包
    let v1 = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0)];
    let v2 = line_string![(x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 3.0, y: 1.0)];
    let mls = MultiLineString::new(vec![v1, v2]);
    let correct = vec![
        Coord::from((2.0, 0.0)),
        Coord::from((3.0, 1.0)),
        Coord::from((1.0, 10.0)),
        Coord::from((0.0, 0.0)),
        Coord::from((2.0, 0.0)),
    ];
    let res = mls.convex_hull();
    assert_eq!(res.exterior().0, correct);
}

#[test]
fn convex_hull_multipolygon_test() {
    // 测试多多边形的凸包
    let p1 = polygon![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 0.0, y: 0.0)];
    let p2 = polygon![(x: 3.0, y: 0.0), (x: 4.0, y: 10.0), (x: 5.0, y: 0.0), (x: 3.0, y: 0.0)];
    let mp = MultiPolygon::new(vec![p1, p2]);
    let correct = vec![
        Coord::from((5.0, 0.0)),
        Coord::from((4.0, 10.0)),
        Coord::from((1.0, 10.0)),
        Coord::from((0.0, 0.0)),
        Coord::from((5.0, 0.0)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}

#[test]
fn collection() {
    // 几何图形集合测试
    let collection = GeometryCollection(vec![
        Point::new(0.0, 0.0).into(),
        Triangle::new(
            coord! { x: 1.0, y: 0.0},
            coord! { x: 4.0, y: 0.0},
            coord! { x: 4.0, y: 4.0 },
        )
        .into(),
    ]);

    let convex_hull = collection.convex_hull();
    assert_eq!(
        convex_hull,
        polygon![
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 0.0, y: 0.0 }
        ]
    );
}
