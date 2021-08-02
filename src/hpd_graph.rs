use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use super::hpd_registrations::HpdRegistrationMap;
use super::portfolio::{Portfolio, PortfolioMap};
use super::synonyms::Synonyms;

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
struct HpdRegistrationContact<'a> {
    #[serde(alias = "CorporationName")]
    corp_name: &'a str,
    #[serde(alias = "FirstName")]
    first_name: &'a str,
    #[serde(alias = "LastName")]
    last_name: &'a str,
    #[serde(alias = "Type")]
    _type: &'a str,
    #[serde(alias = "BusinessHouseNumber")]
    house_no: &'a str,
    #[serde(alias = "BusinessStreetName")]
    street_name: &'a str,
    #[serde(alias = "BusinessApartment")]
    apt_no: &'a str,
    #[serde(alias = "BusinessCity")]
    city: &'a str,
    #[serde(alias = "BusinessState")]
    state: &'a str,
    #[serde(alias = "RegistrationContactID")]
    reg_contact_id: u32,
    #[serde(alias = "RegistrationID")]
    reg_id: u32,
}

pub struct HpdGraph {
    pub graph: HpdPetGraph,
    pub name_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    pub addr_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    pub portfolios: PortfolioMap,
}

impl HpdGraph {
    pub fn from_csv<T: std::io::Read>(
        mut rdr: csv::Reader<T>,
        regs: &HpdRegistrationMap,
        include_corps: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let synonyms = Synonyms::new();
        let mut graph: HpdPetGraph = Graph::new_undirected();
        let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
        let mut raw_record = csv::StringRecord::new();
        let headers = rdr.headers()?.clone();

        while rdr.read_record(&mut raw_record)? {
            let record: HpdRegistrationContact = raw_record.deserialize(Some(&headers))?;
            match record._type.as_ref() {
                "HeadOfficer" | "IndividualOwner" | "CorporateOwner" => {
                    if record.house_no == "" || record.street_name == "" {
                        continue;
                    }
                    let has_full_name = record.first_name != "" && record.last_name != "";
                    if !(has_full_name || (include_corps && record.corp_name != "")) {
                        continue;
                    }
                    if regs.is_expired_or_invalid(record.reg_id) {
                        continue;
                    }
                    let name_string = if has_full_name {
                        format!("{} {}", record.first_name, record.last_name)
                    } else {
                        record.corp_name.to_owned()
                    };
                    let name = synonyms
                        .get(&name_string)
                        .unwrap_or_else(|| Rc::new(name_string));
                    let mut addr_string = format!(
                        "{} {} {}, {} {}",
                        record.house_no,
                        record.street_name,
                        record.apt_no,
                        record.city,
                        record.state
                    );
                    addr_string.make_ascii_uppercase();
                    let addr = Rc::new(addr_string);
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
