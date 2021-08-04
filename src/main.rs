mod bbl;
mod hpd_graph;
mod hpd_registrations;
mod local_bridge;
mod portfolio;
mod ranking;
mod synonyms;

use chrono::Duration;
use clap::{value_t, App, AppSettings, Arg, SubCommand};
use petgraph::algo::{connected_components, dijkstra};
use petgraph::visit::VisitMap;
use std::collections::HashSet;
use std::error::Error;
use std::ops::Deref;
use std::rc::Rc;

use hpd_graph::{HpdGraph, Node};
use hpd_registrations::HpdRegistrationMap;
use portfolio::Portfolio;
use ranking::rank_tuples;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct ProgramArgs {
    max_expiration_age: i64,
    include_corps: bool,
}

struct Program {
    regs: HpdRegistrationMap,
    hpd: HpdGraph,
}

impl Program {
    fn new(args: ProgramArgs) -> Result<Self, Box<dyn Error>> {
        let reg_rdr = csv::Reader::from_path("Multiple_Dwelling_Registrations.csv")?;
        let regs = HpdRegistrationMap::from_csv(reg_rdr, Duration::days(args.max_expiration_age))?;

        let rdr = csv::Reader::from_path("Registration_Contacts.csv")?;
        let hpd = HpdGraph::from_csv(rdr, &regs, args.include_corps).unwrap();

        Ok(Program { regs, hpd })
    }

    fn cmd_json(&self, name: &str) {
        let portfolio = self.get_portfolio_with_name(&name.to_owned());
        println!("{}", portfolio.json());
    }

    fn cmd_info(&self, name: Option<&str>, top: usize) {
        let cc = connected_components(&self.hpd.graph.deref());
        println!(
            "Read {} unique names, {} unique addresses, and {} connected components.",
            self.hpd.name_nodes.len(),
            self.hpd.addr_nodes.len(),
            cc
        );

        if let Some(name) = name {
            let portfolio = self.get_portfolio_with_name(&name.to_owned());
            println!("This is {}.", portfolio.name());
            println!("It has {} buildings.", portfolio.building_count(&self.regs));

            let bizaddrs = portfolio.rank_bizaddrs();
            println!("\nThe most frequent business addresses mentioned in the portfolio are:\n");
            for (bizaddr, total_regs) in bizaddrs.iter().take(top) {
                println!(
                    "{} (mentioned in {} HPD registration contacts)",
                    bizaddr, total_regs
                );
            }

            let names = portfolio.rank_names();
            println!("\nThe most frequent names mentioned in the portfolio are:\n");
            for (name, total_regs) in names.iter().take(top) {
                println!(
                    "{} (mentioned in {} HPD registration contacts)",
                    name, total_regs
                );
            }

            let bridges = portfolio.find_local_bridges().len();

            if bridges > 0 {
                println!(
                    "\nThe portfolio has {} local bridge{}.\n",
                    bridges,
                    if bridges > 1 { "s" } else { "" }
                );
            }
        }
    }

    fn get_portfolio_with_name(&self, name: &String) -> Rc<Portfolio> {
        if let Some(node) = self.hpd.find_name(&name) {
            eprintln!(
                "Found a matching name '{}'.",
                self.hpd.graph.node_weight(node).unwrap().to_str()
            );
            self.hpd.portfolios().for_node(node).unwrap()
        } else {
            eprintln!("Unable to find a match for the name '{}'.", &name);
            std::process::exit(1);
        }
    }

    fn cmd_dot(&self, name: &String) {
        let portfolio = self.get_portfolio_with_name(name);
        println!("{}", portfolio.dot_graph());
    }

    fn cmd_ranking(&self, min_buildings: usize) {
        let mut ranking: Vec<(&Portfolio, usize)> = vec![];
        let portfolios = self.hpd.portfolios();

        for portfolio in portfolios.all() {
            let size = portfolio.building_count(&self.regs);
            if size >= min_buildings {
                ranking.push((portfolio, size));
            }
        }

        rank_tuples(&mut ranking);

        let mut rank = 1;
        for (portfolio, size) in ranking {
            let name = portfolio.name();
            println!("{}. {} - {} buildings", rank, name, size);
            rank += 1;
        }
    }

