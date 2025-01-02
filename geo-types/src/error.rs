use core::fmt;

// 定义错误类型
#[derive(Debug)]
pub enum Error {
    // 几何类型不匹配错误
    MismatchedGeometry {
        expected: &'static str,
        found: &'static str,
    },
}

// 当启用 std 特性时，实现标准错误特征
#[cfg(feature = "std")]
impl std::error::Error for Error {}

// 实现 Display 特征以格式化错误信息
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MismatchedGeometry { expected, found } => {
                write!(f, "Expected a {expected}, but found a {found}")
            }
        }
    }
}

// 测试模块
#[cfg(test)]
mod test {
    use crate::{Geometry, Point, Rect};
    use alloc::string::ToString;
    use core::convert::TryFrom;

    // 测试错误输出
    #[test]
    fn error_output() {
        // 创建点和对应的几何体
        let point = Point::new(1.0, 2.0);
        let point_geometry = Geometry::from(point);

        // 创建矩形和对应的几何体
        let rect = Rect::new(Point::new(1.0, 2.0), Point::new(3.0, 4.0));
        let rect_geometry = Geometry::from(rect);

        // 尝试将点几何体转换回点（应该成功）
        Point::try_from(point_geometry).expect("failed to unwrap inner enum Point");

        // 尝试将矩形几何体转换为点（应该失败）
        let failure = Point::try_from(rect_geometry).unwrap_err();
        assert_eq!(
            failure.to_string(),
            "Expected a geo_types::geometry::point::Point, but found a geo_types::geometry::rect::Rect"
        );
    }
}
