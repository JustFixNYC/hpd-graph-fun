mod hpd_graph;
mod portfolio;

use clap::{value_t, App, AppSettings, Arg, SubCommand};
use petgraph::algo::{connected_components, dijkstra};
use petgraph::visit::VisitMap;
use std::collections::HashSet;
use std::error::Error;

use hpd_graph::{HpdGraph, Node};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
        println!("{}", portfolio.dot_graph(&hpd.graph));
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
