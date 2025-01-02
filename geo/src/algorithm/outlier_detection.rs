use std::iter::Sum;
use std::ops::RangeInclusive;

use crate::{GeoFloat, MultiPoint, Point};

use rstar::primitives::GeomWithData;
use rstar::RTree;

/// 计算一组点的[局部离群因子](https://en.wikipedia.org/wiki/Local_outlier_factor)
///
/// 基于：Breunig, M., Kriegel, H., Ng, R., 和 Sander, J. (2000). *LOF: identifying density-based local
/// outliers.* 在 ACM Int. Conf. on Management of Data, 第 93-104 页。doi: [10.1145/335191.335388](https://doi.org/10.1145/335191.335388)
///
/// LOF 是一种使用局部数据进行异常检测的无监督算法。
///
/// 离群和内部分类对于数据形态**高度依赖**。LOF 值 <= 1 通常可以被认为是内部点，但如一个高度集中的统一数据集可能导致
/// LOF 为 1.05 的点是离群点。
/// LOF 得分因此应在整个数据集的背景下进行评估，以便分类离群点。
///
/// 如果你想运行多个具有不同邻居数量的离群检测过程以构建更稳健的检测数据(参见上文第 100-1 页)，可以使用 [`OutlierDetection::prepared_detector`] 方法，它在运行之间保留
/// 空间索引和点集以提高效率。[`OutlierDetection::generate_ensemble`] 方法
/// 将有效地在邻居输入的连续范围上运行 LOF 算法，允许在生成的数据上进行聚合。
pub trait OutlierDetection<T>
where
    T: GeoFloat,
{
    /// LOF 算法。 `k_neighbours` 指定用于局部离群分类的邻居数。上文链接的论文(参见第 100 页)建议将 `k_neighbours` 值设定为 10 - 20
    /// 作为“现实世界”数据的下限。
    ///
    /// # 关于错误输入的注意事项
    /// 如果 `k_neighbours` >= 集合中的点或 `k_neighbours` < 1，所有输入点都会返回 LOF 值为 1。
    /// 如果输入点有至少 `k_neighbours` 个重复点，LOF 值可以是 `∞` 或 `NaN`。
    /// 因此建议**去重**或确保输入点的唯一性。
    ///
    /// # 关于返回点的注意事项
    /// 离群得分总是对应于输入点的顺序返回。
    ///
    /// # 示例
    ///
    /// ## 多点
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::OutlierDetection;
    /// use geo::{point, MultiPoint};
    ///
    /// let v = vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 0.0, y: 1.0),
    ///     point!(x: 3.0, y: 0.0),
    ///     point!(x: 1.0, y: 1.0),
    /// ];
    ///
    /// let lofscores = v.outliers(2);
    /// // 第三个点是离群点，导致较大的 LOF 得分
    /// assert_relative_eq!(lofscores[2], 3.0);
    /// // 最后一个点是内部点，导致较小的 LOF 得分
    /// assert_relative_eq!(lofscores[3], 1.0);
    /// ```
    ///
    /// ## 计算索引，根据 LOF 得分排序
    ///```
    /// use geo::OutlierDetection;
    /// use geo::{point, MultiPoint};
    ///
    /// // 这些点包含4个强离群点
    /// let v = vec![
    ///     point!(x: 0.16, y: 0.14),
    ///     point!(x: 0.15, y: 0.33),
    ///     point!(x: 0.37, y: 0.25),
    ///     point!(x: 0.3 , y: 0.4),
    ///     point!(x: 0.3 , y: 0.1),
    ///     point!(x: 0.3 , y: 0.2),
    ///     point!(x: 1.3 , y: 2.3),
    ///     point!(x: 1.7 , y: 0.2),
    ///     point!(x: 0.7 , y: -0.9),
    ///     point!(x: 0.21, y: 2.45),
    ///     point!(x: 0.8 , y: 0.7),
    ///     point!(x: 0.9 , y: 0.7),
    ///     point!(x: 0.8 , y: 0.6),
    ///     point!(x: 0.73, y: 0.65),
    ///     point!(x: 0.9 , y: 0.6),
    ///     point!(x: 1.0, y: 0.6),
    ///     point!(x: 1.0, y: 0.7),
    ///     point!(x: 0.25, y: 0.29),
    ///     point!(x: 0.2 , y: 0.2),
    /// ];
    /// let lofs = &mut v.outliers(3);
    /// let mut idx_lofs: Vec<(usize, f64)> = lofs
    ///     .iter()
    ///     .enumerate()
    ///     .map(|(idx, score)| (idx, *score))
    ///     .collect();
    /// // 根据 LOF 得分排序
    /// idx_lofs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    /// // 最有可能的离群点优先
    /// idx_lofs.reverse();
    /// // 四个离群点，LOF 得分远高于 10
    /// idx_lofs
    ///     .iter()
    ///     .take(4)
    ///     .for_each(|score| assert!(score.1 > 10.0));
    ///```
    fn outliers(&self, k_neighbours: usize) -> Vec<T>;

    /// 创建一个准备好的离群检测器，允许多次运行以保留使用中的空间索引。
    /// 一个[`PreparedDetector`]可以高效地重新计算不同 `k_neigbhours` 值的离群点。
    fn prepared_detector(&self) -> PreparedDetector<T>;

    /// 在`k_neighbours`值在`bounds`之间执行连续的运行，
    /// 生成一个 LOF 得分的集合，可以使用如 min、max 或 mean 进行聚合。
    ///
    /// # 示例
    ///```
    /// use geo::OutlierDetection;
    /// use geo::{point, Point, MultiPoint};
    /// let v: Vec<Point<f64>> = vec![
    ///     point!(x: 0.16, y: 0.14),
    ///     point!(x: 0.15, y: 0.33),
    ///     point!(x: 0.37, y: 0.25),
    ///     point!(x: 0.3 , y: 0.4),
    ///     point!(x: 0.3 , y: 0.1),
    ///     point!(x: 0.3 , y: 0.2),
    ///     point!(x: 1.3 , y: 2.3),
    ///     point!(x: 1.7 , y: 0.2),
    ///     point!(x: 0.7 , y: -0.9),
    ///     point!(x: 0.21, y: 2.45),
    ///     point!(x: 0.8 , y: 0.7),
    ///     point!(x: 0.9 , y: 0.7),
    ///     point!(x: 0.8 , y: 0.6),
    ///     point!(x: 0.73, y: 0.65),
    ///     point!(x: 0.9 , y: 0.6),
    ///     point!(x: 1.0, y: 0.6),
    ///     point!(x: 1.0, y: 0.7),
    ///     point!(x: 0.25, y: 0.29),
    ///     point!(x: 0.2 , y: 0.2),
    /// ];
    /// let ensemble = v.generate_ensemble((2..=5));
    /// // 保留每次运行中每个点的最大 LOF 值
    /// // 这将导致一个单独的 Vec
    /// let aggregated = ensemble[1..].iter().fold(ensemble[0].clone(), |acc, xs| {
    ///     acc.iter()
    ///         .zip(xs)
    ///         .map(|(elem1, elem2)| elem1.min(*elem2))
    ///         .collect()
    /// });
    /// assert_eq!(v.len(), aggregated.len());
    ///```
    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>>;

    /// 便捷方法来有效计算 LOF 集合的最小值
    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T>;

    /// 便捷方法来有效计算 LOF 集合的最大值
    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T>;
}

