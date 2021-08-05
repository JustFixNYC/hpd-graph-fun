use maud::{html, PreEscaped, DOCTYPE};
use std::rc::Rc;

use super::hpd_registrations::HpdRegistrationMap;
use super::portfolio::{Portfolio, PortfolioMap};

fn slugify<T: AsRef<str>>(value: T) -> String {
    let value_ref = value.as_ref();
    value_ref
        .chars()
        .map(|c| match c {
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            'a'..='z' => Some(c),
            ' ' => Some('_'),
            _ => None,
        })
        .flatten()
        .collect::<String>()
}

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
        script type="application/json" id="portfolio" { (PreEscaped(portfolio.json())) }
        script src="main.bundle.js" { }
    };

    let html = page.into_string();

    println!(
        "TODO: Write {} as {}.html.",
        html,
        slugify(portfolio.name().as_ref())
    );
}

pub fn make_website(portfolio_map: PortfolioMap, regs: &HpdRegistrationMap, min_buildings: usize) {
    let portfolios = portfolio_map.rank_by_building_count(&regs, min_buildings);

    for (portfolio, _) in &portfolios {
        write_portfolio_html(portfolio);
    }

    println!("Exported {} portfolios.", portfolios.len());
}

#[test]
fn test_slugify_works() {
    assert_eq!(slugify("hello"), "hello".to_owned());
    assert_eq!(slugify("HELLO"), "hello".to_owned());
    assert_eq!(slugify("BOOP'S portfolio"), "boops_portfolio".to_owned());
}
