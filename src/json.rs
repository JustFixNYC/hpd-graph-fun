use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

use super::hpd_graph::{HpdPetGraph, Node};
use super::hpd_registrations::HpdRegistrationMap;

// Note that petgraph supports Serde, but it only supports serializing
// entire graphs, not connected components, which is what we want, so
// I guess we'll have to roll our own here.

#[derive(serde::Serialize)]
pub struct JsonNode<'a> {
    id: usize,
    value: &'a Node,
}

#[derive(serde::Serialize)]
pub struct JsonEdge {
    from: usize,
    to: usize,
    reg_contacts: usize,
    is_bridge: bool,
    bbl: String,
}

#[derive(serde::Serialize)]
pub struct JsonGraph<'a> {
    title: String,
    nodes: Vec<JsonNode<'a>>,
    edges: Vec<JsonEdge>,
}

pub fn portfolio_json<'a>(
    title: String,
    nodes: &'a HashSet<NodeIndex<u32>>,
    petgraph: &'a HpdPetGraph,
    local_bridges: HashSet<EdgeIndex<u32>>,
    regs: &'a HpdRegistrationMap,
) -> JsonGraph<'a> {
    let mut edges_written = HashSet::new();
    let mut graph = JsonGraph {
        title,
        nodes: vec![],
        edges: vec![],
    };

    for node in nodes {
        graph.nodes.push(JsonNode {
            id: node.index(),
            value: petgraph.node_weight(*node).unwrap(),
        });
        for edge in petgraph.edges(*node) {
            let id = edge.id();
            if !edges_written.contains(&id) {
                edges_written.insert(id);
                let reg_infos = edge.weight();

                // All this unwrapping/indexing carries a lot of assumptions with
                // it, but it seems to hold, so we'll change it only if it ever
                // panics.
                let bbl = regs.get_by_id(reg_infos[0].id).unwrap()[0].bbl.to_string();

                graph.edges.push(JsonEdge {
                    from: edge.source().index(),
                    to: edge.target().index(),
                    reg_contacts: reg_infos.len(),
                    is_bridge: local_bridges.contains(&id),
                    bbl,
                });
            }
        }
    }

    graph
}
