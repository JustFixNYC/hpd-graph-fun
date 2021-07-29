use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use super::portfolio::{Portfolio, PortfolioMap};

#[derive(Debug)]
pub enum Node {
    Name(Rc<String>),
    BizAddr(Rc<String>),
}

impl Node {
    pub fn to_str(&self) -> &str {
        match self {
            Node::BizAddr(name) => name,
            Node::Name(name) => name,
        }
        .as_ref()
        .as_ref()
    }
}

type Edge = Vec<RegInfo>;

pub type HpdPetGraph = Graph<Node, Edge, petgraph::Undirected>;

#[derive(Debug)]
pub struct RegInfo {
    pub contact_id: u32,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
struct HpdRegistrationContact {
    #[serde(alias = "CorporationName")]
    corp_name: String,
    #[serde(alias = "FirstName")]
    first_name: String,
    #[serde(alias = "LastName")]
    last_name: String,
    #[serde(alias = "Type")]
    _type: String,
    #[serde(alias = "BusinessHouseNumber")]
    house_no: String,
    #[serde(alias = "BusinessStreetName")]
    street_name: String,
    #[serde(alias = "BusinessApartment")]
    apt_no: String,
    #[serde(alias = "BusinessCity")]
    city: String,
    #[serde(alias = "BusinessState")]
    state: String,
    #[serde(alias = "RegistrationContactID")]
    reg_contact_id: u32,
    #[serde(alias = "RegistrationID")]
    reg_id: u32,
}

pub struct HpdGraph {
    pub graph: HpdPetGraph,
    pub name_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    pub addr_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    portfolios: PortfolioMap,
}

impl HpdGraph {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut graph: HpdPetGraph = Graph::new_undirected();
        let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
        for result in rdr.deserialize() {
            let record: HpdRegistrationContact = result?;
            match record._type.as_ref() {
                "HeadOfficer" | "IndividualOwner" | "CorporateOwner" => {
                    if record.house_no == "" || record.street_name == "" {
                        continue;
                    }
                    let has_full_name = record.first_name != "" && record.last_name != "";
                    if !(has_full_name || record.corp_name != "") {
                        continue;
                    }
                    let name = if has_full_name {
                        Rc::new(format!("{} {}", record.first_name, record.last_name))
                    } else {
                        Rc::new(record.corp_name)
                    };
                    let addr = Rc::new(format!(
                        "{} {} {}, {} {}",
                        record.house_no,
                        record.street_name,
                        record.apt_no,
                        record.city,
                        record.state
                    ));
                    let addr_node = *addr_nodes
                        .entry(Rc::clone(&addr))
                        .or_insert_with(|| graph.add_node(Node::BizAddr(Rc::clone(&addr))));
                    let name_node = *name_nodes
                        .entry(Rc::clone(&name))
                        .or_insert_with(|| graph.add_node(Node::Name(Rc::clone(&name))));
                    let edge_idx = edges
                        .entry((name_node, addr_node))
                        .or_insert_with(|| graph.add_edge(name_node, addr_node, vec![]));
                    let edge = graph.edge_weight_mut(*edge_idx).unwrap();
                    edge.push(RegInfo {
                        id: record.reg_id,
                        contact_id: record.reg_contact_id,
                    });
                }
                _ => {}
            }
        }

        let portfolios = PortfolioMap::from_graph(&graph);

        Ok(HpdGraph {
            graph,
            name_nodes,
            addr_nodes,
            portfolios,
        })
    }

    pub fn portfolio_for_node(&self, node: NodeIndex<u32>) -> Option<&Portfolio> {
        self.portfolios.for_node(node)
    }

    pub fn path_to_string(&self, path: Vec<NodeIndex<u32>>) -> String {
        path.iter()
            .map(|node| self.graph.node_weight(*node).unwrap().to_str())
            .collect::<Vec<&str>>()
            .join(" -> ")
    }

    pub fn find_name(&self, search: &String) -> Option<NodeIndex<u32>> {
        if let Some(node) = self.name_nodes.get(search) {
            return Some(*node);
        }
        for (name, node) in self.name_nodes.iter() {
            if name.find(search).is_some() {
                return Some(*node);
            }
        }
        None
    }
}
