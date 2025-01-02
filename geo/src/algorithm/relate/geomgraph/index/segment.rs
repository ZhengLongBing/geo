use crate::Coord;
use crate::GeoFloat;

#[derive(Debug, Clone)]
pub(crate) struct Segment<F: GeoFloat + rstar::RTreeNum> {
    pub edge_idx: usize,                 // 边的索引
    pub segment_idx: usize,              // 线段的索引
    pub envelope: rstar::AABB<Coord<F>>, // 线段的外包围盒
}

impl<F> Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    // 创建新线段
    pub fn new(edge_idx: usize, segment_idx: usize, p1: Coord<F>, p2: Coord<F>) -> Self {
        Self {
            edge_idx,
            segment_idx,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<F> rstar::RTreeObject for Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Coord<F>>; // 外包围盒的类型

    // 获取线段的外包围盒
    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}
