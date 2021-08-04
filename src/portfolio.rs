use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef, VisitMap};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

use super::hpd_graph::{HpdPetGraph, Node, RegInfo};
use super::hpd_registrations::HpdRegistrationMap;
use super::ranking::rank_tuples;

pub struct Portfolio {
    graph: Rc<HpdPetGraph>,
    nodes: HashSet<NodeIndex<u32>>,
    cached_name: RefCell<Option<Rc<String>>>,
}

impl Portfolio {
    fn new(nodes: HashSet<NodeIndex<u32>>, graph: Rc<HpdPetGraph>) -> Self {
        Portfolio {
            graph,
            nodes,
            cached_name: RefCell::new(None),
        }
    }

    fn get_hpd_reg_contact_count(&self, node: &NodeIndex<u32>) -> usize {
        let mut total = 0;
        for edge in self.graph.edges(*node) {
            let reg_info = edge.weight();
            total += reg_info.len();
        }
        total
    }

    pub fn rank_bizaddrs(&self) -> Vec<(Rc<String>, usize)> {
        let mut result = vec![];

        for node in self.nodes.iter() {
            if let Node::BizAddr(name) = self.graph.node_weight(*node).unwrap() {
                result.push((Rc::clone(name), self.get_hpd_reg_contact_count(node)));
            }
        }

        rank_tuples(&mut result);
        result
    }

    pub fn rank_names(&self) -> Vec<(Rc<String>, usize)> {
        let mut result = vec![];

        for node in self.nodes.iter() {
            if let Node::Name(name) = self.graph.node_weight(*node).unwrap() {
                result.push((Rc::clone(name), self.get_hpd_reg_contact_count(node)));
            }
        }

        rank_tuples(&mut result);
        result
    }

    pub fn name(&self) -> Rc<String> {
        if self.cached_name.borrow().is_none() {
            let mut option = self.cached_name.borrow_mut();
            let name = format!(
                "{}'s portfolio",
                self.get_best_name().unwrap_or("???".to_owned())
            );
            option.replace(Rc::new(name));
        }

        let option = self.cached_name.borrow();
        Rc::clone(option.as_ref().unwrap())
    }

    fn get_best_name(&self) -> Option<String> {
        let mut best: Option<(NodeIndex<u32>, usize)> = None;
        for node in self.nodes.iter() {
            if let Node::Name(_) = self.graph.node_weight(*node).unwrap() {
                let count = self.get_hpd_reg_contact_count(node);
                let is_new_best = if let Some((_, current_count)) = best {
                    current_count < count
                } else {
                    true
                };
                if is_new_best {
                    best = Some((*node, count));
                }
            }
        }
        if let Some((node_idx, _)) = best {
            let node = self.graph.node_weight(node_idx).unwrap();
            Some(node.to_str().to_owned())
        } else {
            None
        }
    }

    pub fn building_count(&self, regs: &HpdRegistrationMap) -> usize {
        let mut bins = HashSet::<u32>::new();
        for node in self.nodes.iter() {
            if let Node::Name(_) = self.graph.node_weight(*node).unwrap() {
                for edge in self.graph.edges(*node) {
                    for reg_info in edge.weight() {
                        for reg in regs.get_by_id(reg_info.id).unwrap() {
                            bins.insert(reg.reg_id);
                        }
                    }
                }
            }
        }
        bins.len()
    }

    pub fn json(&self) -> String {
        // Note that petgraph supports Serde, but it only supports serializing
        // entire graphs, not connected components, which is what we want, so
        // I guess we'll have to roll our own here.

        #[derive(serde::Serialize)]
        struct JsonNode<'a> {
            id: usize,
            value: &'a Node,
        }

        #[derive(serde::Serialize)]
        struct JsonEdge {
            from: usize,
            to: usize,
            reg_contacts: usize,
        }

