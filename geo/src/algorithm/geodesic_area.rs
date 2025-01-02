use crate::geometry::*;
use geographiclib_rs::{Geodesic, PolygonArea, Winding};

/// 计算地球椭球模型上的几何体的周长和面积。
///
/// 使用[Karney (2013)]的方法进行大地测量。
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicArea<T> {
    /// 计算地球椭球模型上的几何体的面积。
    ///
    /// 使用[Karney (2013)]的方法进行大地测量。
    ///
    /// # 假设
    ///  - 多边形的外环假定为逆时针方向缠绕，内环为顺时针方向缠绕。
    ///    这是符合简单特征标准的几何体的标准缠绕方式。
    ///    使用其他缠绕方式可能导致面积为负。请参见下文“解释负面积值”。
    ///  - 假定多边形小于地球的一半。如果处理更大的多边形，请使用`unsigned`方法。
    ///
    /// # 单位
    ///
    /// - 返回值：平方米
    ///
    /// # 解释负面积值
    ///
    /// 负值可能意味着以下两种情况之一：
    /// 1. 多边形是顺时针缠绕（反向缠绕）。如果是这种情况，并且您知道多边形小于地球的一半，可以取面积的绝对值。
    /// 2. 多边形大于半个地球。在这种情况下，返回的多边形面积是不正确的。如果处理非常大的多边形，请使用`unsigned`方法。
    ///
    /// # 示例
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::polygon;
    /// use geo::Polygon;
    ///
    /// // 伦敦的O2
    /// let mut polygon: Polygon<f64> = polygon![
    ///     (x: 0.00388383, y: 51.501574),
    ///     (x: 0.00538587, y: 51.502278),
    ///     (x: 0.00553607, y: 51.503299),
    ///     (x: 0.00467777, y: 51.504181),
    ///     (x: 0.00327229, y: 51.504435),
    ///     (x: 0.00187754, y: 51.504168),
    ///     (x: 0.00087976, y: 51.503380),
    ///     (x: 0.00107288, y: 51.502324),
    ///     (x: 0.00185608, y: 51.501770),
    ///     (x: 0.00388383, y: 51.501574),
    /// ];
    ///
    /// let area = polygon.geodesic_area_unsigned();
    ///
    /// assert_eq!(
    ///     78_596., // 米
    ///     area.round()
    /// );
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_area_signed(&self) -> T;

    /// 计算地球椭球模型上的几何体的面积。支持覆盖地球大部分的大型几何体。
    ///
    /// 使用[Karney (2013)]的方法进行大地测量。
    ///
    /// # 假设
    ///  - 多边形的外环假定为逆时针方向缠绕，内环为顺时针方向缠绕。
    ///    这是符合简单特征标准的几何体的标准缠绕方式。
    ///    使用其他缠绕方式会导致结果不正确。
    ///
    /// # 单位
    ///
    /// - 返回值：平方米
    ///
    /// # 示例
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::polygon;
    /// use geo::Polygon;
    ///
    /// // 描述一个覆盖除了这个小正方形外的整个地球的多边形。
    /// // 多边形的外环在这个正方形，内环是地球的其余部分。
    /// let mut polygon: Polygon<f64> = polygon![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 0.0, y: 1.0),
    ///     (x: 1.0, y: 1.0),
    ///     (x: 1.0, y: 0.0),
    /// ];
    ///
    /// let area = polygon.geodesic_area_unsigned();
    ///
    /// // 超过5万亿平方米！
    /// assert_eq!(area, 510053312945726.94);
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_area_unsigned(&self) -> T;

    /// 计算地球椭球模型上的几何体的周长。
    ///
    /// 使用[Karney (2013)]的方法进行大地测量。
    ///
    /// 对于多边形，此方法返回外环和内环的周长之和。
    /// 要仅获得多边形的外环周长，请使用`polygon.exterior().geodesic_length()`。
    ///
    /// # 单位
    ///
    /// - 返回值：米
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter(&self) -> T;

    /// 在一个操作中计算地球椭球模型上的几何体的周长和面积。
    ///
    /// 以`(周长, 面积)`元组的形式返回周长和面积，并使用[Karney (2013)]的方法进行大地测量。
    ///
    /// # 面积假设
    ///  - 多边形的外环假定为逆时针方向缠绕，内环为顺时针方向缠绕。
    ///    这是符合简单特征标准的几何体的标准缠绕方式。
    ///    使用其他缠绕方式可能导致面积为负。请参见下文“解释负面积值”。
    ///  - 假定多边形小于地球的一半。如果处理更大的多边形，请使用`unsigned`方法。
    ///
    /// # 周长
    /// 对于多边形，此方法返回外环和内环的周长之和。
    /// 要仅获得多边形的外环周长，请使用`polygon.exterior().geodesic_length()`。
    ///
    /// # 单位
    ///
    /// - 返回值：(米, 平方米)
    ///
    /// # 解释负面积值
    ///
    /// 负面积值可能意味着以下两种情况之一：
    /// 1. 多边形是顺时针缠绕（反向缠绕）。如果是这种情况，并且您知道多边形小于地球的一半，可以取面积的绝对值。
    /// 2. 多边形大于半个地球。在这种情况下，返回的多边形面积是不正确的。如果处理非常大的多边形，请使用`unsigned`方法。
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter_area_signed(&self) -> (T, T);

    /// 在一个操作中计算地球椭球模型上的几何体的周长和面积。支持覆盖地球大部分的大型几何体。
    ///
    /// 以`(周长, 面积)`元组的形式返回周长和面积，并使用[Karney (2013)]的方法进行大地测量。
    ///
    /// # 面积假设
    ///  - 多边形的外环假定为逆时针方向缠绕，内环为顺时针方向缠绕。
    ///    这是符合简单特征标准的几何体的标准缠绕方式。
    ///    使用其他缠绕方式会导致结果不正确。
    ///
    /// # 周长
    /// 对于多边形，此方法返回外环和内环的周长。
    /// 要仅获得多边形的外环周长，请使用`polygon.exterior().geodesic_length()`。
    ///
    /// # 单位
    ///
    /// - 返回值：(米, 平方米)
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter_area_unsigned(&self) -> (T, T);
}