/// 此结构体允许在点集中使用不同的 `k_neighbours` 大小进行多次检测操作，
/// 而无需重建底层的空间索引。它的 [`PreparedDetector::outliers`] 方法
/// 与 [`OutlierDetection::outliers`] 方法具有相同的签名，但保留底层的空间索引和点集，
/// 提高效率。
#[derive(Clone, Debug)]
pub struct PreparedDetector<'a, T>
where
    T: GeoFloat,
{
    tree: RTree<GeomWithData<Point<T>, usize>>,
    points: &'a [Point<T>],
}

impl<'a, T> PreparedDetector<'a, T>
where
    T: GeoFloat + Sum,
{
    /// 创建一个新的"准备好"的检测器，允许重复进行具有不同邻居大小的 LOF 算法调用
    fn new(points: &'a [Point<T>]) -> Self {
        let geoms: Vec<GeomWithData<_, usize>> = points
            .iter()
            .enumerate()
            .map(|(idx, point)| GeomWithData::new(*point, idx))
            .collect();
        let tree = RTree::bulk_load(geoms);
        Self { tree, points }
    }

    /// 参见 [`OutlierDetection::outliers`] 以了解用法
    pub fn outliers(&self, kneighbours: usize) -> Vec<T> {
        lof(self.points, &self.tree, kneighbours)
    }
}

