use super::{CoordPos, Direction, TopologyPosition};

use std::fmt;

/// GeometryGraph 包含被标记为拓扑关系的组件（节点和边）。
///
/// 更准确地，每个 `Label` 为每个几何体保存一个 `TopologyPosition`，
/// 说明被标记的节点或边出现在几何体的 `Inside`，`Outside` 或 `OnBoundary`。
///
/// 对于线和点，一个 `TopologyPosition` 只跟踪一个 `On` 位置，
/// 而区域则有 `On`，`Left` 和 `Right` 位置。
///
/// 如果组件与某一个几何体没有任何结合关系，那么这个几何体的 `Label` 的
/// `TopologyPosition` 被称为 `empty`。
#[derive(Clone, PartialEq)]
pub(crate) struct Label {
    geometry_topologies: [TopologyPosition; 2],
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Label {{ A: {:?}, B: {:?} }}",
            &self.geometry_topologies[0], &self.geometry_topologies[1]
        )
    }
}

impl Label {
    pub fn swap_args(&mut self) {
        self.geometry_topologies.swap(0, 1)
    }

    /// 构造一个空的 `Label` 用于将一个一维线或零维点与两个几何体进行关联。
    pub fn empty_line_or_point() -> Label {
        Label {
            geometry_topologies: [
                TopologyPosition::empty_line_or_point(),
                TopologyPosition::empty_line_or_point(),
            ],
        }
    }

    /// 构造一个空的 `Label` 用于将一个二维区域与两个几何体进行关联。
    pub fn empty_area() -> Self {
        Self {
            geometry_topologies: [
                TopologyPosition::empty_area(),
                TopologyPosition::empty_area(),
            ],
        }
    }

    /// 构造一个用 `position` 初始化的 `Label` 为指定的几何体 `geom_index`。
    ///
    /// 标签在其他几何体的位置信息将被初始化为空。
    pub fn new(geom_index: usize, position: TopologyPosition) -> Self {
        let mut label = match position {
            TopologyPosition::LineOrPoint { .. } => Self::empty_line_or_point(),
            TopologyPosition::Area { .. } => Self::empty_area(),
        };
        label.geometry_topologies[geom_index] = position;
        label
    }

    pub fn flip(&mut self) {
        self.geometry_topologies[0].flip();
        self.geometry_topologies[1].flip();
    }

    pub fn position(&self, geom_index: usize, direction: Direction) -> Option<CoordPos> {
        self.geometry_topologies[geom_index].get(direction)
    }

    pub fn on_position(&self, geom_index: usize) -> Option<CoordPos> {
        self.geometry_topologies[geom_index].get(Direction::On)
    }

    pub fn set_position(&mut self, geom_index: usize, direction: Direction, position: CoordPos) {
        self.geometry_topologies[geom_index].set_position(direction, position);
    }

    pub fn set_on_position(&mut self, geom_index: usize, position: CoordPos) {
        self.geometry_topologies[geom_index].set_position(Direction::On, position);
    }

    pub fn set_all_positions(&mut self, geom_index: usize, position: CoordPos) {
        self.geometry_topologies[geom_index].set_all_positions(position)
    }

    pub fn set_all_positions_if_empty(&mut self, geom_index: usize, position: CoordPos) {
        self.geometry_topologies[geom_index].set_all_positions_if_empty(position)
    }

    pub fn geometry_count(&self) -> usize {
        self.geometry_topologies
            .iter()
            .filter(|location| !location.is_empty())
            .count()
    }

    pub fn is_empty(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_empty()
    }

    pub fn is_any_empty(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_any_empty()
    }

    pub fn is_area(&self) -> bool {
        self.geometry_topologies[0].is_area() || self.geometry_topologies[1].is_area()
    }

    pub fn is_geom_area(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_area()
    }

    pub fn is_line(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_line()
    }
}
