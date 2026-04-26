use egg::{EGraph, Id};

use crate::ir::egraph::{TensorInfo, TensorOp};

// TODO(alpha_inv): Remove this hook if/when alpha injectivity is encoded directly
// in the rewrite system with alpha_inv/projection operators. This hook is a
// temporary generated-artifact mechanism for enforcing that AlphaD is bijective
// on values without introducing inverse/projection nodes into ISA e-classes.
pub fn enforce_alpha_injectivity(egraph: &mut EGraph<TensorOp, TensorInfo>) -> bool {
    let mut unions: Vec<(Id, Id)> = vec![];

    for class in egraph.classes() {
        let mut hbm_children: Vec<Id> = vec![];
        let mut d1_children: Vec<Id> = vec![];
        let mut d2_children: Vec<Id> = vec![];

        for node in &class.nodes {
            match node {
                TensorOp::AlphaHBM(child) => hbm_children.push(egraph.find(*child)),
                TensorOp::AlphaD1(child) => d1_children.push(egraph.find(*child)),
                TensorOp::AlphaD2(child) => d2_children.push(egraph.find(*child)),
                _ => {}
            }
        }

        add_child_unions(&mut unions, &hbm_children);
        add_child_unions(&mut unions, &d1_children);
        add_child_unions(&mut unions, &d2_children);
    }

    let mut changed = false;
    for (a, b) in unions {
        changed |= egraph.union(a, b);
    }

    if changed {
        egraph.rebuild();
    }

    changed
}

fn add_child_unions(unions: &mut Vec<(Id, Id)>, children: &[Id]) {
    let Some((&first, rest)) = children.split_first() else {
        return;
    };

    for &child in rest {
        if child != first {
            unions.push((first, child));
        }
    }
}
