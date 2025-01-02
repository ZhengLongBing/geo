use crate::algorithm::{Distance, Euclidean};
use crate::CoordsIter;
use crate::GeoFloat;
use geo_types::{Coord, Point};
use num_traits::Bounded;

/// 使用 [Hausdorff 距离公式] 确定两个几何体之间的距离。
///
/// Hausdorff 距离用于比较两个点集。它测量一个集合中的点到另一个集合中最近点的最大欧几里德距离。
/// Hausdorff 距离常用于测量两个集合之间的不匹配程度。
///
/// [Hausdorff 距离公式]: https://en.wikipedia.org/wiki/Hausdorff_distance
pub trait HausdorffDistance<T>
where
    T: GeoFloat,
{
    fn hausdorff_distance<Rhs>(&self, rhs: &Rhs) -> T
    where
        Rhs: CoordsIter<Scalar = T>;
}

impl<T, G> HausdorffDistance<T> for G
where
    T: GeoFloat,
    G: CoordsIter<Scalar = T>,
{
    fn hausdorff_distance<Rhs>(&self, rhs: &Rhs) -> T
    where
        Rhs: CoordsIter<Scalar = T>,
    {
        // 从 A -> B 计算距离
        let hd1 = self
            .coords_iter()
            .map(|c| {
                rhs.coords_iter()
                    .map(|c2| Euclidean::distance(c, c2))
                    .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val))
            })
            .fold(<T as Bounded>::min_value(), |accum, val| accum.max(val));

        // 从 B -> A 计算距离
        let hd2 = rhs
            .coords_iter()
            .map(|c| {
                self.coords_iter()
                    .map(|c2| Euclidean::distance(c, c2))
                    .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val))
            })
            .fold(<T as Bounded>::min_value(), |accum, val| accum.max(val));

        // 取两者最大值
        hd1.max(hd2)
    }
}

// ┌───────────────────────────┐
// │ Coord 的实现 │
// └───────────────────────────┘

impl<T> HausdorffDistance<T> for Coord<T>
where
    T: GeoFloat,
{
    fn hausdorff_distance<Rhs>(&self, rhs: &Rhs) -> T
    where
        Rhs: CoordsIter<Scalar = T>,
    {
        Point::from(*self).hausdorff_distance(rhs)
    }
}

#[cfg(test)]
mod test {
    use crate::HausdorffDistance;
    use crate::{line_string, polygon, MultiPoint, MultiPolygon};

    #[test]
    fn hd_mpnt_mpnt() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let p2: MultiPoint<_> = vec![(2., 3.), (1., 2.)].into();
        assert_relative_eq!(p1.hausdorff_distance(&p2), 2.236068, epsilon = 1.0e-6);
    }

    #[test]
    fn hd_mpnt_poly() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let poly = polygon![
        (x: 1., y: -3.1), (x: 3.7, y: 2.7),
        (x: 0.9, y: 7.6), (x: -4.8, y: 6.7),
        (x: -7.5, y: 0.9), (x: -4.7, y: -4.),
        (x: 1., y: -3.1)
        ];

        assert_relative_eq!(p1.hausdorff_distance(&poly), 7.553807, epsilon = 1.0e-6)
    }

    #[test]
    fn hd_mpnt_lns() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let lns = line_string![
        (x: 1., y: -3.1), (x: 3.7, y: 2.7),
        (x: 0.9, y: 7.6), (x: -4.8, y: 6.7),
        (x: -7.5, y: 0.9), (x: -4.7, y: -4.),
        (x: 1., y: -3.1)
        ];

        assert_relative_eq!(p1.hausdorff_distance(&lns), 7.553807, epsilon = 1.0e-6)
    }

    #[test]
    fn hd_mpnt_mply() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let multi_polygon = MultiPolygon::new(vec![
            polygon![
              (x: 0.0f32, y: 0.0),
              (x: 2.0, y: 0.0),
              (x: 2.0, y: 1.0),
              (x: 0.0, y: 1.0),
            ],
            polygon![
              (x: 1.0, y: 1.0),
              (x: -2.0, y: 1.0),
              (x: -2.0, y: -1.0),
              (x: 1.0, y: -1.0),
            ],
        ]);

        assert_relative_eq!(
            p1.hausdorff_distance(&multi_polygon),
            2.236068,
            epsilon = 1.0e-6
        )
    }
}