fn lof<T>(
    points: &[Point<T>],
    tree: &RTree<GeomWithData<Point<T>, usize>>,
    kneighbours: usize,
) -> Vec<T>
where
    T: GeoFloat + Sum,
{
    debug_assert!(kneighbours > 0);
    if points.len() <= kneighbours || kneighbours < 1 {
        // 在这种情况下没有必要尝试运行算法
        return points.iter().map(|_| T::one()).collect();
    }
    let knn_dists = points
        .iter()
        .map(|point| {
            tree.nearest_neighbor_iter_with_distance_2(point)
                .take(kneighbours)
                .collect()
        })
        .collect::<Vec<Vec<_>>>();
    // 计算每个点的 LRD（局部可达性密度）
    // LRD 是一个点可以被其邻居找到的估计距离：
    // count(neighbour_set) / sum(max(point.kTh_dist, point.dist2(另一个点)) 对于邻域集中的所有点)
    // 我们称这个最大距离之和为 reachDistance
    let local_reachability_densities: Vec<T> = knn_dists
        .iter()
        .map(|neighbours| {
            // 对于每个点的邻居集，计算第k个距离
            let kth_dist = neighbours
                .iter()
                .map(|(_, distance)| distance)
                .last()
                .unwrap();
            T::from(neighbours.len()).unwrap()
                / neighbours
                    .iter()
                    // 对邻居集中邻居距离和第k个距离之间的最大值求和
                    .map(|(_, distance)| distance.max(*kth_dist))
                    .sum()
        })
        .collect();
    // 一个点 p 的 LOF 是所有点的 LRD 之和
    // 在集合 kNearestSet(p) 中 * 对第一个点 p，所有点的 reachDistance 之和，
    // 除以 p 的 kNN 集中项数的平方。
    knn_dists
        .iter()
        .map(|neighbours| {
            // 对于每个点的邻居集，计算第k个距离
            let kth_dist = neighbours
                .iter()
                .map(|(_, distance)| distance)
                .last()
                .unwrap();
            // 邻居集 LRD 得分之和
            let lrd_scores: T = neighbours
                .iter()
                .map(|(neighbour, _)| local_reachability_densities[neighbour.data])
                .sum();
            // 求和邻居集合的 reachDistance
            let sum_rd: T = neighbours
                .iter()
                .map(|(_, distance)| distance.max(*kth_dist))
                .sum();
            (lrd_scores * sum_rd) / T::from(neighbours.len().pow(2)).unwrap()
        })
        .collect()
}

impl<T> OutlierDetection<T> for MultiPoint<T>
where
    T: GeoFloat + Sum,
{
    fn outliers(&self, k_neighbours: usize) -> Vec<T> {
        let pd = self.prepared_detector();
        pd.outliers(k_neighbours)
    }

    fn prepared_detector(&self) -> PreparedDetector<T> {
        PreparedDetector::new(&self.0)
    }

    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>> {
        let pd = self.prepared_detector();
        bounds.map(|kneighbours| pd.outliers(kneighbours)).collect()
    }
    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.min(b)).collect())
            .unwrap()
    }

    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.max(b)).collect())
            .unwrap()
    }
}

