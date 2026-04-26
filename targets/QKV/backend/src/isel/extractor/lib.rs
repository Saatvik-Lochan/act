use std::collections::HashSet;
use std::time::Instant;

use egg::{EGraph, Id};

use crate::ir::egraph::{TensorInfo, TensorOp};
use crate::ir::pii::PiiGraph;

use crate::isel::extractor::alpha::extract_alpha;

pub fn extract(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    root: Id,
    _inputs: &HashSet<Id>,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
    _limit: usize,
) -> Vec<PiiGraph> {
    let nodes = egraph.total_number_of_nodes();
    let start = Instant::now();

    let piis = extract_alpha(egraph, root, hbm_offsets);

    println!("Alpha Extractor over #nodes={}", nodes);
    println!("Number of PII graphs extracted: {}", piis.len());
    println!("Extraction time: {:?}", start.elapsed());
    println!();

    piis
}
