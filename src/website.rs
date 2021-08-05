use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

use super::hpd_registrations::HpdRegistrationMap;
use super::portfolio::{Portfolio, PortfolioMap};

static SITE_DIR: &'static str = "public";
static INDEX_FILENAME: &'static str = "index.html";

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

fn write_website_file<T: AsRef<str>>(filename: T, content: &String) -> Result<(), Box<dyn Error>> {
    let filename: PathBuf = [SITE_DIR, filename.as_ref()].iter().collect();

    match std::fs::write(filename, content) {
        Ok(result) => Ok(result),
        Err(e) => Err(Box::new(e)),
    }
}

fn header<T: AsRef<str>>(title: T) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        link rel="stylesheet" href="styles.css";
        title { (title) }
        h1 { (title) }
    }
}

fn portfolio_html(portfolio: &Rc<Portfolio>) -> String {
    let page = html! {
        (header(portfolio.name().as_ref()))
        div id="graph" {}
        form id="search-form" {
            input type="search" value="" placeholder="🔎" id="search-input";
            button type="submit" { "Search" }
        }
        p id="message" {}
        p id="back" { a href=(INDEX_FILENAME) { "« Back" } }
        script type="application/json" id="portfolio" { (PreEscaped(portfolio.json())) }
        script src="main.bundle.js" { }
    };

    page.into_string()
}

pub fn make_website(
    portfolio_map: PortfolioMap,
    regs: &HpdRegistrationMap,
    min_buildings: usize,
) -> Result<(), Box<dyn Error>> {
    let portfolios = portfolio_map.rank_by_building_count(&regs, min_buildings);
    let mut list_items: Vec<(String, Rc<String>)> = vec![];

    for (portfolio, _) in &portfolios {
        let html = portfolio_html(portfolio);
        let name = portfolio.name();
        let filename = format!("{}.html", slugify(name.as_ref()));
        write_website_file(&filename, &html)?;
        list_items.push((filename, name));
    }

    let index_html = html! {
        (header("hpd-graph-fun"))
        ul {
            @for (href, name) in &list_items {
                li { a href=(href) { (name) } }
            }
        }
    };

    write_website_file(&INDEX_FILENAME, &index_html.into_string())?;
    println!(
        "Exported {} portfolios. You can view them at {}/{}.",
        portfolios.len(),
        SITE_DIR,
        INDEX_FILENAME
    );
    Ok(())
}

#[test]
fn test_slugify_works() {
    assert_eq!(slugify("hello"), "hello".to_owned());
    assert_eq!(slugify("HELLO"), "hello".to_owned());
    assert_eq!(slugify("BOOP'S portfolio"), "boops_portfolio".to_owned());
}
