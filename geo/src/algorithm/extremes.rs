use crate::CoordsIter;
use crate::{Coord, CoordNum};

/// 找到几何体的极端坐标和索引。
///
/// # 示例
///
/// ```
/// use geo::extremes::Extremes;
/// use geo::polygon;
///
/// // 一个菱形形状
/// let polygon = polygon![
///     (x: 1.0, y: 0.0),
///     (x: 2.0, y: 1.0),
///     (x: 1.0, y: 2.0),
///     (x: 0.0, y: 1.0),
///     (x: 1.0, y: 0.0),
/// ];
///
/// let extremes = polygon.extremes().unwrap();
///
/// assert_eq!(extremes.y_max.index, 2);
/// assert_eq!(extremes.y_max.coord.x, 1.);
/// assert_eq!(extremes.y_max.coord.y, 2.);
/// ```
pub trait Extremes<'a, T: CoordNum> {
    /// 计算几何体的极值坐标。
    fn extremes(&'a self) -> Option<Outcome<T>>;
}

/// 表示一个极值坐标的结构体
#[derive(Debug, PartialEq, Eq)]
pub struct Extreme<T: CoordNum> {
    pub index: usize,    // 坐标的索引
    pub coord: Coord<T>, // 坐标值
}

/// 包含四个极值的结构体
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<T: CoordNum> {
    pub x_min: Extreme<T>, // x轴最小值
    pub y_min: Extreme<T>, // y轴最小值
    pub x_max: Extreme<T>, // x轴最大值
    pub y_max: Extreme<T>, // y轴最大值
}

impl<'a, T, G> Extremes<'a, T> for G
where
    G: CoordsIter<Scalar = T>,
    T: CoordNum,
{
    fn extremes(&'a self) -> Option<Outcome<T>> {
        let mut iter = self.exterior_coords_iter().enumerate();

        let mut outcome = iter.next().map(|(index, coord)| Outcome {
            x_min: Extreme { index, coord },
            y_min: Extreme { index, coord },
            x_max: Extreme { index, coord },
            y_max: Extreme { index, coord },
        })?;

        for (index, coord) in iter {
            if coord.x < outcome.x_min.coord.x {
                outcome.x_min = Extreme { coord, index };
            }

            if coord.y < outcome.y_min.coord.y {
                outcome.y_min = Extreme { coord, index };
            }

            if coord.x > outcome.x_max.coord.x {
                outcome.x_max = Extreme { coord, index };
            }

            if coord.y > outcome.y_max.coord.y {
                outcome.y_max = Extreme { coord, index };
            }
        }

        Some(outcome)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, polygon, MultiPoint};

    #[test]
    fn polygon() {
        // 一个菱形形状
        let polygon = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0),
        ];

        let actual = polygon.extremes();

        assert_eq!(
            Some(Outcome {
                x_min: Extreme {
                    index: 3,
                    coord: coord! { x: 0.0, y: 1.0 }
                },
                y_min: Extreme {
                    index: 0,
                    coord: coord! { x: 1.0, y: 0.0 }
                },
                x_max: Extreme {
                    index: 1,
                    coord: coord! { x: 2.0, y: 1.0 }
                },
                y_max: Extreme {
                    index: 2,
                    coord: coord! { x: 1.0, y: 2.0 }
                }
            }),
            actual
        );
    }

    #[test]
    fn empty() {
        // 测试空几何
        let multi_point: MultiPoint<f32> = MultiPoint::new(vec![]);

        let actual = multi_point.extremes();

        assert!(actual.is_none());
    }
}
