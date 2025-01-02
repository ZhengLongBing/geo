use super::geomgraph::{Edge, EdgeEnd, EdgeIntersection};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

/// 计算从具有`edge_intersections`的[`Edge`]中产生的[`EdgeEnd`]，
/// 该Edge已经使用适当的自身和适当的[`EdgeIntersection`]进行了填充。
///
/// 基于[JTS的EdgeEndBuilder，截至版本1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/EdgeEndBuilder.java)
pub(crate) struct EdgeEndBuilder<F: GeoFloat> {
    _marker: std::marker::PhantomData<F>,
}

impl<F: GeoFloat> EdgeEndBuilder<F> {
    pub fn new() -> Self {
        EdgeEndBuilder {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn compute_ends_for_edges(&self, edges: &[Rc<RefCell<Edge<F>>>]) -> Vec<EdgeEnd<F>> {
        let mut list = vec![];
        for edge in edges {
            self.compute_ends_for_edge(&mut edge.borrow_mut(), &mut list);
        }
        list
    }

    /// 为[`Edge`]中的所有交点（如果有的话）创建存根边并将其插入到图的`list`中。
    fn compute_ends_for_edge(&self, edge: &mut Edge<F>, list: &mut Vec<EdgeEnd<F>>) {
        edge.add_edge_intersection_list_endpoints();

        let mut ei_iter = edge.edge_intersections().iter();
        let mut ei_prev;
        let mut ei_curr = None;
        let mut ei_next = ei_iter.next();
        if ei_next.is_none() {
            return;
        }

        loop {
            ei_prev = ei_curr;
            ei_curr = ei_next;
            ei_next = ei_iter.next();
            if let Some(ei_curr) = ei_curr {
                self.create_edge_end_for_prev(edge, list, ei_curr, ei_prev);
                self.create_edge_end_for_next(edge, list, ei_curr, ei_next);
            }

            if ei_curr.is_none() {
                break;
            }
        }
    }

    /// 为交点ei_curr之前的边添加`EdgeEnd`（如果有的话）到`list`中。
    ///
    /// 提供上一个交点以防它是存根边的端点。
    /// 否则，父边的上一个点将是端点。
    fn create_edge_end_for_prev(
        &self,
        edge: &Edge<F>,
        list: &mut Vec<EdgeEnd<F>>,
        ei_curr: &EdgeIntersection<F>,
        ei_prev: Option<&EdgeIntersection<F>>,
    ) {
        let mut i_prev = ei_curr.segment_index();
        if ei_curr.distance().is_zero() {
            // 如果在边的起始处，则没有上一个边
            if i_prev == 0 {
                return;
            }
            i_prev -= 1;
        }

        let mut coord_prev = edge.coords()[i_prev];
        // 如果上一个交点超过了上一个顶点，则使用它
        if let Some(ei_prev) = ei_prev {
            if ei_prev.segment_index() >= i_prev {
                coord_prev = ei_prev.coordinate();
            }
        }

        let mut label = edge.label().clone();
        // 由于edgeStub的方向与其父边相反，因此需翻转边标签的侧面
        label.flip();

        let edge_end = EdgeEnd::new(ei_curr.coordinate(), coord_prev, label);
        list.push(edge_end);
    }

    /// 为交点ei_curr之后的边添加`EdgeEnd`（如果有的话）到`list`中。
    ///
    /// 提供下一个交点以防它是存根边的端点。
    /// 否则，父边的下一个点将是端点。
    fn create_edge_end_for_next(
        &self,
        edge: &Edge<F>,
        list: &mut Vec<EdgeEnd<F>>,
        ei_curr: &EdgeIntersection<F>,
        ei_next: Option<&EdgeIntersection<F>>,
    ) {
        let i_next = ei_curr.segment_index() + 1;

        // 如果没有下一个边，则无事可做
        if i_next >= edge.coords().len() && ei_next.is_none() {
            return;
        }

        let mut coord_next = edge.coords()[i_next];

        // 如果下一个交点在当前段中，则使用它作为端点
        if let Some(ei_next) = ei_next {
            if ei_next.segment_index() == ei_curr.segment_index() {
                coord_next = ei_next.coordinate();
            }
        }

        let label = edge.label().clone();
        let edge_end = EdgeEnd::new(ei_curr.coordinate(), coord_next, label);
        list.push(edge_end);
    }
}
