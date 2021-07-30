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

pub struct HpdRegistration {
    reg_id: u32,
    bbl: BBL,
    reg_end_date: NaiveDate,
}

pub struct HpdRegistrationMap {
    regs_by_id: HashMap<u32, HpdRegistration>,
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;
        let mut regs_by_bbl = HashMap::<BBL, HpdRegistration>::new();

        for result in rdr.deserialize() {
            let r: RawHpdRegistration = result?;
            let reg_end_date =
                NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y").unwrap();
            let bbl = BBL::from_numbers(r.boro, r.block, r.lot).unwrap();
            let should_insert = match regs_by_bbl.get(&bbl) {
                Some(reg) => reg.reg_end_date < reg_end_date,
                None => true
            };
            if should_insert {
                let reg = HpdRegistration {
                    reg_id: r.reg_id,
                    reg_end_date,
                    bbl,
                };
                regs_by_bbl.insert(bbl, reg);
            }
            count += 1;
        }

        println!("Loaded {} registrations (skipped {}).", regs_by_bbl.len(), count - regs_by_bbl.len());

        let mut regs_by_id = HashMap::<u32, HpdRegistration>::with_capacity(regs_by_bbl.len());

        for (bbl, reg) in regs_by_bbl.into_iter() {
            let reg_end_date = reg.reg_end_date;
            let old = regs_by_id.insert(reg.reg_id, reg);
            if let Some(old) = old {
                println!("Warning: HPD registration {} is for {:?} ({}) and {:?} ({})!", old.reg_id, old.bbl, old.reg_end_date, bbl, reg_end_date);
            }
        }

        Ok(HpdRegistrationMap {
            regs_by_id
        })
    }

    pub fn get_by_id(&self, id: u32) -> Option<&HpdRegistration> {
        self.regs_by_id.get(&id)
    }
}
