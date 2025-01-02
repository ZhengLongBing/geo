use super::{CoordNum, Kernel, Orientation};
use crate::Coord;

use num_traits::{Float, NumCast};

/// 使用快速鲁棒谓词提供鲁棒浮点谓词的鲁棒内核。
/// 只应与能够始终转换为f64且无精度损失的类型一起使用。
#[derive(Default, Debug)]
pub struct RobustKernel;

impl<T> Kernel<T> for RobustKernel
where
    T: CoordNum + Float,
{
    fn orient2d(p: Coord<T>, q: Coord<T>, r: Coord<T>) -> Orientation {
        use robust::{orient2d, Coord};

        // 将点的坐标转换为f64进行计算，以获得鲁棒的2D方向。
        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from(p.x).unwrap(),
                y: <f64 as NumCast>::from(p.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(q.x).unwrap(),
                y: <f64 as NumCast>::from(q.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(r.x).unwrap(),
                y: <f64 as NumCast>::from(r.y).unwrap(),
            },
        );

        // 根据计算结果的符号确定方向：顺时针、逆时针或共线。
        if orientation < 0. {
            Orientation::Clockwise
        } else if orientation > 0. {
            Orientation::CounterClockwise
        } else {
            Orientation::Collinear
        }
    }
}
