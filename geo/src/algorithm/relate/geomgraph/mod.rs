#![allow(dead_code)] // 允许未使用的代码
#![allow(unused_imports)] // 允许未使用的导入

use std::fmt;

pub(crate) use edge::Edge; // 使用 edge 模块中的 Edge
pub(crate) use edge_end::{EdgeEnd, EdgeEndKey}; // 使用 edge_end 模块中的 EdgeEnd 和 EdgeEndKey
pub(crate) use edge_end_bundle::{EdgeEndBundle, LabeledEdgeEndBundle}; // 使用 edge_end_bundle 模块中的 EdgeEndBundle 和 LabeledEdgeEndBundle
pub(crate) use edge_end_bundle_star::{EdgeEndBundleStar, LabeledEdgeEndBundleStar}; // 使用 edge_end_bundle_star 模块中的 EdgeEndBundleStar 和 LabeledEdgeEndBundleStar
pub(crate) use edge_intersection::EdgeIntersection; // 使用 edge_intersection 模块中的 EdgeIntersection
pub use geometry_graph::GeometryGraph; // 使用 geometry_graph 模块中的 GeometryGraph
pub(crate) use intersection_matrix::IntersectionMatrix; // 使用 intersection_matrix 模块中的 IntersectionMatrix
pub(crate) use label::Label; // 使用 label 模块中的 Label
pub(crate) use line_intersector::{LineIntersection, LineIntersector}; // 使用 line_intersector 模块中的 LineIntersection 和 LineIntersector
pub(crate) use node::CoordNode; // 使用 node 模块中的 CoordNode
use planar_graph::PlanarGraph; // 使用 planar_graph 模块中的 PlanarGraph
pub(crate) use quadrant::Quadrant; // 使用 quadrant 模块中的 Quadrant
pub(crate) use robust_line_intersector::RobustLineIntersector; // 使用 robust_line_intersector 模块中的 RobustLineIntersector
use topology_position::TopologyPosition; // 使用 topology_position 模块中的 TopologyPosition

pub use crate::coordinate_position::CoordPos; // 使用 coordinate_position 模块中的 CoordPos
use crate::dimensions::Dimensions; // 使用 dimensions 模块中的 Dimensions

mod edge; // 声明 edge 子模块
mod edge_end; // 声明 edge_end 子模块
mod edge_end_bundle; // 声明 edge_end_bundle 子模块
mod edge_end_bundle_star; // 声明 edge_end_bundle_star 子模块
mod edge_intersection; // 声明 edge_intersection 子模块
mod geometry_graph; // 声明 geometry_graph 子模块
pub(crate) mod index; // 声明 index 子模块
mod label; // 声明 label 子模块
mod node; // 声明 node 子模块
pub(crate) mod node_map; // 声明 node_map 子模块
mod planar_graph; // 声明 planar_graph 子模块
mod quadrant; // 声明 quadrant 子模块
mod topology_position; // 声明 topology_position 子模块

pub(crate) mod intersection_matrix; // 声明 intersection_matrix 子模块
mod line_intersector; // 声明 line_intersector 子模块
mod robust_line_intersector; // 声明 robust_line_intersector 子模块

/// 相对于一个点的位置
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    On,    // 在点上方
    Left,  // 在点左侧
    Right, // 在点右侧
}
