use egg::*;
use std::collections::{HashSet, VecDeque, HashMap};

use crate::ir::egraph::*;
use crate::ir::pii::PiiGraph;

/// Worklist algorithm for compile-time detection of constants.
/// Marks e-classes as const (updates analysis data), inserts a DetectedConst enode
/// carrying a pretty-printed representative op string, unions it into the class,
/// and returns the set of constant eclass ids.
/// Worklist algorithm for compile-time detection of constants.
pub fn detect_constants(
    egraph: &EGraph<TensorOp, TensorInfo>,
    inputs: &HashSet<Id>,
) -> HashSet<Id> {
    let mut worklist = VecDeque::new();
    let mut constants = HashSet::new();

    // initialize worklist
    for ec in egraph.classes() {
        worklist.push_back(ec.id);
    }

    while worklist.len() > 0 {
        let eclass = worklist.pop_front().unwrap();
        if constants.contains(&eclass) {
            continue;
        }

        for en in egraph[eclass].nodes.iter() {
            if en
                .children()
                .iter()
                .all(|child_ec| constants.contains(child_ec))
                && !inputs.contains(&eclass)
            {
                constants.insert(eclass);
                for mut parent_ec in egraph[eclass].parents() {
                    parent_ec = egraph.find(parent_ec);
                    worklist.push_back(parent_ec);
                }
            }
        }
    }

    constants
}

/// Annotate the egraph for the given constant e-classes.
/// For each eclass in `constants` this sets the `is_const` flag in the analysis
/// data and inserts a `TensorOp::DetectedConst(repr)` enode where `repr` is a
/// human-friendly operator-only representative (if available).
pub fn annotate_constants(egraph: &mut EGraph<TensorOp, TensorInfo>, constants: &HashSet<Id>) {
    for eclass in constants.iter() {
        let mut new_metadata = egraph[*eclass].data.clone();
        new_metadata.is_const = true;
        egraph.set_analysis_data(*eclass, new_metadata);

        let mut visited: HashSet<Id> = HashSet::new();
        let repr = match op_repr(egraph, *eclass, &mut visited) {
            Some(s) => s,
            None => panic!(
                "annotate_constants: no operator-only representative found for constant eclass {}",
                eclass
            ),
        };
        let const_ec = egraph.add(TensorOp::DetectedConst(repr));
        egraph.union(*eclass, const_ec);
    }
}

// Try to produce a string representation for an eclass whose expression is composed
// purely of Op* operators. Returns None if no such representative can be found.
fn op_repr(
    egraph: &EGraph<TensorOp, TensorInfo>,
    eclass: Id,
    visited: &mut HashSet<Id>,
) -> Option<String> {
    if visited.contains(&eclass) {
        return None;
    }
    visited.insert(eclass);

    for en in egraph[eclass].nodes.iter() {
        if let Some(s) = op_repr_en(egraph, en, visited) {
            visited.remove(&eclass);
            return Some(s);
        }
    }

    visited.remove(&eclass);
    None
}

// Build Op*-only representation for a single enode.
fn op_repr_en(
    egraph: &EGraph<TensorOp, TensorInfo>,
    en: &TensorOp,
    visited: &mut HashSet<Id>,
) -> Option<String> {
    // Only accept operator-only enodes (Op*). Reject ISA instructions, Var, DetectedConst, etc.
    let label = match en {
        TensorOp::OpAdd(_)
        | TensorOp::OpBitcvt(_)
        | TensorOp::OpBroadcast(_, _)
        | TensorOp::OpConcat(_, _)
        | TensorOp::OpConstant(_)
        | TensorOp::OpConvert(_, _)
        | TensorOp::OpDivide(_)
        | TensorOp::OpDot(_)
        | TensorOp::OpExp(_)
        | TensorOp::OpEye(_)
        | TensorOp::OpOr(_)
        | TensorOp::OpReduceSum(_, _)
        | TensorOp::OpReshape(_, _)
        | TensorOp::OpShiftLeft(_)
        | TensorOp::OpShiftRightLogical(_)
        | TensorOp::OpSlice(_, _)
        | TensorOp::OpXor(_)
        | TensorOp::OpTranspose(_, _)
         => format!("{}", en),
        _ => return None,
    };
    let children_ids = en.children();
    if children_ids.is_empty() {
        return Some(format!("{}()", label));
    }

    let mut parts: Vec<String> = Vec::new();
    for &cid in children_ids.iter() {
        if let Some(s) = op_repr(egraph, cid, visited) {
            parts.push(s);
        } else {
            return None;
        }
    }

    Some(format!("{}({})", label, parts.join(", ")))
}

pub fn get_hbm_offset(hbm_offsets: &Vec<(Option<Id>, i32)>, eclass: Id) -> Option<i32> {
    for (buf_ec, offset) in hbm_offsets.iter() {
        if buf_ec.is_some() && buf_ec.unwrap() == eclass {
            return Some(*offset);
        }
    }
    None
}

pub fn recexpr_to_pii(
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