impl GeodesicArea<f64> for Polygon {
    fn geodesic_perimeter(&self) -> f64 {
        let (perimeter, _area) = geodesic_area(self, true, false, false);
        perimeter
    }

    fn geodesic_area_signed(&self) -> f64 {
        let (_perimeter, area) = geodesic_area(self, true, false, false);
        area
    }

    fn geodesic_area_unsigned(&self) -> f64 {
        let (_perimeter, area) = geodesic_area(self, false, false, false);
        area
    }

    fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
        geodesic_area(self, true, false, false)
    }

    fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
        geodesic_area(self, false, false, false)
    }
}

fn geodesic_area(poly: &Polygon, sign: bool, reverse: bool, exterior_only: bool) -> (f64, f64) {
    let g = Geodesic::wgs84();

    let (exterior_winding, interior_winding) = if reverse {
        (Winding::Clockwise, Winding::CounterClockwise)
    } else {
        (Winding::CounterClockwise, Winding::Clockwise)
    };

    // 添加外环
    let (outer_perimeter, outer_area) = {
        let mut pa = PolygonArea::new(&g, exterior_winding);
        poly.exterior().points().for_each(|p| {
            pa.add_point(p.y(), p.x());
        });
        let (perimeter, area, _) = pa.compute(sign);
        (perimeter, area)
    };

    // 添加内环
    let (interior_perimeter, mut inner_area) = if exterior_only {
        (0.0, 0.0)
    } else {
        let mut inner_area = 0.;
        let mut inner_perimeter = 0.;
        poly.interiors().iter().for_each(|ring| {
            let mut pa = PolygonArea::new(&g, interior_winding);
            ring.points().for_each(|p| {
                pa.add_point(p.y(), p.x());
            });
            let (perimeter, area, _) = pa.compute(sign);
            inner_area += area.abs();
            inner_perimeter += perimeter;
        });
        (inner_perimeter, inner_area)
    };

    if outer_area < 0.0 && inner_area > 0.0 {
        inner_area = -inner_area;
    }

    (
        outer_perimeter + interior_perimeter,
        outer_area - inner_area,
    )
}

/// 生成结果为零的`GeodesicArea`实现。
macro_rules! zero_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                0.0
            }

            fn geodesic_area_signed(&self) -> f64 {
                0.0
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                0.0
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                (0.0, 0.0)
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                (0.0, 0.0)
            }
        }
    };
}

/// 生成一个`GeodesicArea`实现，该实现委托给`Polygon`实现。
macro_rules! to_polygon_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                self.to_polygon().geodesic_perimeter()
            }

            fn geodesic_area_signed(&self) -> f64 {
                self.to_polygon().geodesic_area_signed()
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                self.to_polygon().geodesic_area_unsigned()
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                self.to_polygon().geodesic_perimeter_area_signed()
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                self.to_polygon().geodesic_perimeter_area_unsigned()
            }
        }
    };
}

