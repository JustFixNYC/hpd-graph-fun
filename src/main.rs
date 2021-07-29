use clap::{value_t, App, AppSettings, Arg, SubCommand};
use petgraph::algo::{connected_components, dijkstra};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use petgraph::visit::{Dfs, VisitMap};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::rc::Rc;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum Node {
    Name(Rc<String>),
    BizAddr(Rc<String>),
}

impl Node {
    fn to_str(&self) -> &str {
        match self {
            Node::BizAddr(name) => name,
            Node::Name(name) => name,
        }
        .as_ref()
        .as_ref()
    }
}

type Edge = Vec<RegInfo>;

type HpdPetGraph = Graph<Node, Edge, petgraph::Undirected>;

#[derive(Debug)]
struct RegInfo {
    #[allow(dead_code)]
    contact_id: u32,

    #[allow(dead_code)]
    id: u32,
}

#[derive(Debug, Deserialize)]
struct HpdRegistration {
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

struct Portfolio {
    nodes: HashSet<NodeIndex<u32>>,
    name: String,
}

impl Portfolio {
    fn new(nodes: HashSet<NodeIndex<u32>>, name: String) -> Self {
        Portfolio { nodes, name }
    }

    fn dot_graph(&self, hpd: &HpdGraph) -> String {
        let g = &hpd.graph;
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

struct PortfolioMap {
    portfolios: Vec<Portfolio>,
    node_portfolios: HashMap<NodeIndex<u32>, usize>,
}

impl PortfolioMap {
    fn from_graph(graph: &HpdPetGraph) -> Self {
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

    fn for_node(&self, node: NodeIndex<u32>) -> Option<&Portfolio> {
        if let Some(idx) = self.node_portfolios.get(&node) {
            Some(&self.portfolios[*idx])
        } else {
            None
        }
    }
}

struct HpdGraph {
    graph: HpdPetGraph,
    name_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    addr_nodes: HashMap<Rc<String>, NodeIndex<u32>>,
    portfolios: PortfolioMap,
}

impl HpdGraph {
    fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut graph: HpdPetGraph = Graph::new_undirected();
        let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
        for result in rdr.deserialize() {
            let record: HpdRegistration = result?;
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

    fn portfolio_for_node(&self, node: NodeIndex<u32>) -> Option<&Portfolio> {
        self.portfolios.for_node(node)
    }

    fn path_to_string(&self, path: Vec<NodeIndex<u32>>) -> String {
        path.iter()
            .map(|node| self.graph.node_weight(*node).unwrap().to_str())
            .collect::<Vec<&str>>()
            .join(" -> ")
    }

    fn find_name(&self, search: &String) -> Option<NodeIndex<u32>> {
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

fn cmd_dot(name: &String) -> Result<(), Box<dyn Error>> {
    let hpd = make_hpd_graph()?;

    if let Some(node) = hpd.find_name(&name) {
        eprintln!(
            "Found a matching name '{}'.",
            hpd.graph.node_weight(node).unwrap().to_str()
        );
        let portfolio = hpd.portfolio_for_node(node).unwrap();
        println!("{}", portfolio.dot_graph(&hpd));
    } else {
        eprintln!("Unable to find a match for the name '{}'.", &name);
        std::process::exit(1);
    }

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
        if let Some(Node::Name(_)) = hpd.graph.node_weight(node) {
            let mut max_cost = 0;
            let mut max_cost_node = None;
            let dijkstra_map = dijkstra(&hpd.graph, node, None, |_| 1);
            for (other_node, cost) in dijkstra_map {
                visits.visit(other_node);
                if let Some(Node::Name(_)) = hpd.graph.node_weight(other_node) {
                    if cost > max_cost {
                        max_cost = cost;
                        max_cost_node = Some(other_node);
                    }
                }
            }
            if max_cost >= min_length {
                if let Some(other_node) = max_cost_node {
                    let (_, path) =
                        petgraph::algo::astar(&hpd.graph, node, |n| n == other_node, |_| 1, |_| 1)
                            .unwrap();
                    println!("length {} path: {}\n", max_cost, &hpd.path_to_string(path));
                }
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
        .subcommand(
            SubCommand::with_name("dot")
                .about("Output a dot graph of a particular portfolio")
                .arg(Arg::with_name("NAME").required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("longpaths") {
        let min_length = value_t!(matches.value_of("min-length"), u32).unwrap_or_else(|e| e.exit());
        cmd_longpaths(min_length).unwrap();
    } else if let Some(_) = matches.subcommand_matches("info") {
        cmd_info().unwrap();
    } else if let Some(matches) = matches.subcommand_matches("dot") {
        let name = matches.value_of("NAME").unwrap().to_owned();
        cmd_dot(&name).unwrap();
    }
}
