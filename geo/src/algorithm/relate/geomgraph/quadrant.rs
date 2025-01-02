use crate::GeoNum;

/// 用于处理笛卡尔平面象限的实用函数，
/// 标签如下：
/**
 *          (+)
 *        NW ┃ NE
 *    (-) ━━━╋━━━━ (+)
 *        SW ┃ SE
 *          (-)
 */
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
pub enum Quadrant {
    NE, // 东北象限
    NW, // 西北象限
    SW, // 西南象限
    SE, // 东南象限
}

impl Quadrant {
    /// 创建新的象限实例
    pub fn new<F: GeoNum>(dx: F, dy: F) -> Option<Quadrant> {
        // 如果 dx 和 dy 都为零，返回 None
        if dx.is_zero() && dy.is_zero() {
            return None;
        }

        // 根据 dx 和 dy 确定所在象限
        match (dy >= F::zero(), dx >= F::zero()) {
            (true, true) => Quadrant::NE,
            (true, false) => Quadrant::NW,
            (false, false) => Quadrant::SW,
            (false, true) => Quadrant::SE,
        }
        .into()
    }
}
