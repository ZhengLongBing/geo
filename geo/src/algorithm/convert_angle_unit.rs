use geo_types::Coord;
use geo_types::CoordFloat;

use crate::{MapCoords, MapCoordsInPlace};

/// 定义将坐标转换为弧度的特征
///
/// `ToRadians` 添加了两个方法：`to_radians` 用于返回新实例，`to_radians_in_place` 用于原地转换。
pub trait ToRadians<T: CoordFloat>:
    Sized + MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>
{
    /// 将坐标转换为弧度，并返回一个新实例
    fn to_radians(&self) -> Self {
        self.map_coords(|Coord { x, y }| Coord {
            x: x.to_radians(),
            y: y.to_radians(),
        })
    }

    /// 原地将坐标转换为弧度
    fn to_radians_in_place(&mut self) {
        self.map_coords_in_place(|Coord { x, y }| Coord {
            x: x.to_radians(),
            y: y.to_radians(),
        })
    }
}
impl<T: CoordFloat, G: MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>> ToRadians<T> for G {}

/// 定义将坐标转换为角度的特征
///
/// `ToDegrees` 添加了两个方法：`to_degrees` 用于返回新实例，`to_degrees_in_place` 用于原地转换。
pub trait ToDegrees<T: CoordFloat>:
    Sized + MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>
{
    /// 将坐标转换为角度，并返回一个新实例
    fn to_degrees(&self) -> Self {
        self.map_coords(|Coord { x, y }| Coord {
            x: x.to_degrees(),
            y: y.to_degrees(),
        })
    }

    /// 原地将坐标转换为角度
    fn to_degrees_in_place(&mut self) {
        self.map_coords_in_place(|Coord { x, y }| Coord {
            x: x.to_degrees(),
            y: y.to_degrees(),
        })
    }
}
impl<T: CoordFloat, G: MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>> ToDegrees<T> for G {}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use approx::assert_relative_eq;
    use geo_types::Line;

    use super::*;

    /// 模拟一个以度表示的线
    fn line_degrees_mock() -> Line {
        Line::new((90.0, 180.), (0., -90.))
    }

    /// 模拟一个以弧度表示的线
    fn line_radians_mock() -> Line {
        Line::new((PI / 2., PI), (0., -PI / 2.))
    }

    #[test]
    /// 测试 `to_radians` 方法
    fn converts_to_radians() {
        assert_relative_eq!(line_radians_mock(), line_degrees_mock().to_radians())
    }

    #[test]
    /// 测试 `to_radians_in_place` 方法
    fn converts_to_radians_in_place() {
        let mut line = line_degrees_mock();
        line.to_radians_in_place();
        assert_relative_eq!(line_radians_mock(), line)
    }

    #[test]
    /// 测试 `to_degrees` 方法
    fn converts_to_degrees() {
        assert_relative_eq!(line_degrees_mock(), line_radians_mock().to_degrees())
    }

    #[test]
    /// 测试 `to_degrees_in_place` 方法
    fn converts_to_degrees_in_place() {
        let mut line = line_radians_mock();
        line.to_degrees_in_place();
        assert_relative_eq!(line_degrees_mock(), line)
    }
}
