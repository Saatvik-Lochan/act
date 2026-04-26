use std::collections::HashMap;

use egg::{CostFunction, EGraph, Extractor, Id, Language, RecExpr};

use crate::ir::egraph::{TensorInfo, TensorOp};
use crate::ir::pii::PiiGraph;
use crate::isel::extractor::utils::{get_hbm_offset, recexpr_to_pii};

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

