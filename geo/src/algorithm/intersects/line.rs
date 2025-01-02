use super::{point_in_rect, Intersects};
use crate::*;

impl<T> Intersects<Coord<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        // 首先检查点是否与线共线。
        T::Ker::orient2d(self.start, self.end, *rhs) == Orientation::Collinear
        // 此外，该点必须在 start 和 end 的边界内具有这两个坐标。
            && point_in_rect(*rhs, self.start, self.end)
    }
}
symmetric_intersects_impl!(Coord<T>, Line<T>);
symmetric_intersects_impl!(Line<T>, Point<T>);

impl<T> Intersects<Line<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        // 特殊情况：self 相当于一个点。
        if self.start == self.end {
            return line.intersects(&self.start);
        }

        // 前提条件：start 和 end 是不同的。

        // 检查 rhs.{start,end} 的方向相对于 self.{start,end} 是否不同。
        let check_1_1 = T::Ker::orient2d(self.start, self.end, line.start);
        let check_1_2 = T::Ker::orient2d(self.start, self.end, line.end);

        if check_1_1 != check_1_2 {
            // 由于检查结果不同，rhs.{start,end} 是不同的，并且 rhs 不与 self 共线。
            // 因此在 rhs 的无限延伸线上有确切的一个点与 self 共线。

            // 连续性表明，这个点不在 rhs 的外部。现在，交换 self 和 rhs 进行相同的检查。

            let check_2_1 = T::Ker::orient2d(line.start, line.end, self.start);
            let check_2_2 = T::Ker::orient2d(line.start, line.end, self.end);

            // 通过类似的论证，在 self 上有确切的一个点与 rhs 共线。
            // 因此，这两个必须是相同的，并且位于（内部或边界，但不在外部）在两条线中。
            check_2_1 != check_2_2
        } else if check_1_1 == Orientation::Collinear {
            // 特殊情况：共线的线段。

            // 相当于 4 个点线相交检查，但移除对核谓词的调用。
            point_in_rect(line.start, self.start, self.end)
                || point_in_rect(line.end, self.start, self.end)
                || point_in_rect(self.end, line.start, line.end)
                || point_in_rect(self.end, line.start, line.end)
        } else {
            false
        }
    }
}

impl<T> Intersects<Triangle<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        self.intersects(&rhs.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T>, Line<T>);
