use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, VisitMap};
use std::collections::{HashMap, HashSet};

use super::hpd_graph::HpdPetGraph;
use super::hpd_registrations::HpdRegistrationMap;

pub struct Portfolio {
    nodes: HashSet<NodeIndex<u32>>,
    pub name: String,
}

impl Portfolio {
    fn new(nodes: HashSet<NodeIndex<u32>>, name: String) -> Self {
        Portfolio { nodes, name }
    }

    pub fn building_count(&self, g: &HpdPetGraph, regs: &HpdRegistrationMap) -> usize {
        use petgraph::visit::IntoEdgeReferences;

        let g = petgraph::visit::NodeFiltered::from_fn(&g, move |g| self.nodes.is_visited(&g));
        let mut bins = HashSet::<u32>::new();
        for edge in g.edge_references() {
            for reg_info in edge.weight() {
                for reg in regs.get_by_id(reg_info.id).unwrap() {
                    bins.insert(reg.reg_id);
                }
            }
        }
        bins.len()
    }

    pub fn dot_graph(&self, g: &HpdPetGraph) -> String {
        let g = petgraph::visit::NodeFiltered::from_fn(&g, move |g| self.nodes.is_visited(&g));
        let d = Dot::with_attr_getters(
            &g,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, edge| format!("label=\"{}\"", edge.weight().len()),
            &|_, (_, whatever)| format!("label=\"{}\"", whatever.to_str()),
        );

        format!("// {}\n\n{:?}", self.name, d)
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

    pub fn for_node(&self, node: NodeIndex<u32>) -> Option<&Portfolio> {
        if let Some(idx) = self.node_portfolios.get(&node) {
            Some(&self.portfolios[*idx])
        } else {
            None
        }
    }
}
