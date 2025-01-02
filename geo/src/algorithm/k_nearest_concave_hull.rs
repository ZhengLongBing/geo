use crate::{
    Contains, ConvexHull, Coord, CoordNum, GeoFloat, Intersects, LineString, MultiPoint, Point,
    Polygon,
};
use num_traits::Float;
use rstar::RTreeNum;
use std::cmp::max;

const K_MULTIPLIER: f32 = 1.5;

/// 另一种针对[凹包络线](trait.algorithm.ConcaveHull.html)的方法。该算法基于
/// Adriano Moreira 和 Maribel Santos 的[k最近邻方法](https://pdfs.semanticscholar.org/2397/17005c3ebd5d6a42fc833daf97a0edee1ce4.pdf)。
///
/// 该算法的基本思想简单：
/// 1. 找到一个未来包络线上的点（例如，具有最小Y坐标的点）。
/// 2. 找到该点的K个最近邻点。
/// 3. 选择最近的点之一作为包络线的下一个点，该点会使与前一个线段形成最大左转弯。
/// 4. 重复步骤2-4。
///
/// 当给定的K无法计算包络线时，会选择更大的值并从头开始计算。
///
/// 在最坏的情况下，当找不到合适的K来构建正确的包络线时，将返回凸包络线。
///
/// 通常该算法比用于[凹包络线](trait.algorithm.ConcaveHull.html)特性的算法慢几倍，但效果更好，并且
/// 不需要手动调整系数。
///
/// 赋予算法的K值越大，包络线通常会越“平滑”，但计算时间可能会更长。如果性能不是至关重要的，K=3是一个安全的值
/// （对于该算法，较小的值没有意义）。如果K等于或大于输入点的数量，将生成凸包络线。
pub trait KNearestConcaveHull {
    type Scalar: CoordNum;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar>;
}

impl<T> KNearestConcaveHull for Vec<Point<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

impl<T> KNearestConcaveHull for [Point<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

impl<T> KNearestConcaveHull for Vec<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter(), k)
    }
}

impl<T> KNearestConcaveHull for [Coord<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter(), k)
    }
}

impl<T> KNearestConcaveHull for MultiPoint<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

fn concave_hull<'a, T>(coords: impl Iterator<Item = &'a Coord<T>>, k: u32) -> Polygon<T>
where
    T: 'a + GeoFloat + RTreeNum,
{
    let dataset = prepare_dataset(coords);
    concave_hull_inner(dataset, k)
}

const DELTA: f32 = 0.000000001;

/// 从数据集中删除重复的坐标。
fn prepare_dataset<'a, T>(coords: impl Iterator<Item = &'a Coord<T>>) -> rstar::RTree<Coord<T>>
where
    T: 'a + GeoFloat + RTreeNum,
{
    let mut dataset: rstar::RTree<Coord<T>> = rstar::RTree::new();
    for coord in coords {
        let closest = dataset.nearest_neighbor(coord);
        if let Some(closest) = closest {
            if coords_are_equal(coord, closest) {
                continue;
            }
        }

        dataset.insert(*coord)
    }

    dataset
}

/// 如果两个坐标值在0.0000001%范围内相同，则两个点被认为是相等的（见DELTA常量的值）。
fn coords_are_equal<T>(c1: &Coord<T>, c2: &Coord<T>) -> bool
where
    T: GeoFloat + RTreeNum,
{
    float_equal(c1.x, c2.x) && float_equal(c1.y, c2.y)
}

fn float_equal<T>(a: T, b: T) -> bool
where
    T: GeoFloat,
{
    let da = a * T::from(DELTA).expect("从常量转换始终有效。").abs();
    b > (a - da) && b < (a + da)
}

fn polygon_from_tree<T>(dataset: &rstar::RTree<Coord<T>>) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    assert!(dataset.size() <= 3);

    let mut coords: Vec<Coord<T>> = dataset.iter().cloned().collect();
    if !coords.is_empty() {
        // 关闭线串，只要它不为空
        coords.push(coords[0]);
    }

    Polygon::new(LineString::from(coords), vec![])
}

fn concave_hull_inner<T>(original_dataset: rstar::RTree<Coord<T>>, k: u32) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    let set_length = original_dataset.size();
    if set_length <= 3 {
        return polygon_from_tree(&original_dataset);
    }
    if k >= set_length as u32 {
        return fall_back_hull(&original_dataset);
    }

    let k_adjusted = adjust_k(k);
    let mut dataset = original_dataset.clone();

    let first_coord = get_first_coord(&dataset);
    let mut hull = vec![first_coord];

    let mut current_coord = first_coord;
    dataset.remove(&first_coord);

    let mut prev_coord = current_coord;
    let mut curr_step = 2;
    while (current_coord != first_coord || curr_step == 2) && dataset.size() > 0 {
        if curr_step == 5 {
            // 插入第一个坐标以闭合环
            dataset.insert(first_coord);
        }

        let mut nearest_coords: Vec<_> =
            get_nearest_coords(&dataset, &current_coord, k_adjusted).collect();
        sort_by_angle(&mut nearest_coords, &current_coord, &prev_coord);

        let selected = nearest_coords
            .iter()
            .find(|x| !intersects(&hull, &[&current_coord, x]));

        if let Some(sel) = selected {
            prev_coord = current_coord;
            current_coord = **sel;
            hull.push(current_coord);
            dataset.remove(&current_coord);

            curr_step += 1;
        } else {
            return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
        }
    }

    let poly = Polygon::new(LineString::from(hull), vec![]);

    if original_dataset
        .iter()
        .any(|&coord| !coord_inside(&coord, &poly))
    {
        return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
    }

    poly
}

