use std::collections::HashSet;
use std::time::Instant;

use egg::{EGraph, Id};

use crate::ir::egraph::{TensorInfo, TensorOp};
use crate::ir::pii::PiiGraph;

use crate::isel::extractor::smoothe::smoothe_extract;

pub fn extract(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    root: Id,
    _inputs: &HashSet<Id>,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
    _limit: usize,
) -> Vec<PiiGraph> {
    let nodes = egraph.total_number_of_nodes();
    let start = Instant::now();

    let root = egraph.find(root);
    let mut alpha_roots: Vec<Id> = vec![];

    for en in &egraph[root].nodes {
        if let TensorOp::AlphaHBM(child) = en {
            alpha_roots.push(egraph.find(*child));
        }
    }

    if alpha_roots.len() > 1 {
        panic!(
            "SmoothE extractor invariant violation: root eclass {:?} contains multiple AlphaHBM nodes after alpha injectivity enforcement",
            root
        );
    }

    let Some(isa_root) = alpha_roots.first().copied() else {
        println!(
            "SmoothE extractor: root eclass {:?} does not contain AlphaHBM; no PII graph extracted.",
            root
        );
        return vec![];
    };

    let piis = vec![smoothe_extract(egraph, isa_root, hbm_offsets)];

    println!("SmoothE Extractor over #nodes={}", nodes);
    println!("Number of PII graphs extracted: {}", piis.len());
    println!("Extraction time: {:?}", start.elapsed());
    println!();

    piis
}
