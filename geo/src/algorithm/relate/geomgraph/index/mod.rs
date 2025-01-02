mod edge_set_intersector; // 边集交互器模块
mod prepared_geometry; // 预处理的几何模块
mod rstar_edge_set_intersector; // R*树边集交互器模块
mod segment; // 线段模块
mod segment_intersector; // 线段交互器模块
mod simple_edge_set_intersector; // 简单边集交互器模块

pub(crate) use edge_set_intersector::EdgeSetIntersector; // 使用边集交互器
pub use prepared_geometry::PreparedGeometry; // 使用预处理的几何
pub(crate) use rstar_edge_set_intersector::RStarEdgeSetIntersector; // 使用R*树边集交互器
pub(crate) use segment::Segment; // 使用线段
pub(crate) use segment_intersector::SegmentIntersector; // 使用线段交互器
pub(crate) use simple_edge_set_intersector::SimpleEdgeSetIntersector; // 使用简单边集交互器