/// 生成一个`GeodesicArea`实现，该实现计算其每个子组件的面积并求和。
macro_rules! sum_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_perimeter())
            }

            fn geodesic_area_signed(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_area_signed())
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_area_unsigned())
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                self.iter()
                    .fold((0.0, 0.0), |(total_perimeter, total_area), next| {
                        let (perimeter, area) = next.geodesic_perimeter_area_signed();
                        (total_perimeter + perimeter, total_area + area)
                    })
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                self.iter()
                    .fold((0.0, 0.0), |(total_perimeter, total_area), next| {
                        let (perimeter, area) = next.geodesic_perimeter_area_unsigned();
                        (total_perimeter + perimeter, total_area + area)
                    })
            }
        }
    };
}

zero_impl!(Point);
zero_impl!(Line);
zero_impl!(LineString);
zero_impl!(MultiPoint);
zero_impl!(MultiLineString);
to_polygon_impl!(Rect);
to_polygon_impl!(Triangle);
sum_impl!(GeometryCollection);
sum_impl!(MultiPolygon);

impl GeodesicArea<f64> for Geometry<f64> {
    crate::geometry_delegate_impl! {
        fn geodesic_perimeter(&self) -> f64;
        fn geodesic_area_signed(&self) -> f64;
        fn geodesic_area_unsigned(&self) -> f64;
        fn geodesic_perimeter_area_signed(&self) -> (f64, f64);
        fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::line_measures::{Geodesic, Length};
    use crate::polygon;

    #[test]
    fn test_negative() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 144., y: -15.),
            (x: 154., y: -27.),
            (x: 148., y: -39.),
            (x: 130., y: -33.),
            (x: 117., y: -37.),
            (x: 113., y: -22.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            -7786102826806.07,
            polygon.geodesic_area_signed(),
            epsilon = 0.01
        );

        let geoid = geographiclib_rs::Geodesic::wgs84();
        assert_relative_eq!(
            geoid.area() - 7786102826806.07,
            polygon.geodesic_area_unsigned(),
            epsilon = 0.01
        );

