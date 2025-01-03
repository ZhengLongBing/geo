use std::ops::Mul;

use num_traits::FromPrimitive;

use crate::{
    coord, Coord, CoordFloat, Geometry, LineString, MultiLineString, MultiPolygon, Polygon,
};

/// 使用 Chaikin 算法平滑 `LineString`、`Polygon`、`MultiLineString` 和 `MultiPolygon`。
///
/// [Chaikin 平滑算法](http://www.idav.ucdavis.edu/education/CAGDNotes/Chaikins-Algorithm/Chaikins-Algorithm.html)
///
/// 平滑的每次迭代都会使几何体的顶点数加倍，因此某些情况下，之后应用简化以移除无关紧要的坐标可能是有意义的。
///
/// 此实现保留开放线串的起始和结束顶点，并平滑闭合线串起始和结束之间的角部。
pub trait ChaikinSmoothing<T>
where
    T: CoordFloat + FromPrimitive,
{
    /// 创建新的几何体，应用 Chaikin 平滑 `n_iterations` 次。
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self;
}

impl<T> ChaikinSmoothing<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        if n_iterations == 0 {
            self.clone()
        } else {
            let mut smooth = smoothen_linestring(self);
            for _ in 0..(n_iterations - 1) {
                smooth = smoothen_linestring(&smooth);
            }
            smooth
        }
    }
}

impl<T> ChaikinSmoothing<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        MultiLineString::new(
            self.0
                .iter()
                .map(|ls| ls.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

impl<T> ChaikinSmoothing<T> for Polygon<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        Polygon::new(
            self.exterior().chaikin_smoothing(n_iterations),
            self.interiors()
                .iter()
                .map(|ls| ls.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

impl<T> ChaikinSmoothing<T> for MultiPolygon<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        MultiPolygon::new(
            self.0
                .iter()
                .map(|poly| poly.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

macro_rules! blanket_run_chaikin_smoothing {
    ($geo:expr, $n_iter:expr) => {{
        let smooth = $geo.chaikin_smoothing($n_iter);
        let geo: Geometry<T> = smooth.into();
        geo
    }};
}

impl<T> ChaikinSmoothing<T> for Geometry<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Geometry<T> {
        match self {
            Geometry::LineString(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::MultiLineString(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::Polygon(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::MultiPolygon(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            _ => self.clone(),
        }
    }
}

fn smoothen_linestring<T>(linestring: &LineString<T>) -> LineString<T>
where
    T: CoordFloat + Mul<T> + FromPrimitive,
{
    let mut out_coords: Vec<_> = Vec::with_capacity(linestring.0.len() * 2);

    if let (Some(first), Some(last)) = (linestring.0.first(), linestring.0.last()) {
        if first != last {
            // 当线串为开放时保留起始坐标
            out_coords.push(*first);
        }
    }
    for window_coordinates in linestring.0.windows(2) {
        let (q, r) = smoothen_coordinates(window_coordinates[0], window_coordinates[1]);
        out_coords.push(q);
        out_coords.push(r);
    }

    if let (Some(first), Some(last)) = (linestring.0.first(), linestring.0.last()) {
        if first != last {
            // 保留开放线串的最后一个坐标
            out_coords.push(*last);
        } else {
            // 平滑闭合线串起始和结束之间的边缘，同时保持线串闭合。
            if let Some(out_first) = out_coords.first().copied() {
                out_coords.push(out_first);
            }
        }
    }
    out_coords.into()
}

fn smoothen_coordinates<T>(c0: Coord<T>, c1: Coord<T>) -> (Coord<T>, Coord<T>)
where
    T: CoordFloat + Mul<T> + FromPrimitive,
{
    let q = coord! {
        x: (T::from(0.75).unwrap() * c0.x) + (T::from(0.25).unwrap() * c1.x),
        y: (T::from(0.75).unwrap() * c0.y) + (T::from(0.25).unwrap() * c1.y),
    };
    let r = coord! {
        x: (T::from(0.25).unwrap() * c0.x) + (T::from(0.75).unwrap() * c1.x),
        y: (T::from(0.25).unwrap() * c0.y) + (T::from(0.75).unwrap() * c1.y),
    };
    (q, r)
}

#[cfg(test)]
mod test {
    use crate::ChaikinSmoothing;
    use crate::{Geometry, LineString, Point, Polygon};

    #[test]
    fn geometry() {
        // 测试已实现的几何类型
        let ls = LineString::from(vec![(3.0, 0.0), (6.0, 3.0), (3.0, 6.0), (0.0, 3.0)]);
        let ls_geo: Geometry = ls.into();
        let ls_geo_out = ls_geo.chaikin_smoothing(1);
        let ls_out: LineString = ls_geo_out.try_into().unwrap();
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.0, 0.0),
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.0, 3.0),
            ])
        );

        // 测试未实现的几何类型
        let pt = Point::from((3.0, 0.0));
        let pt_geo: Geometry = pt.into();
        let pt_geo_out = pt_geo.chaikin_smoothing(1);
        let pt_out: Point = pt_geo_out.try_into().unwrap();
        assert_eq!(pt_out, Point::from((3.0, 0.0)));
    }

    #[test]
    fn linestring_open() {
        let ls = LineString::from(vec![(3.0, 0.0), (6.0, 3.0), (3.0, 6.0), (0.0, 3.0)]);
        let ls_out = ls.chaikin_smoothing(1);
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.0, 0.0),
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.0, 3.0),
            ])
        );
    }

    #[test]
    fn linestring_closed() {
        let ls = LineString::from(vec![
            (3.0, 0.0),
            (6.0, 3.0),
            (3.0, 6.0),
            (0.0, 3.0),
            (3.0, 0.0),
        ]);
        let ls_out = ls.chaikin_smoothing(1);
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.75, 2.25),
                (2.25, 0.75),
                (3.75, 0.75)
            ])
        );
    }

    #[test]
    fn polygon() {
        let poly = Polygon::new(
            LineString::from(vec![
                (3.0, 0.0),
                (6.0, 3.0),
                (3.0, 6.0),
                (0.0, 3.0),
                (3.0, 0.0),
            ]),
            vec![],
        );
        let poly_out = poly.chaikin_smoothing(1);
        assert_eq!(
            poly_out.exterior(),
            &LineString::from(vec![
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.75, 2.25),
                (2.25, 0.75),
                (3.75, 0.75)
            ])
        );
    }
}
