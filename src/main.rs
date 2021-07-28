use std::error::Error;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use petgraph::visit::VisitMap;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use petgraph::algo::{connected_components, dijkstra};
use serde::Deserialize;

enum Node {
    Name(Rc<String>),
    BizAddr(Rc<String>),
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
}

struct HpdGraph {
    graph: Graph::<Node, (), petgraph::Undirected>,
    name_nodes: HashMap::<Rc<String>, NodeIndex<u32>>,
    addr_nodes: HashMap::<Rc<String>, NodeIndex<u32>>,
}

impl HpdGraph {
    fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut graph = Graph::<Node, (), petgraph::Undirected>::new_undirected();
        let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
        let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
        for result in rdr.deserialize() {
            let record: HpdRegistration = result?;
            match record._type.as_ref() {
                "HeadOfficer"|"IndividualOwner"|"CorporateOwner" => {
                    if record.house_no == "" || record.street_name == "" || record.first_name == "" || record.last_name == "" {
                        continue;
                    }
                    let full_name = Rc::new(format!("{} {}", record.first_name, record.last_name));
                    let addr = Rc::new(format!("{} {} {}, {} {}", record.house_no, record.street_name, record.apt_no, record.city, record.state));
                    let addr_node = *addr_nodes.entry(Rc::clone(&addr)).or_insert_with(|| {
                        graph.add_node(Node::BizAddr(Rc::clone(&addr)))
                    });
                    let name_node = *name_nodes.entry(Rc::clone(&full_name)).or_insert_with(|| {
                        graph.add_node(Node::Name(Rc::clone(&full_name)))
                    });
                    edges.entry((name_node, addr_node)).or_insert_with(|| {
                        graph.add_edge(name_node, addr_node, ())
                    });
                },
                _ => {}
            }
        }

        Ok(HpdGraph {
            graph,
            name_nodes,
            addr_nodes
        })
    }


}

fn example() -> Result<(), Box<dyn Error>> {
    let rdr = csv::Reader::from_path("Registration_Contacts.csv")?;
    let hpd = HpdGraph::from_csv(rdr)?;
    let cc = connected_components(&hpd.graph);
    println!(
        "Read {} unique names, {} unique addresses, and {} connected components.",
        hpd.name_nodes.len(),
        hpd.addr_nodes.len(),
        cc
    );

    let mut visits = HashSet::new();

    println!("\nLongest paths:\n");

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
            if max_cost > 10 {
                println!("  {} {} -> {}", max_cost, full_name, max_cost_full_name.unwrap());
            }
        }
    }

    println!("\nVisited {} total nodes.", visits.len());

    Ok(())
}

fn main() {
    example().expect("it should work!");
}
