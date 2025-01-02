use std::cmp::Ordering;

use super::SweepPoint;
use crate::GeoNum;

/// 扫描过程中生成的事件。
#[derive(Debug)]
pub(crate) struct Event<T: GeoNum, P> {
    pub point: SweepPoint<T>,
    pub ty: EventType,
    pub payload: P,
}

/// 用于有序集合中的相等性检查。注意，它忽略 segment_key。
impl<T: GeoNum, P> PartialEq for Event<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point && self.ty == other.ty
    }
}

/// 断言完全相等性。
impl<T: GeoNum, P> Eq for Event<T, P> {}

/// 用于最大堆（`BinaryHeap`）的排序。注意，它忽略 segment_key。
/// 这足以用于允许重复项的堆。
impl<T: GeoNum, P> PartialOrd for Event<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 从 `PartialOrd` 导出 `Ord` 并期望不失败。
impl<T: GeoNum, P> Ord for Event<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        // 这里的反转是为了符合最大堆/队列的实现。
        self.point
            .cmp(&other.point)
            .then_with(|| self.ty.cmp(&other.ty))
            .reverse()
    }
}

/// 扫描事件的类型。
///
/// 当扫描达到线段的起点/终点时，会生成扫描事件。此外，
/// 我们还支持扫描中的点几何，这在数学上解释为以点为中心的无限小垂直段。
///
/// 变体的顺序对算法很重要。我们要求右端点在左端点之前排序，以确保扫描的活动线段始终完全有序。
/// 点段被解释为点周围的无限小垂直段，因此它的左右事件分别在线变体之前和之后。
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub(crate) enum EventType {
    PointLeft,
    LineRight,
    LineLeft,
    PointRight,
}
