use maud::{html, DOCTYPE};
use std::rc::Rc;

use super::hpd_registrations::HpdRegistrationMap;
use super::portfolio::{Portfolio, PortfolioMap};

fn write_portfolio_html(portfolio: &Rc<Portfolio>) {
    let page = html! {
        (DOCTYPE)
        meta charset="utf-8";
        link rel="stylesheet" href="styles.css";
        title { (portfolio.name()) }
        div id="graph" {}
        h1 { (portfolio.name()) }
        form id="search-form" {
            input type="search" value="" placeholder="🔎" id="search-input";
            button type="submit" { "Search" }
        }
        p id="message" {}
        script src="main.bundle.js" { }
    };

    let html = page.into_string();

    println!("TODO: Write {}", html);
}

pub fn make_website(portfolio_map: PortfolioMap, regs: &HpdRegistrationMap, min_buildings: usize) {
    let portfolios = portfolio_map.rank_by_building_count(&regs, min_buildings);

    for (portfolio, _) in &portfolios {
        write_portfolio_html(portfolio);
    }

    println!("Exported {} portfolios.", portfolios.len());
}
