use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, VisitMap};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::hpd_graph::{HpdPetGraph, Node};
use super::hpd_registrations::HpdRegistrationMap;

pub struct Portfolio {
    nodes: HashSet<NodeIndex<u32>>,
    pub name: String,
}

impl Portfolio {
    fn new(nodes: HashSet<NodeIndex<u32>>, name: String) -> Self {
        Portfolio { nodes, name }
    }

    pub fn rank_bizaddrs(&self, g: &HpdPetGraph) -> Vec<(Rc<String>, usize)> {
        let mut result = vec![];

        for node in self.nodes.iter() {
            if let Node::BizAddr(name) = g.node_weight(*node).unwrap() {
                let mut total_regs = 0;
                for edge in g.edges(*node) {
                    let reg_info = edge.weight();
                    total_regs += reg_info.len();
                }
                result.push((Rc::clone(name), total_regs));
            }
        }

        result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        result.reverse();
        result
    }

    pub fn get_best_name(&self, g: &HpdPetGraph) -> Option<String> {
        let mut best: Option<(NodeIndex<u32>, usize)> = None;
        for node in self.nodes.iter() {
            if let Node::Name(_) = g.node_weight(*node).unwrap() {
                let mut total_regs = 0;
                for edge in g.edges(*node) {
                    let reg_info = edge.weight();
                    total_regs += reg_info.len();
                }
                let is_new_best = if let Some((_, current_total_regs)) = best {
                    current_total_regs < total_regs
                } else {
                    true
                };
                if is_new_best {
                    best = Some((*node, total_regs));
                }
            }
        }
        if let Some((node_idx, _)) = best {
            let node = g.node_weight(node_idx).unwrap();
            Some(node.to_str().to_owned())
        } else {
            None
        }
    }

    pub fn building_count(&self, g: &HpdPetGraph, regs: &HpdRegistrationMap) -> usize {
        let mut bins = HashSet::<u32>::new();
        for node in self.nodes.iter() {
            if let Node::Name(_) = g.node_weight(*node).unwrap() {
                for edge in g.edges(*node) {
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

    pub fn dot_graph(&self, g: &HpdPetGraph) -> String {
        let gf = petgraph::visit::NodeFiltered::from_fn(&g, |g| self.nodes.is_visited(&g));
        let d = Dot::with_attr_getters(
            &gf,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, edge| format!("label=\"{}\"", edge.weight().len()),
            &|_, (_, node)| match node {
                Node::BizAddr(addr) => {
                    format!(
                        "label=\"{}\", color=lightblue2, style=filled, shape=box",
                        addr
                    )
                }
                Node::Name(name) => format!("label=\"{}\", color=whitesmoke, style=filled", name),
            },
        );

        format!(
            "// {}'s portfolio\n\n{:?}",
            self.get_best_name(g).unwrap_or(self.name.clone()),
            d
        )
    }
}

pub struct PortfolioMap {
    portfolios: Vec<Portfolio>,
    node_portfolios: HashMap<NodeIndex<u32>, usize>,
}

impl PortfolioMap {
    pub fn from_graph(graph: &HpdPetGraph) -> Self {
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
            let mut dfs = Dfs::new(&graph, start);

            while let Some(node) = dfs.next(&graph) {
                visited.visit(node);
                nodes.insert(node);
                node_portfolios.insert(node, portfolio_idx);
            }

            portfolios.push(Portfolio::new(
                nodes,
                format!("Portfolio #{}", portfolio_idx),
            ));

            portfolio_idx += 1;
        }

        PortfolioMap {
            portfolios,
            node_portfolios,
        }
    }

    pub fn all(&self) -> &Vec<Portfolio> {
        &self.portfolios
    }

    pub fn for_node(&self, node: NodeIndex<u32>) -> Option<&Portfolio> {
        if let Some(idx) = self.node_portfolios.get(&node) {
            Some(&self.portfolios[*idx])
        } else {
            None
        }
    }
}
