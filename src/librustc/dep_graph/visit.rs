// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use hir;
use hir::def_id::DefId;
use hir::intravisit::Visitor;
use ty::TyCtxt;

use super::dep_node::DepNode;


/// Visit all the items in the krate in some order. When visiting a
/// particular item, first create a dep-node by calling `dep_node_fn`
/// and push that onto the dep-graph stack of tasks, and also create a
/// read edge from the corresponding AST node. This is used in
/// compiler passes to automatically record the item that they are
/// working on.
pub fn visit_all_items_in_krate<'tcx,V,F>(tcx: &TyCtxt<'tcx>,
                                          mut dep_node_fn: F,
                                          visitor: &mut V)
    where F: FnMut(DefId) -> DepNode<DefId>, V: Visitor<'tcx>
{
    struct TrackingVisitor<'visit, 'tcx: 'visit, F: 'visit, V: 'visit> {
        tcx: &'visit TyCtxt<'tcx>,
        dep_node_fn: &'visit mut F,
        visitor: &'visit mut V
    }

    impl<'visit, 'tcx, F, V> Visitor<'tcx> for TrackingVisitor<'visit, 'tcx, F, V>
        where F: FnMut(DefId) -> DepNode<DefId>, V: Visitor<'tcx>
    {
        fn visit_item(&mut self, i: &'tcx hir::Item) {
            let item_def_id = self.tcx.map.local_def_id(i.id);
            let task_id = (self.dep_node_fn)(item_def_id);
            let _task = self.tcx.dep_graph.in_task(task_id);
            debug!("Started task {:?}", task_id);
            self.tcx.dep_graph.read(DepNode::Hir(item_def_id));
            self.visitor.visit_item(i)
        }
    }

    let krate = tcx.dep_graph.with_ignore(|| tcx.map.krate());
    let mut tracking_visitor = TrackingVisitor {
        tcx: tcx,
        dep_node_fn: &mut dep_node_fn,
        visitor: visitor
    };
    krate.visit_all_items(&mut tracking_visitor)
}
