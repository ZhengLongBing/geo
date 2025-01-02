use num_traits::FromPrimitive;

use crate::line_measures::Haversine;
// Densify 很快将被废弃，所以现在暂时允许使用已废弃的功能
#[allow(deprecated)]
use crate::HaversineLength;
use crate::{
    CoordFloat, Densify, Line, LineString, MultiLineString, MultiPolygon, Polygon, Rect, Triangle,
};

#[deprecated(
    since = "0.29.0",
    note = "请使用 `Densify` 特征中的 `line.densify::<Haversine>()` 代替。"
)]
/// 返回一个新的球面几何对象，该对象包含现有的和新的插值坐标，插值坐标之间的最大距离为 `max_distance`。
///
/// 注意：`max_distance` 必须大于 0。
///
/// ## 单位
///
/// - `max_distance`：米
///
/// # 示例
/// ```
/// use geo::{coord, Line, LineString};
/// #[allow(deprecated)]
/// use geo::DensifyHaversine;
///
/// let line = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 0.0, y: 1.0 });
/// // 已知输出
/// let output: LineString = vec![[0.0, 0.0], [0.0, 0.5], [0.0, 1.0]].into();
/// // 进行插值
/// let dense = line.densify_haversine(100000.0);
/// assert_eq!(dense, output);
///```
pub trait DensifyHaversine<F: CoordFloat> {
    type Output;

    fn densify_haversine(&self, max_distance: F) -> Self::Output;
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for MultiPolygon<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = MultiPolygon<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for Polygon<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = Polygon<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = MultiLineString<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = LineString<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = LineString<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for Triangle<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = Polygon<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[allow(deprecated)]
impl<T> DensifyHaversine<T> for Rect<T>
where
    T: CoordFloat + FromPrimitive,
    Line<T>: HaversineLength<T>,
    LineString<T>: HaversineLength<T>,
{
    type Output = Polygon<T>;

    fn densify_haversine(&self, max_distance: T) -> Self::Output {
        self.densify::<Haversine>(max_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, CoordsIter};

    #[test]
    fn test_polygon_densify() {
        let exterior: LineString = vec![
            [4.925, 45.804],
            [4.732, 45.941],
            [4.935, 46.513],
            [5.821, 46.103],
            [5.627, 45.611],
            [5.355, 45.883],
            [4.925, 45.804],
        ]
        .into();

        let polygon = Polygon::new(exterior, vec![]);

        let output_exterior: LineString = vec![
            [4.925, 45.804],
            [4.732, 45.941],
            [4.8329711649985505, 46.2270449096239],
            [4.935, 46.513],
            [5.379659133344039, 46.30885540136222],
            [5.821, 46.103],
            [5.723570877658867, 45.85704103535437],
            [5.627, 45.611],
            [5.355, 45.883],
            [4.925, 45.804],
        ]
        .into();

        #[allow(deprecated)]
        let dense = polygon.densify_haversine(50000.0);
        assert_relative_eq!(dense.exterior(), &output_exterior);
    }

    #[test]
    fn test_linestring_densify() {
        let linestring: LineString = vec![
            [-3.202, 55.9471],
            [-3.2012, 55.9476],
            [-3.1994, 55.9476],
            [-3.1977, 55.9481],
            [-3.196, 55.9483],
            [-3.1947, 55.9487],
            [-3.1944, 55.9488],
            [-3.1944, 55.949],
        ]
        .into();

        let output: LineString = vec![
            [-3.202, 55.9471],
            [-3.2012, 55.9476],
            [-3.2002999999999995, 55.94760000327935],
            [-3.1994, 55.9476],
            [-3.1985500054877773, 55.94785000292509],
            [-3.1977, 55.9481],
            [-3.196, 55.9483],
            [-3.1947, 55.9487],
            [-3.1944, 55.9488],
            [-3.1944, 55.949],
        ]
        .into();

        #[allow(deprecated)]
        let dense = linestring.densify_haversine(110.0);
        assert_relative_eq!(dense, output);
    }

    #[test]
    fn test_line_densify() {
        let output: LineString = vec![[0.0, 0.0], [0.0, 0.5], [0.0, 1.0]].into();
        let line = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 0.0, y: 1.0 });
        #[allow(deprecated)]
        let dense = line.densify_haversine(100000.0);
        assert_relative_eq!(dense, output);
    }

    #[test]
    fn test_empty_linestring() {
        let linestring: LineString<f64> = LineString::new(vec![]);
        #[allow(deprecated)]
        let dense = linestring.densify_haversine(10.0);
        assert_eq!(0, dense.coords_count());
    }
}
