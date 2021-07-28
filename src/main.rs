use std::error::Error;
use std::rc::Rc;
use std::collections::HashMap;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use petgraph::algo::connected_components;
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

fn example() -> Result<(), Box<dyn Error>> {
    let mut graph = Graph::<Node, (), petgraph::Undirected>::new_undirected();
    let mut name_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
    let mut addr_nodes = HashMap::<Rc<String>, NodeIndex<u32>>::new();
    let mut edges = HashMap::<(NodeIndex<u32>, NodeIndex<u32>), EdgeIndex<u32>>::new();
    let mut rdr = csv::Reader::from_path("Registration_Contacts.csv")?;
    for result in rdr.deserialize() {
        let record: HpdRegistration = result?;
        match record._type.as_ref() {
            "HeadOfficer"|"IndividualOwner"|"CorporateOwner" => {
                if record.house_no == "" || record.street_name == "" || record.first_name == "" || record.last_name == "" {
                    continue;
                }
                let full_name = Rc::new(format!("{} {}", record.first_name, record.last_name));
                let addr = Rc::new(format!("{} {} {}, {} {}", record.house_no, record.street_name, record.apt_no, record.city, record.state));
                if !addr_nodes.contains_key(&addr) {
                    let ni = graph.add_node(Node::BizAddr(Rc::clone(&addr)));
                    addr_nodes.insert(Rc::clone(&addr), ni);
                }
                if !name_nodes.contains_key(&full_name) {
                    let ni = graph.add_node(Node::Name(Rc::clone(&full_name)));
                    name_nodes.insert(Rc::clone(&full_name), ni);
                }
                let name_node = *name_nodes.get(&full_name).unwrap();
                let addr_node = *addr_nodes.get(&addr).unwrap();
                let edge = (name_node, addr_node);
                if !edges.contains_key(&edge) {
                    let ei = graph.add_edge(name_node, addr_node, ());
                    edges.insert(edge, ei);
                }
            },
            _ => {}
        }
    }
    let cc = connected_components(&graph);
    println!(
        "Read {} unique names, {} unique addresses, and {} unique edges and {} connected components.",
        name_nodes.len(),
        addr_nodes.len(),
        edges.len(),
        cc
    );

    /*
    for start in graph.node_indices() {
        let mut dfs = petgraph::visit::Dfs::new(&graph, start);
        let mut visited_vec = vec![];

        while let Some(visited) = dfs.next(&graph) {
            visited_vec.push(visited);
        }

        if visited_vec.len() > 10 {
            let visited_strs: Vec<&str> = visited_vec.iter().map(|v| graph.node_weight(*v).unwrap().as_ref()).collect();
            println!("Ooooo {}: {}", visited_vec.len(), visited_strs.join(" ->  "));
        }
    }*/
    //use petgraph::dot::{Dot, Config};
    //let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    //println!("{:?}", dot);
    Ok(())
}

fn main() {
    example().expect("it should work!");
}
