use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

use super::bbl::BBL;

#[derive(Deserialize)]
struct RawHpdRegistration {
    #[serde(alias = "RegistrationID")]
    reg_id: u32,

    #[serde(alias = "BoroID")]
    boro: u8,

    #[serde(alias = "Block")]
    block: u32,

    #[serde(alias = "Lot")]
    lot: u16,

    #[serde(alias = "RegistrationEndDate")]
    reg_end_date: String,
}

struct HpdRegistration {
    reg_id: u32,
    bbl: BBL,
    reg_end_date: NaiveDate,
}

pub struct HpdRegistrationMap {
    regs: HashMap<BBL, HpdRegistration>
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;
        let mut regs = HashMap::<BBL, HpdRegistration>::new();

        for result in rdr.deserialize() {
            let r: RawHpdRegistration = result?;
            let reg_end_date =
                NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y").unwrap();
            let bbl = BBL::from_numbers(r.boro, r.block, r.lot).unwrap();
            let should_insert = match regs.get(&bbl) {
                Some(reg) => reg.reg_end_date < reg_end_date,
                None => true
            };
            if should_insert {
                let reg = HpdRegistration {
                    reg_id: r.reg_id,
                    reg_end_date,
                    bbl,
                };
                regs.insert(bbl, reg);
            }
            count += 1;
        }

        println!("Parsed {} registrations (skipped {}).", regs.len(), count - regs.len());

        Ok(HpdRegistrationMap {
            regs
        })
    }
}
