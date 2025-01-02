use super::{
    node_map::{NodeFactory, NodeMap}, // 使用 node_map 模块中的 NodeFactory 和 NodeMap
    CoordNode,
    CoordPos,
    Edge,
    Label, // 使用上级模块中的 CoordNode, CoordPos, Edge, Label
};
use crate::{Coord, GeoFloat}; // 使用 crate 内的坐标和 GeoFloat 相关内容

use std::cell::RefCell; // 使用 RefCell 实现内部可变性
use std::rc::Rc; // 使用 Rc 实现引用计数

#[derive(Clone, PartialEq)]
pub(crate) struct PlanarGraphNode; // 平面图节点

/// 基本节点构造器不允许有关联边
impl<F> NodeFactory<F> for PlanarGraphNode
where
    F: GeoFloat,
{
    type Node = CoordNode<F>; // 节点类型定义为 CoordNode
    fn create_node(coordinate: Coord<F>) -> Self::Node {
        // 使用给定坐标创建节点
        CoordNode::new(coordinate)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct PlanarGraph<F: GeoFloat> {
    pub(crate) nodes: NodeMap<F, PlanarGraphNode>, // 节点映射
    edges: Vec<Rc<RefCell<Edge<F>>>>,              // 使用 Rc 和 RefCell 包裹边进行管理
}

impl<F: GeoFloat> PlanarGraph<F> {
    pub fn clone_for_arg_index(&self, from_arg_index: usize, to_arg_index: usize) -> Self {
        let mut graph = Self {
            nodes: self.nodes.clone(), // 克隆节点映射
            edges: self
                .edges
                .iter()
                .map(|e| Rc::new(RefCell::new(e.borrow().clone()))) // 深拷贝边
                .collect(),
        };
        assert_eq!(from_arg_index, 0); // 确保 from_arg_index 是 0
        if from_arg_index != to_arg_index {
            graph.swap_labels(); // 如果来源和目标索引不同，交换标签
        }
        graph
    }

    fn swap_labels(&mut self) {
        for node in self.nodes.iter_mut() {
            node.swap_label_args(); // 交换节点标签参数
        }
        for edge in &mut self.edges {
            edge.borrow_mut().swap_label_args(); // 交换边的标签参数
        }
    }

    pub fn assert_eq_graph(&self, other: &Self) {
        assert_eq!(self.nodes, other.nodes); // 确认节点映射相等
        assert_eq!(self.edges, other.edges); // 确认边向量相等
    }

    pub fn edges(&self) -> &[Rc<RefCell<Edge<F>>>] {
        &self.edges // 返回边的引用
    }

    pub fn new() -> Self {
        PlanarGraph {
            nodes: NodeMap::new(), // 创建新的节点映射
            edges: vec![],         // 初始化空的边向量
        }
    }

    pub fn is_boundary_node(&self, geom_index: usize, coord: Coord<F>) -> bool {
        // 判断给定坐标的节点是否是边界节点
        self.nodes
            .find(coord)
            .and_then(|node| node.label().on_position(geom_index))
            .map(|position| position == CoordPos::OnBoundary)
            .unwrap_or(false)
    }

    pub fn insert_edge(&mut self, edge: Edge<F>) {
        self.edges.push(Rc::new(RefCell::new(edge))); // 插入边
    }

    pub fn add_node_with_coordinate(&mut self, coord: Coord<F>) -> &mut CoordNode<F> {
        // 添加坐标
        self.nodes.insert_node_with_coordinate(coord)
    }

    pub fn boundary_nodes(&self, geom_index: usize) -> impl Iterator<Item = &CoordNode<F>> {
        // 迭代返回边界节点
        self.nodes.iter().filter(move |node| {
            matches!(
                node.label().on_position(geom_index),
                Some(CoordPos::OnBoundary)
            )
        })
    }
}