impl<T> OutlierDetection<T> for [Point<T>]
where
    T: GeoFloat + Sum,
{
    fn outliers(&self, k_neighbours: usize) -> Vec<T> {
        let pd = self.prepared_detector();
        pd.outliers(k_neighbours)
    }

    fn prepared_detector(&self) -> PreparedDetector<T> {
        PreparedDetector::new(self)
    }

    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>> {
        let pd = self.prepared_detector();
        bounds.map(|kneighbours| pd.outliers(kneighbours)).collect()
    }

    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.min(b)).collect())
            .unwrap()
    }

    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.max(b)).collect())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn test_lof() {
        // 第三个点是离群点
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(3.0, 0.0),
            Point::new(1.0, 1.0),
        ];

        let lofs = &v.outliers(3);
        assert_eq!(lofs[2], 3.3333333333333335);
    }
    #[test]
    fn test_lof2() {
        // 第四个点是离群点
        let v = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 3.0),
        ];
        let lofs = &v.outliers(3);
        assert_eq!(lofs[3], 3.3333333333333335);
    }
    #[test]
    fn test_lof3() {
        // 第二个点是离群点，排序并反转，因此分数按降序排列
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 3.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
        ];
        let lofs = &mut v.outliers(3);
        lofs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lofs.reverse();
        assert_eq!(lofs[0], 3.3333333333333335);
    }
    #[test]
    fn test_lof4() {
        // 该数据集包含4个离群点
        // 应检测出索引 6, 7, 8, 9
        // 顺序：9, 6, 8, 7
        let v = vec![
            point!(x: 0.16, y: 0.14),
            point!(x: 0.15, y: 0.33),
            point!(x: 0.37, y: 0.25),
            point!(x: 0.3 , y: 0.4),
            point!(x: 0.3 , y: 0.1),
            point!(x: 0.3 , y: 0.2),
            point!(x: 1.3 , y: 2.3),
            point!(x: 1.7 , y: 0.2),
            point!(x: 0.7 , y: -0.9),
            point!(x: 0.21, y: 2.45),
            point!(x: 0.8 , y: 0.7),
            point!(x: 0.9 , y: 0.7),
            point!(x: 0.8 , y: 0.6),
            point!(x: 0.73, y: 0.65),
            point!(x: 0.9 , y: 0.6),
            point!(x: 1.0, y: 0.6),
            point!(x: 1.0, y: 0.7),
            point!(x: 0.25, y: 0.29),
            point!(x: 0.2 , y: 0.2),
        ];
        let lofs = &mut v.outliers(3);
        let mut idx_lofs: Vec<(usize, f64)> = lofs
            .iter()
            .enumerate()
            .map(|(idx, score)| (idx, *score))
            .collect();
        idx_lofs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        idx_lofs.reverse();
        // 四个离群点，得分远高于 10
        idx_lofs
            .iter()
            .take(4)
            .for_each(|score| assert!(score.1 > 10.0));
        // 剩下的低于 2
        idx_lofs
            .iter()
            .skip(4)
            .for_each(|score| assert!(score.1 < 2.0));
        // 确保得分计算正确
        assert_eq!(idx_lofs[0].0, 9);
        assert_eq!(idx_lofs[1].0, 6);
        assert_eq!(idx_lofs[2].0, 8);
        assert_eq!(idx_lofs[3].0, 7);
    }
    #[test]
    fn test_lof5() {
        // 第三个点是离群点
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(3.0, 0.0),
            Point::new(1.0, 1.0),
        ];

        let prepared = &v.prepared_detector();
        let s1 = prepared.outliers(2);
        let s2 = prepared.outliers(3);
        // 不同的邻居大小给出了不同的分数
        assert_ne!(s1[2], s2[2]);
    }
}
