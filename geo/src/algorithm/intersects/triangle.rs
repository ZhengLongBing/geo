use super::Intersects;
use crate::*;

impl<T> Intersects<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        // 计算三角形的每条边相对于给定点的方向
        let mut orientations = self
            .to_lines()
            .map(|l| T::Ker::orient2d(l.start, l.end, *rhs));

        orientations.sort(); // 对方向进行排序

        // 检查方向是否不一致，且任何一个方向不是共线的
        !orientations
            .windows(2)
            .any(|win| win[0] != win[1] && win[1] != Orientation::Collinear)

        // // 忽略鲁棒性谓词，因而速度更快
        // let p0x = self.0.x.to_f64().unwrap(); // 第一个顶点的x坐标
        // let p0y = self.0.y.to_f64().unwrap(); // 第一个顶点的y坐标
        // let p1x = self.1.x.to_f64().unwrap(); // 第二个顶点的x坐标
        // let p1y = self.1.y.to_f64().unwrap(); // 第二个顶点的y坐标
        // let p2x = self.2.x.to_f64().unwrap(); // 第三个顶点的x坐标
        // let p2y = self.2.y.to_f64().unwrap(); // 第三个顶点的y坐标

        // let px = rhs.x.to_f64().unwrap(); // 点的x坐标
        // let py = rhs.y.to_f64().unwrap(); // 点的y坐标

        // 计算s的值用于判断点的相对位置
        // let s = (p0x - p2x) * (py - p2y) - (p0y - p2y) * (px - p2x);
        // 计算t的值用于判断点的相对位置
        // let t = (p1x - p0x) * (py - p0y) - (p1y - p0y) * (px - p0x);

        // 检查s和t符号是否不同，以及s和t是否不为零
        // if (s < 0.) != (t < 0.) && s != 0. && t != 0. {
        //     return false;
        // }

        // 计算d的值用于判断点的相对位置
        // let d = (p2x - p1x) * (py - p1y) - (p2y - p1y) * (px - p1x);
        // 检查d是否为零或其符号与s和t之和的关系
        // d == 0. || (d < 0.) == (s + t <= 0.)
    }
}

symmetric_intersects_impl!(Coord<T>, Triangle<T>);
symmetric_intersects_impl!(Triangle<T>, Point<T>);

impl<T> Intersects<Triangle<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        // 将三角形转换为多边形后调用相交函数
        self.to_polygon().intersects(&rhs.to_polygon())
    }
}
