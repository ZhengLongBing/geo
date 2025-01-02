use super::{CoordNode, CoordPos, EdgeEnd};
use crate::{Coord, GeoFloat};

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

/// 节点的映射，根据节点的坐标进行索引
#[derive(Clone, PartialEq)]
pub(crate) struct NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    map: BTreeMap<NodeKey<F>, NF::Node>,
    _node_factory: PhantomData<NF>,
}

/// 创建存储在 `NodeMap` 中的节点
pub(crate) trait NodeFactory<F: GeoFloat>: PartialEq {
    type Node;
    fn create_node(coordinate: Coord<F>) -> Self::Node;
}

impl<F, NF> fmt::Debug for NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMap")
            .field("map.len()", &self.map.len())
            .finish()
    }
}

#[derive(Clone)]
struct NodeKey<F: GeoFloat>(Coord<F>);

impl<F: GeoFloat> std::cmp::Ord for NodeKey<F> {
    fn cmp(&self, other: &NodeKey<F>) -> std::cmp::Ordering {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        crate::utils::lex_cmp(&self.0, &other.0)
    }
}

impl<F: GeoFloat> std::cmp::PartialOrd for NodeKey<F> {
    fn partial_cmp(&self, other: &NodeKey<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: GeoFloat> std::cmp::PartialEq for NodeKey<F> {
    fn eq(&self, other: &NodeKey<F>) -> bool {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        self.0 == other.0
    }
}

impl<F: GeoFloat> std::cmp::Eq for NodeKey<F> {}

impl<F, NF> NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    pub fn new() -> Self {
        NodeMap {
            map: BTreeMap::new(),
            _node_factory: PhantomData,
        }
    }
    /// 添加具有给定 `Coord` 的 `NF::Node`。
    ///
    /// 注意：坐标必须是非 NaN 的。
    pub fn insert_node_with_coordinate(&mut self, coord: Coord<F>) -> &mut NF::Node {
        debug_assert!(!coord.x.is_nan() && !coord.y.is_nan(), "不支持 NaN 坐标");
        let key = NodeKey(coord);
        self.map
            .entry(key)
            .or_insert_with(|| NF::create_node(coord))
    }

    /// 返回匹配 `coord` 的 `NF::Node`，如果有的话
    pub fn find(&self, coord: Coord<F>) -> Option<&NF::Node> {
        self.map.get(&NodeKey(coord))
    }

    /// 按照 `Coord` 的词典顺序迭代 `NF::Node`
    pub fn iter(&self) -> impl Iterator<Item = &NF::Node> {
        self.map.values()
    }

    /// 按照 `Coord` 的词典顺序迭代 `NF::Node`
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NF::Node> {
        self.map.values_mut()
    }

    /// 按照 `Coord` 的词典顺序迭代 `NF::Node`
    pub fn into_iter(self) -> impl Iterator<Item = NF::Node> {
        self.map.into_values()
    }
}