        // 确认外环的大地测量长度与周长一致
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_positive() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 113., y: -22.),
            (x: 117., y: -37.),
            (x: 130., y: -33.),
            (x: 148., y: -39.),
            (x: 154., y: -27.),
            (x: 144., y: -15.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_area_signed(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_area_unsigned(),
            epsilon = 0.01
        );

        // 确认外环的大地测量长度与周长一致
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_missing_endpoint() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 113., y: -22.),
            (x: 117., y: -37.),
            (x: 130., y: -33.),
            (x: 148., y: -39.),
            (x: 154., y: -27.),
            (x: 144., y: -15.),
            // (x: 125., y: -15.), <-- 缺少终点
        ];
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_area_signed(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_area_unsigned(),
            epsilon = 0.01
        );

        // 确认外环的大地测量长度与周长一致
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_holes() {
        let mut poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 1., y: 2.),
                    (x: 2., y: 2.),
                    (x: 2., y: 1.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 5., y: 6.),
                    (x: 6., y: 6.),
                    (x: 6., y: 5.),
                    (x: 5., y: 5.)
                ],
            ],
        ];

        assert_relative_eq!(
            1203317999173.7063,
            poly.geodesic_area_signed(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            1203317999173.7063,
            poly.geodesic_area_unsigned(),
            epsilon = 0.01
        );
        assert_relative_eq!(5307742.446635911, poly.geodesic_perimeter(), epsilon = 0.01);

        let (perimeter, area) = poly.geodesic_perimeter_area_signed();

        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);

        let (perimeter, area) = poly.geodesic_perimeter_area_unsigned();

        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);

        // 测试外环和内环均为CW缠绕
        use crate::algorithm::winding_order::Winding;
        poly.exterior_mut(|exterior| {
            exterior.make_cw_winding();
        });

        let (perimeter, area) = poly.geodesic_perimeter_area_signed();
        assert_relative_eq!(-1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);

        // 测试外环CW和内环CCW缠绕
        poly.interiors_mut(|interiors| {
            for interior in interiors {
                interior.make_ccw_winding();
            }
        });

        let (perimeter, area) = poly.geodesic_perimeter_area_signed();
        assert_relative_eq!(-1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);

        // 测试外环和内环均为CCW缠绕
        poly.exterior_mut(|exterior| {
            exterior.make_ccw_winding();
        });

        let (perimeter, area) = poly.geodesic_perimeter_area_signed();
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
    }

    #[test]
    fn test_bad_interior_winding() {
        let poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 2., y: 1.),
                    (x: 2., y: 2.),
                    (x: 1., y: 2.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 6., y: 5.),
                    (x: 6., y: 6.),
                    (x: 5., y: 6.),
                    (x: 5., y: 5.)
                ],
            ],
        ];

        assert_relative_eq!(1203317999173.7063, poly.geodesic_area_signed());
    }

    #[test]
    fn test_diamond() {
        // 一个菱形形状
        let mut diamond = polygon![
            // 外环逆时针方向
            exterior: [
                (x: 1.0, y: 0.0),
                (x: 2.0, y: 1.0),
                (x: 1.0, y: 2.0),
                (x: 0.0, y: 1.0),
                (x: 1.0, y: 0.0),
            ],
            // 内环顺时针方向
            interiors: [
                [
                    (x: 1.0, y: 0.5),
                    (x: 0.5, y: 1.0),
                    (x: 1.0, y: 1.5),
                    (x: 1.5, y: 1.0),
                    (x: 1.0, y: 0.5),
                ],
            ],
        ];
        assert_relative_eq!(18462065880.09138, diamond.geodesic_area_unsigned());
        assert_relative_eq!(18462065880.09138, diamond.geodesic_area_signed());
        assert_relative_eq!(941333.0085011568, diamond.geodesic_perimeter());

        let (perimeter, area) = diamond.geodesic_perimeter_area_signed();
        assert_relative_eq!(941333.0085011568, perimeter);
        assert_relative_eq!(18462065880.09138, area);

        let (perimeter, area) = diamond.geodesic_perimeter_area_unsigned();
        assert_relative_eq!(941333.0085011568, perimeter);
        assert_relative_eq!(18462065880.09138, area);

        // 测试外环和内环均为CW缠绕
        use crate::algorithm::winding_order::Winding;
        diamond.exterior_mut(|exterior| {
            exterior.make_cw_winding();
        });

        let (perimeter, area) = diamond.geodesic_perimeter_area_signed();
        assert_relative_eq!(-18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);

        // 测试外环CW和内环CCW缠绕
        diamond.interiors_mut(|interiors| {
            for interior in interiors {
                interior.make_ccw_winding();
            }
        });

        let (perimeter, area) = diamond.geodesic_perimeter_area_signed();
        assert_relative_eq!(-18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);

        // 测试外环和内环均为CCW缠绕
        diamond.exterior_mut(|exterior| {
            exterior.make_ccw_winding();
        });

        let (perimeter, area) = diamond.geodesic_perimeter_area_signed();
        assert_relative_eq!(18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);
    }

    #[test]
    fn test_very_large_polygon() {
        // 描述一个覆盖除这个小正方形外的整个地球的多边形。
        // 多边形的外环在这个正方形，内环是地球的其余部分。
        let polygon_large: Polygon<f64> = polygon![
            (x: 0.0, y: 0.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 1.0),
            (x: 1.0, y: 0.0),
        ];

        let area = polygon_large.geodesic_area_unsigned();
        assert_eq!(area, 510053312945726.94);

        // 一个几乎覆盖整个地球的大型多边形，然后是一个同样覆盖几乎整个地球的洞。
        // 这是一个有趣的多边形，因为无论缠绕顺序如何，带符号和无符号面积都是相同的。
        let polygon_large_with_hole: Polygon<f64> = polygon![
            exterior: [
                (x: 0.5, y: 0.5),
                (x: 0.5, y: 1.0),
                (x: 1.0, y: 1.0),
                (x: 1.0, y: 0.5),
                (x: 0.5, y: 0.5),
            ],
            interiors: [
                [
                    (x: 0.0, y: 0.0),
                    (x: 2.0, y: 0.0),
                    (x: 2.0, y: 2.0),
                    (x: 0.0, y: 2.0),
                    (x: 0.0, y: 0.0),
                ],
            ],
        ];

        let area = polygon_large_with_hole.geodesic_area_signed();
        assert_relative_eq!(area, 46154562709.8, epsilon = 0.1);

        let area = polygon_large_with_hole.geodesic_area_unsigned();
        assert_relative_eq!(area, 46154562709.8, epsilon = 0.1);
    }
}
