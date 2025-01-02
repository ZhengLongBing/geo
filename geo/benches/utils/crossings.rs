#![allow(dead_code)]

use std::iter::FromIterator;

use geo::algorithm::sweep::Intersections;
use geo::{line_intersection::line_intersection, Line};

use rstar::{primitives::GeomWithData, RTree};

/// 使用优化算法计算直线交叉的数量
pub fn count_bo(lines: &[Line<f64>]) -> usize {
    // 使用Intersections来从迭代器中计算交互数量
    Intersections::from_iter(lines.iter()).count()
}

/// 使用暴力算法计算直线交叉的数量
pub fn count_brute(lines: &[Line<f64>]) -> usize {
    let mut count = 0; // 初始化交叉计数
    let n = lines.len(); // 获得线段数组的长度
    for i in 0..n {
        let l1 = &lines[i]; // 选择当前线段
        for l2 in lines.iter().take(n).skip(i + 1) {
            // 比较剩余线段
            if line_intersection(*l1, *l2).is_some() {
                // 如果有交点
                count += 1; // 增加计数
            }
        }
    }
    count
}

/// 使用RTree计算直线交叉的数量
pub fn count_rtree(lines: &[Line<f64>]) -> usize {
    let lines: Vec<_> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| GeomWithData::new(*l, i)) // 为每条线添加索引信息
        .collect();

    let tree = RTree::bulk_load(lines); // 使用bulk_load创建RTree
    tree.intersection_candidates_with_other_tree(&tree) // 找到交叉候选
        .filter_map(|(l1, l2)| {
            if l1.data >= l2.data {
                // 避免重复计算
                None
            } else {
                line_intersection(*l1.geom(), *l2.geom()) // 计算交点
            }
        })
        .count() // 计算交点的数量
}
