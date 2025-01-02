use super::{CoordPos, Direction};

use std::fmt;

/// `TopologyPosition` 是图形组件的拓扑关系标签，
/// 对应到一个几何体的每个 [`Direction`](Direction)。
///
/// 如果图形组件是一个区域边缘，则每个 [`Direction`] 上都有一个位置：
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum TopologyPosition {
    Area {
        on: Option<CoordPos>,    // 在边上
        left: Option<CoordPos>,  // 边的左侧
        right: Option<CoordPos>, // 边的右侧
    },
    LineOrPoint {
        on: Option<CoordPos>, // 对于线或节点，只有一个 `On` 位置的拓扑关系
    },
}

impl fmt::Debug for TopologyPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_position(position: &Option<CoordPos>, f: &mut fmt::Formatter) -> fmt::Result {
            match position {
                Some(CoordPos::Inside) => write!(f, "i"),     // 内部
                Some(CoordPos::OnBoundary) => write!(f, "b"), // 在边界上
                Some(CoordPos::Outside) => write!(f, "e"),    // 外部
                None => write!(f, "_"),                       // 无
            }
        }
        match self {
            Self::LineOrPoint { on } => fmt_position(on, f)?, // 针对线或点
            Self::Area { on, left, right } => {
                // 针对区域
                fmt_position(left, f)?;
                fmt_position(on, f)?;
                fmt_position(right, f)?;
            }
        }
        Ok(())
    }
}

impl TopologyPosition {
    pub fn area(on: CoordPos, left: CoordPos, right: CoordPos) -> Self {
        Self::Area {
            on: Some(on),       // 设置在边上的位置
            left: Some(left),   // 设置左侧位置
            right: Some(right), // 设置右侧位置
        }
    }

    pub fn empty_area() -> Self {
        Self::Area {
            on: None,    // 无在边上位置
            left: None,  // 无左侧位置
            right: None, // 无右侧位置
        }
    }

    pub fn line_or_point(on: CoordPos) -> Self {
        Self::LineOrPoint { on: Some(on) } // 设置线或点位置
    }

    pub fn empty_line_or_point() -> Self {
        Self::LineOrPoint { on: None } // 无线或点位置
    }

    pub fn get(&self, direction: Direction) -> Option<CoordPos> {
        match (direction, self) {
            (Direction::Left, Self::Area { left, .. }) => *left,
            (Direction::Right, Self::Area { right, .. }) => *right,
            (Direction::On, Self::LineOrPoint { on }) | (Direction::On, Self::Area { on, .. }) => {
                *on
            }
            (_, Self::LineOrPoint { .. }) => {
                panic!("线或点只对 `Direction::On` 有位置") // 线或点只能含有 `On` 位置
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        // 检查是否为空
        matches!(
            self,
            Self::LineOrPoint { on: None }
                | Self::Area {
                    on: None,
                    left: None,
                    right: None,
                }
        )
    }

    pub fn is_any_empty(&self) -> bool {
        // 检查是否有任意为空
        !matches!(
            self,
            Self::LineOrPoint { on: Some(_) }
                | Self::Area {
                    on: Some(_),
                    left: Some(_),
                    right: Some(_),
                }
        )
    }

    pub fn is_area(&self) -> bool {
        // 检查是否为区域
        matches!(self, Self::Area { .. })
    }

    pub fn is_line(&self) -> bool {
        // 检查是否为线
        matches!(self, Self::LineOrPoint { .. })
    }

    pub fn flip(&mut self) {
        // 翻转左右位置
        match self {
            Self::LineOrPoint { .. } => {}
            Self::Area { left, right, .. } => {
                std::mem::swap(left, right);
            }
        }
    }

    pub fn set_all_positions(&mut self, position: CoordPos) {
        // 设置所有位置
        match self {
            Self::LineOrPoint { on } => {
                *on = Some(position);
            }
            Self::Area { on, left, right } => {
                *on = Some(position);
                *left = Some(position);
                *right = Some(position);
            }
        }
    }

    pub fn set_all_positions_if_empty(&mut self, position: CoordPos) {
        // 如果为空则设置所有位置
        match self {
            Self::LineOrPoint { on } => {
                if on.is_none() {
                    *on = Some(position);
                }
            }
            Self::Area { on, left, right } => {
                if on.is_none() {
                    *on = Some(position);
                }
                if left.is_none() {
                    *left = Some(position);
                }
                if right.is_none() {
                    *right = Some(position);
                }
            }
        }
    }

    pub fn set_position(&mut self, direction: Direction, position: CoordPos) {
        // 设置指定位置
        match (direction, self) {
            (Direction::On, Self::LineOrPoint { on }) => *on = Some(position),
            (_, Self::LineOrPoint { .. }) => {
                panic!("为 Self::Line 指定了无效的赋值维度") // 无效的赋值操作
            }
            (Direction::On, Self::Area { on, .. }) => *on = Some(position),
            (Direction::Left, Self::Area { left, .. }) => *left = Some(position),
            (Direction::Right, Self::Area { right, .. }) => *right = Some(position),
        }
    }

    pub fn set_on_position(&mut self, position: CoordPos) {
        // 设置 `On` 位置
        match self {
            Self::LineOrPoint { on } | Self::Area { on, .. } => {
                *on = Some(position);
            }
        }
    }

    pub fn set_locations(&mut self, new_on: CoordPos, new_left: CoordPos, new_right: CoordPos) {
        // 设置位置
        match self {
            Self::LineOrPoint { .. } => {
                error!("为 {self:?} 指定了无效的赋值维度");
                debug_assert!(false, "为 {self:?} 指定了无效的赋值维度");
            }
            Self::Area { on, left, right } => {
                *on = Some(new_on);
                *left = Some(new_left);
                *right = Some(new_right);
            }
        }
    }
}
