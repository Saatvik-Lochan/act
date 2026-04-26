use egg::{EGraph, Id};

use crate::ir::dtype::Dtype;
use crate::ir::egraph::{TensorInfo, TensorOp};

pub fn precond_load_rm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_load_rm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_load_rm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 3);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0];


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });

}


pub fn precond_load_cm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_load_cm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_load_cm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 4);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0];


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, });

}


pub fn precond_store_rm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0] / 128;

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_store_rm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0] / 128;

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_store_rm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 3);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[2].shape[0] / 128;


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, });

}


pub fn precond_store_cm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![64, n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![64, n, 2], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    if lhs_metadata[3] != (TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_store_cm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 4);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_store_cm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 4);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[3].shape[0] / 128;


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![n*128], dtype: Dtype::U8, is_const: false, });

}


pub fn precond_mov(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[1].shape[0];

    if lhs_metadata[0] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_mov(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[1].shape[0];

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_mov(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 2);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[1].shape[0];


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });

}


pub fn precond_gemm(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 3);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();

    if lhs_metadata[0] != (TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[1] != (TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[2] != (TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_gemm(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 3);
    let _lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();

    let rhs_metadata = vec![None; 3];
    rhs_metadata
}


pub fn set_shapes_gemm(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 3);
    assert_eq!(rhs_eclasses.len(), 3);
    let _lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![64, 64], dtype: Dtype::BF16, is_const: false, });

}


pub fn precond_softmax(egraph: &EGraph<TensorOp, TensorInfo>, lhs_eclasses: &Vec<Id>) -> bool {
    assert_eq!(lhs_eclasses.len(), 7);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[6].shape[0];

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
    if lhs_metadata[4] != (TensorInfo { shape: vec![n], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[5] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    if lhs_metadata[6] != (TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, }) {
        return false;
    }
    true
}


pub fn metadata_softmax(
    egraph: &EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    _lhs_enodes: &Vec<Option<TensorOp>>,
) -> Vec<Option<String>> {
    assert_eq!(lhs_eclasses.len(), 7);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[6].shape[0];

    let mut rhs_metadata = vec![None; 2];
    *rhs_metadata.last_mut().unwrap() = Some(n.to_string());
    rhs_metadata
}


pub fn set_shapes_softmax(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    lhs_eclasses: &Vec<Id>,
    rhs_eclasses: &Vec<Id>,
) {
    assert_eq!(lhs_eclasses.len(), 7);
    assert_eq!(rhs_eclasses.len(), 2);
    let lhs_metadata: Vec<TensorInfo> = lhs_eclasses.iter().map(|id| egraph[*id].data.clone()).collect();
    let n = lhs_metadata[6].shape[0];


    egraph.set_analysis_data(*rhs_eclasses.last().unwrap(),
        TensorInfo { shape: vec![n, 64], dtype: Dtype::BF16, is_const: false, });

}



