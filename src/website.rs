use super::hpd_registrations::HpdRegistrationMap;
use super::portfolio::PortfolioMap;

pub fn make_website(portfolio_map: PortfolioMap, regs: &HpdRegistrationMap, min_buildings: usize) {
    let portfolios = portfolio_map.rank_by_building_count(&regs, min_buildings);

    println!("TODO: Export {} portfolios.", portfolios.len());
}
