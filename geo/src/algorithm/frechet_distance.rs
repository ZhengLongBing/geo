use crate::coords_iter::CoordsIter;
use crate::line_measures::{Distance, Euclidean};
use crate::{GeoFloat, LineString};
use num_traits::FromPrimitive;

/// 使用[Frechet距离]确定两个`LineString`之间的相似度。
///
/// 基于T. Eiter和H. Mannila的[计算离散Frechet距离]。
///
/// [Frechet距离]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
/// [计算离散Frechet距离]: http://www.kr.tuwien.ac.at/staff/eiter/et-archive/cdtr9464.pdf
pub trait FrechetDistance<T, Rhs = Self> {
    /// 使用[Frechet距离]确定两个`LineString`之间的相似度。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::FrechetDistance;
    /// use geo::line_string;
    ///
    /// let line_string_a = line_string![
    ///     (x: 1., y: 1.),
    ///     (x: 2., y: 1.),
    ///     (x: 2., y: 2.),
    ///     (x: 3., y: 3.)
    /// ];
    ///
    /// let line_string_b = line_string![
    ///     (x: 2., y: 2.),
    ///     (x: 0., y: 1.),
    ///     (x: 2., y: 4.),
    ///     (x: 3., y: 4.)
    /// ];
    ///
    /// let distance = line_string_a.frechet_distance(&line_string_b);
    ///
    /// assert_eq!(2., distance);
    /// ```
    ///
    /// [Frechet距离]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
    fn frechet_distance(&self, rhs: &Rhs) -> T;
}

impl<T> FrechetDistance<T, LineString<T>> for LineString<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn frechet_distance(&self, ls: &LineString<T>) -> T {
        if self.coords_count() != 0 && ls.coords_count() != 0 {
            Data {
                cache: vec![T::zero(); self.coords_count() * ls.coords_count()],
                ls_a: self,
                ls_b: ls,
            }
            .compute_linear()
        } else {
            T::zero()
        }
    }
}

struct Data<'a, T>
where
    T: GeoFloat + FromPrimitive,
{
    cache: Vec<T>,
    ls_a: &'a LineString<T>,
    ls_b: &'a LineString<T>,
}

impl<T> Data<'_, T>
where
    T: GeoFloat + FromPrimitive,
{
    /// [参考实现]: https://github.com/joaofig/discrete-frechet/tree/master
    fn compute_linear(&mut self) -> T {
        let columns_count = self.ls_b.coords_count();

        for (i, &a) in self.ls_a.coords().enumerate() {
            for (j, &b) in self.ls_b.coords().enumerate() {
                let dist = Euclidean::distance(a, b);

                self.cache[i * columns_count + j] = match (i, j) {
                    (0, 0) => dist,
                    (_, 0) => self.cache[(i - 1) * columns_count].max(dist),
                    (0, _) => self.cache[j - 1].max(dist),
                    (_, _) => self.cache[(i - 1) * columns_count + j]
                        .min(self.cache[(i - 1) * columns_count + j - 1])
                        .min(self.cache[i * columns_count + j - 1])
                        .max(dist),
                };
            }
        }

        self.cache[self.cache.len() - 1]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_point_in_linestring() {
        // 测试单个点的情况
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(0., 2.)]);
        assert_relative_eq!(
            Euclidean::distance(ls_a.0[0], ls_b.0[0]),
            ls_a.frechet_distance(&ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings() {
        // 测试完全相同的LineString
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_relative_eq!(0., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn different_dimensions_linestrings() {
        // 测试不同维度的LineString
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        assert_relative_eq!(2f64.sqrt(), ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_1() {
        // 测试具体的Frechet距离计算示例1
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (2., 3.)]);
        assert_relative_eq!(2., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_2() {
        // 测试具体的Frechet距离计算示例2
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.)]);
        assert_relative_eq!(2., ls_a.frechet_distance(&ls_b));
    }

    #[test] // 比较长的LineString时,不应因为堆栈溢出而发生恐慌或中止
    fn test_frechet_long_linestrings() {
        // 测试非常长的LineString
        let ls: LineString = {
            let delta = 0.01;

            let mut ls = vec![(0.0, 0.0); 10_000];
            for i in 1..ls.len() {
                let (lat, lon) = ls[i - 1];
                ls[i] = (lat - delta, lon + delta);
            }

            ls.into()
        };

        assert_relative_eq!(ls.frechet_distance(&ls.clone()), 0.0);
    }
}