    fn cmd_longpaths(&self, min_length: u32) {
        let mut visits = HashSet::new();

        println!("\nPaths with minimum length {}:\n", min_length);

        for node in self.hpd.graph.node_indices() {
            if visits.is_visited(&node) {
                continue;
            }
            visits.visit(node);
            if let Some(Node::Name(_)) = self.hpd.graph.node_weight(node) {
                let mut max_cost = 0;
                let mut max_cost_node = None;
                let dijkstra_map = dijkstra(&self.hpd.graph.deref(), node, None, |_| 1);
                for (other_node, cost) in dijkstra_map {
                    visits.visit(other_node);
                    if let Some(Node::Name(_)) = self.hpd.graph.node_weight(other_node) {
                        if cost > max_cost {
                            max_cost = cost;
                            max_cost_node = Some(other_node);
                        }
                    }
                }
                if max_cost >= min_length {
                    if let Some(other_node) = max_cost_node {
                        let (_, path) = petgraph::algo::astar(
                            &self.hpd.graph.deref(),
                            node,
                            |n| n == other_node,
                            |_| 1,
                            |_| 1,
                        )
                        .unwrap();
                        println!(
                            "length {} path: {}\n",
                            max_cost,
                            &self.hpd.path_to_string(path)
                        );
                    }
                }
            }
        }
    }
}

fn main() {
    let matches = App::new("hpd-graph-fun")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(VERSION)
        .author("Atul Varma <atul@justfix.nyc>")
        .about(
            "Fun with NYC Housing Preservation & Development (HPD) graph structure data analysis.",
        )
        .arg(
            Arg::with_name("max-expiration-age")
                .long("max-expiration-age")
                .value_name("DAYS")
                .default_value("90")
                .takes_value(true)
                .help(
                    "Ignore HPD registrations that have expired more than this number of days ago",
                ),
        )
        .arg(
            Arg::with_name("include-corps")
                .long("include-corps")
                .help("Include corporation names in portfolios"),
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("Shows general information about the graph")
                .arg(Arg::with_name("NAME"))
                .arg(
                    Arg::with_name("top")
                        .short("t")
                        .long("top")
                        .value_name("N")
                        .default_value("5")
                        .help("Show the top N names and business addresses in the portfolio")
                        .takes_value(true),
                ),
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
        .subcommand(
            SubCommand::with_name("json")
                .about("Output JSON of a particular portfolio")
                .arg(Arg::with_name("NAME").required(true)),
        )
        .subcommand(
            SubCommand::with_name("ranking")
                .about("Show a ranking of the largest portfolios")
                .arg(
                    Arg::with_name("min-buildings")
                        .short("b")
                        .long("min-buildings")
                        .default_value("0")
                        .help("Only show portfolios of a minimum size")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let args = ProgramArgs {
        max_expiration_age: value_t!(matches.value_of("max-expiration-age"), i64)
            .unwrap_or_else(|e| e.exit()),
        include_corps: matches.is_present("include-corps"),
    };
    if let Some(matches) = matches.subcommand_matches("longpaths") {
        let min_length = value_t!(matches.value_of("min-length"), u32).unwrap_or_else(|e| e.exit());
        Program::new(args).unwrap().cmd_longpaths(min_length);
    } else if let Some(matches) = matches.subcommand_matches("info") {
        let name = matches.value_of("NAME");
        let top = value_t!(matches.value_of("top"), usize).unwrap_or_else(|e| e.exit());
        Program::new(args).unwrap().cmd_info(name, top);
    } else if let Some(matches) = matches.subcommand_matches("dot") {
        let name = matches.value_of("NAME").unwrap().to_owned();
        Program::new(args).unwrap().cmd_dot(&name);
    } else if let Some(matches) = matches.subcommand_matches("json") {
        let name = matches.value_of("NAME").unwrap().to_owned();
        Program::new(args).unwrap().cmd_json(&name);
    } else if let Some(matches) = matches.subcommand_matches("ranking") {
        let min_buildings =
            value_t!(matches.value_of("min-buildings"), usize).unwrap_or_else(|e| e.exit());
        Program::new(args).unwrap().cmd_ranking(min_buildings);
    }
}