fn fall_back_hull<T>(dataset: &rstar::RTree<Coord<T>>) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    let multipoint = MultiPoint::from(dataset.iter().cloned().collect::<Vec<Coord<T>>>());
    multipoint.convex_hull()
}

fn get_next_k(curr_k: u32) -> u32 {
    max(curr_k + 1, ((curr_k as f32) * K_MULTIPLIER) as u32)
}

fn adjust_k(k: u32) -> u32 {
    max(k, 3)
}

fn get_first_coord<T>(coord_set: &rstar::RTree<Coord<T>>) -> Coord<T>
where
    T: GeoFloat + RTreeNum,
{
    let mut min_y = Float::max_value();
    let mut result = coord_set
        .iter()
        .next()
        .expect("我们之前检查过集中的坐标数大于3。");

    for coord in coord_set.iter() {
        if coord.y < min_y {
            min_y = coord.y;
            result = coord;
        }
    }

    *result
}

fn get_nearest_coords<'a, T>(
    dataset: &'a rstar::RTree<Coord<T>>,
    base_coord: &Coord<T>,
    candidate_no: u32,
) -> impl Iterator<Item = &'a Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    dataset
        .nearest_neighbor_iter(base_coord)
        .take(candidate_no as usize)
}

fn sort_by_angle<T>(coords: &mut [&Coord<T>], curr_coord: &Coord<T>, prev_coord: &Coord<T>)
where
    T: GeoFloat,
{
    let base_angle = pseudo_angle(prev_coord.x - curr_coord.x, prev_coord.y - curr_coord.y);
    coords.sort_by(|a, b| {
        let mut angle_a = pseudo_angle(a.x - curr_coord.x, a.y - curr_coord.y) - base_angle;
        if angle_a < T::zero() {
            angle_a = angle_a + T::from(4.0).unwrap();
        }

        let mut angle_b = pseudo_angle(b.x - curr_coord.x, b.y - curr_coord.y) - base_angle;
        if angle_b < T::zero() {
            angle_b = angle_b + T::from(4.0).unwrap();
        }

        angle_a.partial_cmp(&angle_b).unwrap().reverse()
    });
}

fn pseudo_angle<T>(dx: T, dy: T) -> T
where
    T: GeoFloat,
{
    if dx == T::zero() && dy == T::zero() {
        return T::zero();
    }

    let p = dx / (dx.abs() + dy.abs());
    if dy < T::zero() {
        T::from(3.).unwrap() + p
    } else {
        T::from(1.).unwrap() - p
    }
}

fn intersects<T>(hull: &[Coord<T>], line: &[&Coord<T>; 2]) -> bool
where
    T: GeoFloat,
{
    // 这是完成轮廓的情况。
    if *line[1] == hull[0] {
        return false;
    }

    let coords = hull.iter().take(hull.len() - 1).cloned().collect();
    let linestring = LineString::new(coords);
    let line = crate::Line::new(*line[0], *line[1]);
    linestring.intersects(&line)
}

fn coord_inside<T>(coord: &Coord<T>, poly: &Polygon<T>) -> bool
where
    T: GeoFloat,
{
    poly.contains(coord) || poly.exterior().contains(coord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords_iter::CoordsIter;
    use geo_types::coord;

    #[test]
    fn coord_ordering() {
        let coords = [
            coord!(x: 1.0, y: 1.0),
            coord!(x: -1.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 1.0, y: 0.0),
        ];

        let mut coords_mapped: Vec<&Coord<f32>> = coords.iter().collect();

        let center = coord!(x: 0.0, y: 0.0);
        let prev_coord = coord!(x: 1.0, y: 1.0);

        let expected = vec![&coords[3], &coords[1], &coords[2], &coords[0]];

        sort_by_angle(&mut coords_mapped, &center, &prev_coord);
        assert_eq!(coords_mapped, expected);

        let expected = vec![&coords[1], &coords[2], &coords[0], &coords[3]];

        let prev_coord = coord!(x: 1.0, y: -1.0);
        sort_by_angle(&mut coords_mapped, &center, &prev_coord);
        assert_eq!(coords_mapped, expected);
    }

    #[test]
    fn get_first_coord_test() {
        let coords = vec![
            coord!(x: 1.0, y: 1.0),
            coord!(x: -1.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 0.0, y: 0.5),
        ];
        let tree = rstar::RTree::bulk_load(coords);
        let first = coord!(x: -1.0, y: 0.0);

        assert_eq!(get_first_coord(&tree), first);
    }

    #[test]
    fn concave_hull_test() {
        let coords = vec![
            coord!(x: 0.0, y: 0.0),
            coord!(x: 1.0, y: 0.0),
            coord!(x: 2.0, y: 0.0),
            coord!(x: 3.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 1.0, y: 1.0),
            coord!(x: 2.0, y: 1.0),
            coord!(x: 3.0, y: 1.0),
            coord!(x: 0.0, y: 2.0),
            coord!(x: 1.0, y: 2.5),
            coord!(x: 2.0, y: 2.5),
            coord!(x: 3.0, y: 2.0),
            coord!(x: 0.0, y: 3.0),
            coord!(x: 3.0, y: 3.0),
        ];

        let poly = concave_hull(coords.iter(), 3);
        assert_eq!(poly.exterior().coords_count(), 12);

        let must_not_be_in = [&coords[6]];
        for coord in poly.exterior().coords_iter() {
            for not_coord in must_not_be_in.iter() {
                assert_ne!(&coord, *not_coord);
            }
        }
    }

    #[test]
    fn empty_hull() {
        let actual: Polygon<f64> = concave_hull([].iter(), 3);
        let expected = Polygon::new(LineString::new(vec![]), vec![]);
        assert_eq!(actual, expected);
    }
}
