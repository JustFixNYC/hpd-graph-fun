use clap::{value_t, App, AppSettings, Arg, SubCommand};
use petgraph::algo::{connected_components, dijkstra};
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use petgraph::visit::VisitMap;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::rc::Rc;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

enum Node {
    Name(Rc<String>),
    BizAddr(Rc<String>),
}

type Edge = Vec<RegInfo>;

struct RegInfo {
    contact_id: u32,
    id: u32,
}

#[derive(Debug, Deserialize)]
struct HpdRegistration {
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

struct HpdGraph {
    graph: Graph<Node, Edge, petgraph::Undirected>,
    name_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    addr_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
}

impl HpdGraph {
    fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut graph = Graph::<Node, Edge, petgraph::Undirected>::new_undirected();
        let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
        for result in rdr.deserialize() {
            let record: HpdRegistration = result?;
            match record._type.as_ref() {
                "HeadOfficer" | "IndividualOwner" | "CorporateOwner" => {
                    if record.house_no == ""
                        || record.street_name == ""
                        || record.first_name == ""
                        || record.last_name == ""
                    {
                        continue;
                    }
                    let full_name = Rc::new(format!("{} {}", record.first_name, record.last_name));
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
                        .entry(Rc::clone(&full_name))
                        .or_insert_with(|| graph.add_node(Node::Name(Rc::clone(&full_name))));
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

        Ok(HpdGraph {
            graph,
            name_nodes,
            addr_nodes,
        })
    }
}

fn make_hpd_graph() -> Result<HpdGraph, Box<dyn Error>> {
    let rdr = csv::Reader::from_path("Registration_Contacts.csv")?;
    HpdGraph::from_csv(rdr)
}

fn cmd_info() -> Result<(), Box<dyn Error>> {
    let hpd = make_hpd_graph()?;
    let cc = connected_components(&hpd.graph);
    println!(
        "Read {} unique names, {} unique addresses, and {} connected components.",
        hpd.name_nodes.len(),
        hpd.addr_nodes.len(),
        cc
    );

    Ok(())
}

fn cmd_longpaths(min_length: u32) -> Result<(), Box<dyn Error>> {
    let hpd = make_hpd_graph()?;
    let mut visits = HashSet::new();

    println!("\nPaths with minimum length {}:\n", min_length);

    for node in hpd.graph.node_indices() {
        if visits.is_visited(&node) {
            continue;
        }
        visits.visit(node);
        if let Some(Node::Name(full_name)) = hpd.graph.node_weight(node) {
            let mut max_cost = 0;
            let mut max_cost_full_name = None;
            let dijkstra_map = dijkstra(&hpd.graph, node, None, |_| 1);
            for (other_node, cost) in dijkstra_map {
                visits.visit(other_node);
                if let Some(Node::Name(other_full_name)) = hpd.graph.node_weight(other_node) {
                    if cost > max_cost {
                        max_cost = cost;
                        max_cost_full_name = Some(Rc::clone(other_full_name));
                    }
                }
            }
            if max_cost >= min_length {
                println!(
                    "  {} {} -> {}",
                    max_cost,
                    full_name,
                    max_cost_full_name.unwrap()
                );
            }
        }
    }

    Ok(())
}

fn main() {
    let matches = App::new("hpd-graph-fun")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(VERSION)
        .author("Atul Varma <atul@justfix.nyc>")
        .about(
            "Fun with NYC Housing Preservation & Development (HPD) graph structure data analysis.",
        )
        .subcommand(
            SubCommand::with_name("info").about("Shows general information about the graph"),
        )
        .subcommand(
            SubCommand::with_name("longpaths")
                .about("Shows the longest paths in the graph")
                .arg(
                    Arg::with_name("min-length")
                        .short("m")
                        .value_name("LENGTH")
                        .default_value("10")
                        .help("Only show paths with this minimum length")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("longpaths") {
        let min_length = value_t!(matches.value_of("min-length"), u32).unwrap_or_else(|e| e.exit());
        cmd_longpaths(min_length).unwrap();
    } else if let Some(_) = matches.subcommand_matches("info") {
        cmd_info().unwrap();
    }
}
