use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::fmt::Display;

use egg::*;
use serde_json::Value;

use crate::ir::egraph::{TensorInfo, TensorOp};
use crate::ir::pii::PiiGraph;
use crate::isel::extractor::utils::get_hbm_offset;

const SMOOTHE_DIR: &str = "/workspace/smoothe";
const SMOOTHE_INPUT: &str = "/workspace/egraph.json";

pub fn smoothe_extract(
    egraph: &mut EGraph<TensorOp, TensorInfo>,
    root: Id,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
) -> PiiGraph {
    let root = egraph.find(root);
    let output_dir = Path::new(SMOOTHE_DIR);
    let input_path = output_dir.join(SMOOTHE_INPUT);

    // Serialize egraph in egraph-serialize / extraction-gym format.
    let mut serialized_egraph = egg_to_serialized_egraph(egraph);
    serialized_egraph.root_eclasses = vec![egraph_serialize::ClassId::from(root.to_string())];
    serialized_egraph
        .to_json_file(&input_path)
        .expect("failed to write serialized egraph");

    // Run SmoothE. `src.train` writes `<output_dir>/smoothe_log/<input_stem>_smoothe.json`.
    let status = Command::new("conda")
        .arg("run")
        .arg("--no-capture-output")
        .arg("-n")
        .arg("smoothe")
        .arg("python")
        .arg("-m")
        .arg("src.train")
        .arg("--input_file")
        .arg(&input_path)
        .arg("--acyclic")
        .current_dir(SMOOTHE_DIR)
        .env("PYTHONPATH", ".")
        .status()
        .expect("failed to run SmoothE");

    assert!(status.success(), "SmoothE failed");

    let solution_path = output_dir.join("logs").join("smoothe_log").join("egraph_smoothe.json");
    let selected = read_smoothe_solution(&solution_path);

    selected_enodes_to_pii(egraph, root, &selected, hbm_offsets)
}

fn read_smoothe_solution(path: &Path) -> Vec<String> {
    let data = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read SmoothE solution {:?}: {}", path, err));
    let json: Value = serde_json::from_str(&data)
        .unwrap_or_else(|err| panic!("failed to parse SmoothE solution {:?}: {}", path, err));

    json.get("solution")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("SmoothE solution {:?} does not contain array field `solution`", path))
        .iter()
        .map(|v| {
            v.as_str()
                .unwrap_or_else(|| panic!("SmoothE solution node id is not a string: {:?}", v))
                .to_string()
        })
        .collect()
}

fn selected_enodes_to_pii(
    egraph: &EGraph<TensorOp, TensorInfo>,
    root: Id,
    selected_node_ids: &[String],
    hbm_offsets: &Vec<(Option<Id>, i32)>,
) -> PiiGraph {
    let choices = selected_node_ids
        .iter()
        .map(|node_id| parse_selected_node_id(egraph, node_id))
        .collect::<HashMap<Id, usize>>();

    let mut pii = PiiGraph::default();
    let mut eclass_to_pii: HashMap<Id, usize> = HashMap::new();
    let mut visiting: HashSet<Id> = HashSet::new();

    build_selected_pii_node(
        egraph,
        egraph.find(root),
        &choices,
        hbm_offsets,
        &mut pii,
        &mut eclass_to_pii,
        &mut visiting,
    );

    pii
}

fn build_selected_pii_node(
    egraph: &EGraph<TensorOp, TensorInfo>,
    eclass: Id,
    choices: &HashMap<Id, usize>,
    hbm_offsets: &Vec<(Option<Id>, i32)>,
    pii: &mut PiiGraph,
    eclass_to_pii: &mut HashMap<Id, usize>,
    visiting: &mut HashSet<Id>,
) -> usize {
    let eclass = egraph.find(eclass);
    if let Some(&pii_id) = eclass_to_pii.get(&eclass) {
        return pii_id;
    }
    if !visiting.insert(eclass) {
        panic!("SmoothE selected cyclic extraction involving eclass {:?}", eclass);
    }

    let node_idx = *choices
        .get(&eclass)
        .unwrap_or_else(|| panic!("SmoothE did not select an enode for reachable eclass {:?}", eclass));
    let op = egraph[eclass]
        .nodes
        .get(node_idx)
        .unwrap_or_else(|| panic!("selected node index {} out of bounds for eclass {:?}", node_idx, eclass))
        .clone();

    let children = op
        .children()
        .iter()
        .map(|child| {
            build_selected_pii_node(
                egraph,
                egraph.find(*child),
                choices,
                hbm_offsets,
                pii,
                eclass_to_pii,
                visiting,
            )
        })
        .collect::<Vec<_>>();

    let pii_id = pii.add_node(
        op,
        egraph[eclass].data.clone(),
        children,
        get_hbm_offset(hbm_offsets, eclass),
    );
    eclass_to_pii.insert(eclass, pii_id);
    visiting.remove(&eclass);

    pii_id
}

fn parse_selected_node_id(
    egraph: &EGraph<TensorOp, TensorInfo>,
    node_id: &str,
) -> (Id, usize) {
    let (class, node_idx) = node_id
        .rsplit_once('.')
        .unwrap_or_else(|| panic!("SmoothE selected node id is not `<eclass>.<index>`: {}", node_id));
    let class = class
        .parse::<usize>()
        .unwrap_or_else(|err| panic!("invalid selected eclass id `{}` in `{}`: {}", class, node_id, err));
    let node_idx = node_idx
        .parse::<usize>()
        .unwrap_or_else(|err| panic!("invalid selected node index in `{}`: {}", node_id, err));
    let eclass = egraph.find(Id::from(class));

    if !egraph[eclass].nodes.get(node_idx).is_some() {
        panic!("SmoothE selected node `{}` does not exist in egraph", node_id);
    }

    (eclass, node_idx)
}

// Copied from https://github.com/egraphs-good/egraph-serialize
pub fn egg_to_serialized_egraph<L, A>(egraph: &EGraph<L, A>) -> egraph_serialize::EGraph
where
    L: Language + Display,
    A: Analysis<L>,
{
    use egraph_serialize::*;
    let mut out = EGraph::default();
    for class in egraph.classes() {
        for (i, node) in class.nodes.iter().enumerate() {
            out.add_node(
                format!("{}.{}", class.id, i),
                Node {
                    op: node.to_string(),
                    children: node
                        .children()
                        .iter()
                        .map(|id| NodeId::from(format!("{}.0", egraph.find(*id))))
                        .collect(),
                    eclass: ClassId::from(format!("{}", class.id)),
                    cost: Cost::new(1.0).unwrap(),
                    subsumed: false,
                },
            )
        }
    }
    out
}
