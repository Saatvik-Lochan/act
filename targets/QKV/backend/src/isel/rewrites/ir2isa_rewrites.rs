use egg::{EGraph, Id};

use crate::ir::dtype::Dtype;
use crate::ir::egraph::{TensorInfo, TensorOp};

fn set_two_rhs_shapes(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    rhs_eclasses: &Vec<Id>,
    isa_idx: usize,
    alpha_idx: usize,
    info: TensorInfo,
) {
    egraph.set_analysis_data(rhs_eclasses[isa_idx], info.clone());
    egraph.set_analysis_data(rhs_eclasses[alpha_idx], info);
}

pub fn precond_load_rm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_load_rm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_load_rm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 4);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });
}

pub fn precond_load_cm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 5);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[4] != (TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_load_cm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 5);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0];

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_load_cm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 5);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0];

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, });
}

pub fn precond_store_rm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_store_rm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_store_rm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 4);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, });
}

pub fn precond_store_cm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 5);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0] / 128;

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![64, n, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[4] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_store_cm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 5);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0] / 128;

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_store_cm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 5);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[4].shape[0] / 128;

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, });
}

pub fn precond_mov(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_mov(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_mov(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 3);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });
}

pub fn precond_gemm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 5);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();

    for info in lhs_metadata.iter() {
        if *info != (TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, }) {
            return false;
        }
    }
    true
}

pub fn metadata_gemm(
    _egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 5);

    vec![None; 4]
}

pub fn set_shapes_gemm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 5);
    assert_eq!(rhs_eclasses.len(), 4);

    set_two_rhs_shapes(egraph, rhs_eclasses, 2, 3,
        TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, });
}

pub fn precond_softmax(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 9);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[8].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[4] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[5] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[6] != (TensorInfo { shape: vec![n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[7] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[8] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}

pub fn metadata_softmax(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 9);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[8].shape[0];

    vec![None, Some(n.to_string()), None]
}

pub fn set_shapes_softmax(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 9);
    assert_eq!(rhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[8].shape[0];

    set_two_rhs_shapes(egraph, rhs_eclasses, 1, 2,
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });
}

