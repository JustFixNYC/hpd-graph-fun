use chrono::{NaiveDate, Duration};
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

    #[serde(alias = "BIN")]
    bin: Option<u32>,

    #[serde(alias = "RegistrationEndDate")]
    reg_end_date: String,
}

#[derive(Debug)]
pub struct HpdRegistration {
    reg_id: u32,
    bbl: BBL,
    bin: Option<u32>,
    reg_end_date: NaiveDate,
}

pub struct HpdRegistrationMap {
    regs_by_id: HashMap<u32, Vec<HpdRegistration>>,
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>, max_expiration_age: Duration) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;
        let mut regs_by_id = HashMap::<u32, Vec<HpdRegistration>>::new();
        let today = chrono::offset::Local::today().naive_local();

        for result in rdr.deserialize() {
            let r: RawHpdRegistration = result?;
            let reg_end_date =
                NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y").unwrap();
            let bbl = BBL::from_numbers(r.boro, r.block, r.lot).unwrap();
            let age = today - reg_end_date;
            if age < max_expiration_age {
                let reg = HpdRegistration {
                    reg_id: r.reg_id,
                    reg_end_date,
                    bbl,
                    bin: r.bin
                };
                let regs = regs_by_id.entry(r.reg_id).or_insert_with(|| vec![]);
                regs.push(reg);
            }
            count += 1;
        }

        println!("Loaded {} registrations (skipped {}).", regs_by_id.len(), count - regs_by_id.len());

        Ok(HpdRegistrationMap {
            regs_by_id
        })
    }

    pub fn is_expired(&self, id: u32) -> bool {
        self.regs_by_id.get(&id).is_none()
    }
}
