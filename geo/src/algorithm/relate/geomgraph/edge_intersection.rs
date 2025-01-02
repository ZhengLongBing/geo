use crate::{Coord, GeoFloat};

/// 表示一个边上的点，该点与另一个边相交。
///
/// 相交可能是一个单一的点，也可能是一个线段（在这种情况下，这个点是线段的起始点）。相交点必须是精确的。
///
/// 基于 [JTS's EdgeIntersection 截止至版本 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeIntersection.java)
#[derive(Debug, Clone)]
pub(crate) struct EdgeIntersection<F: GeoFloat> {
    coord: Coord<F>,
    segment_index: usize,
    dist: F,
}

impl<F: GeoFloat> EdgeIntersection<F> {
    /// 创建一个新的 `EdgeIntersection` 实例
    pub fn new(coord: Coord<F>, segment_index: usize, dist: F) -> EdgeIntersection<F> {
        EdgeIntersection {
            coord,
            segment_index,
            dist,
        }
    }

    /// 返回相交的坐标
    pub fn coordinate(&self) -> Coord<F> {
        self.coord
    }

    /// 返回段的索引
    pub fn segment_index(&self) -> usize {
        self.segment_index
    }

    /// 返回距离
    pub fn distance(&self) -> F {
        self.dist
    }
}

impl<F: GeoFloat> std::cmp::PartialEq for EdgeIntersection<F> {
    /// 判断两个`EdgeIntersection`是否相等
    fn eq(&self, other: &EdgeIntersection<F>) -> bool {
        self.segment_index == other.segment_index && self.dist == other.dist
    }
}

impl<F: GeoFloat> std::cmp::Eq for EdgeIntersection<F> {}

impl<F: GeoFloat> std::cmp::PartialOrd for EdgeIntersection<F> {
    /// 部分比较，返回 `Option<std::cmp::Ordering>`
    fn partial_cmp(&self, other: &EdgeIntersection<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: GeoFloat> std::cmp::Ord for EdgeIntersection<F> {
    /// 全量比较，返回`std::cmp::Ordering`，“Ord”在`BTreeMap`中要求节点完全可比较，但由于比较的是浮点数，因此要求结果中的值不是`NaN`。
    fn cmp(&self, other: &EdgeIntersection<F>) -> std::cmp::Ordering {
        if self.segment_index < other.segment_index {
            return std::cmp::Ordering::Less;
        }
        if self.segment_index > other.segment_index {
            return std::cmp::Ordering::Greater;
        }
        if self.dist < other.dist {
            return std::cmp::Ordering::Less;
        }
        if self.dist > other.dist {
            return std::cmp::Ordering::Greater;
        }

        // BTreeMap要求节点是完全可排序的，但我们比较的是浮点数，因此我们需要非NaN的有效结果。
        debug_assert!(!self.dist.is_nan() && !other.dist.is_nan());

        std::cmp::Ordering::Equal
    }
}

impl<F: GeoFloat> EdgeIntersection<F> {}
