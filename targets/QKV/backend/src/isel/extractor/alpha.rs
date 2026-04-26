use std::collections::HashMap;

use egg::{CostFunction, EGraph, Extractor, Id, Language, RecExpr};

use crate::ir::egraph::{TensorInfo, TensorOp};
use crate::ir::pii::PiiGraph;
use crate::isel::extractor::utils::get_hbm_offset;

#[derive(Default)]
struct UnitCost;

impl CostFunction<TensorOp> for UnitCost {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &TensorOp, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        1 + enode.children().iter().map(|id| costs(*id)).sum::<usize>()
    }
}

pub fn extract_alpha(
    egraph: &EGraph<TensorOp, TensorInfo>,
    root: Id,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
) -> Vec<PiiGraph> {
    let root = egraph.find(root);
    let mut alpha_roots: Vec<Id> = vec![];

    for en in &egraph[root].nodes {
        if let TensorOp::AlphaHBM(child) = en {
            alpha_roots.push(egraph.find(*child));
        }
    }

    if alpha_roots.len() > 1 {
        panic!(
            "alpha extraction invariant violation: root eclass {:?} contains multiple AlphaHBM nodes after alpha injectivity enforcement",
            root
        );
    }

    let Some(isa_root) = alpha_roots.first().copied() else {
        println!(
            "Alpha extractor: root eclass {:?} does not contain AlphaHBM; no PII graph extracted.",
            root
        );
        return vec![];
    };

    let extractor = Extractor::new(egraph, UnitCost::default());
    let (_cost, expr) = extractor.find_best(isa_root);
    vec![recexpr_to_pii(egraph, &expr, hbm_offsets)]
}

fn recexpr_to_pii(
    egraph: &EGraph<TensorOp, TensorInfo>,
    expr: &RecExpr<TensorOp>,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
) -> PiiGraph {
    let mut pii = PiiGraph::default();
    let mut expr_to_pii: HashMap<Id, usize> = HashMap::new();
    let mut expr_to_egraph: HashMap<Id, Id> = HashMap::new();

    for (idx, enode) in expr.as_ref().iter().enumerate() {
        let expr_id = Id::from(idx);

        let mapped_enode = enode.clone().map_children(|child_expr_id| {
            *expr_to_egraph
                .get(&child_expr_id)
                .expect("extracted expression is not in topological order")
        });

        let eclass = egraph
            .lookup(mapped_enode.clone())
            .unwrap_or_else(|| panic!("could not recover eclass for extracted enode: {:?}", mapped_enode));
        let eclass = egraph.find(eclass);
        expr_to_egraph.insert(expr_id, eclass);

        let children: Vec<usize> = enode
            .children()
            .iter()
            .map(|child_expr_id| {
                *expr_to_pii
                    .get(child_expr_id)
                    .expect("child PII node missing during RecExpr conversion")
            })
            .collect();

        let pii_id = pii.add_node(
            mapped_enode,
            egraph[eclass].data.clone(),
            children,
            get_hbm_offset(hbm_offsets, eclass),
        );
        expr_to_pii.insert(expr_id, pii_id);
    }

    pii
}