        #[derive(serde::Serialize)]
        struct JsonGraph<'a> {
            title: String,
            nodes: Vec<JsonNode<'a>>,
            edges: Vec<JsonEdge>,
        }

        let mut edges_written = HashSet::new();
        let mut graph = JsonGraph {
            title: self.name().to_string(),
            nodes: vec![],
            edges: vec![],
        };

        for node in &self.nodes {
            graph.nodes.push(JsonNode {
                id: node.index(),
                value: self.graph.node_weight(*node).unwrap(),
            });
            for edge in self.graph.edges(*node) {
                let id = edge.id();
                if !edges_written.contains(&id) {
                    edges_written.insert(id);
                    graph.edges.push(JsonEdge {
                        from: edge.source().index(),
                        to: edge.target().index(),
                        reg_contacts: edge.weight().len(),
                    });
                }
            }
        }

        serde_json::to_string(&graph).unwrap()
    }

    pub fn dot_graph(&self) -> String {
        let g = self.graph.deref();
        let gf = petgraph::visit::NodeFiltered::from_fn(&g, |g| self.nodes.is_visited(&g));
        let bridges: HashSet<EdgeIndex<u32>> = self
            .find_local_bridges()
            .into_iter()
            .map(|(n1, n2)| self.graph.find_edge(n1, n2).unwrap())
            .collect();
        let get_edge_str = |_, edge: petgraph::graph::EdgeReference<Vec<RegInfo>>| {
            let is_bridge = bridges.contains(&edge.id());
            let color = if is_bridge { "red" } else { "black" };
            format!("label=\" {}\" color={}", edge.weight().len(), color)
        };

        let d = Dot::with_attr_getters(
            &gf,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &get_edge_str,
            &|_, (_, node)| match node {
                Node::BizAddr(addr) => {
                    format!(
                        "label=\"{}\", color=lightblue2, style=filled, shape=box",
                        addr.to_lowercase()
                    )
                }
                Node::Name(name) => format!(
                    "label=\"{}\", color=whitesmoke, style=filled",
                    name.to_lowercase()
                ),
            },
        );

        format!("// {}\n\n{:?}", self.name(), d)
    }

    pub fn find_local_bridges(&self) -> Vec<(NodeIndex<u32>, NodeIndex<u32>)> {
        if let Some(node) = self.nodes.iter().next() {
            let lbf = super::local_bridge::LocalBridgeFinder::new(&self.graph, *node);
            let bridges = lbf
                .find_local_bridges()
                .into_iter()
                .filter(|(n1, n2)| {
                    // Ignore any bridges that, if removed, would orphan a single node.
                    self.graph.neighbors(*n1).count() > 1 && self.graph.neighbors(*n2).count() > 1
                })
                .collect();
            bridges
        } else {
            vec![]
        }
    }
}

pub struct PortfolioMap {
    portfolios: Vec<Rc<Portfolio>>,
    node_portfolios: HashMap<NodeIndex<u32>, usize>,
}

impl PortfolioMap {
    pub fn from_graph(graph: Rc<HpdPetGraph>) -> Self {
        let mut visited = HashSet::with_capacity(graph.node_count());
        let mut portfolios = vec![];
        let mut node_portfolios = HashMap::new();
        let mut portfolio_idx = 0;

        for start in graph.node_indices() {
            if visited.is_visited(&start) {
                continue;
            }
            visited.visit(start);
            let mut nodes = HashSet::new();
            let mut dfs = Dfs::new(&graph.deref(), start);

            while let Some(node) = dfs.next(&graph.deref()) {
                visited.visit(node);
                nodes.insert(node);
                node_portfolios.insert(node, portfolio_idx);
            }

            portfolios.push(Rc::new(Portfolio::new(nodes, Rc::clone(&graph))));

            portfolio_idx += 1;
        }

        PortfolioMap {
            portfolios,
            node_portfolios,
        }
    }

    pub fn all(&self) -> &Vec<Rc<Portfolio>> {
        &self.portfolios
    }

    pub fn for_node(&self, node: NodeIndex<u32>) -> Option<Rc<Portfolio>> {
        if let Some(idx) = self.node_portfolios.get(&node) {
            Some(Rc::clone(&self.portfolios[*idx]))
        } else {
            None
        }
    }
}
