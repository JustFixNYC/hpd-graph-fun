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

    #[serde(alias = "BIN")]
    bin: Option<u32>,

    #[serde(alias = "RegistrationEndDate")]
    reg_end_date: String,
}

pub struct HpdRegistration {
    reg_id: u32,
    bbl: BBL,
    bin: u32,
    reg_end_date: NaiveDate,
}

pub struct HpdRegistrationMap {
    regs_by_id: HashMap<u32, HpdRegistration>,
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;
        let mut regs_by_bin = HashMap::<u32, HpdRegistration>::new();

        for result in rdr.deserialize() {
            let r: RawHpdRegistration = result?;
            let reg_end_date =
                NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y").unwrap();
            let bbl = BBL::from_numbers(r.boro, r.block, r.lot).unwrap();
            if let Some(bin) = r.bin {
                let should_insert = match regs_by_bin.get(&bin) {
                    Some(reg) => reg.reg_end_date < reg_end_date,
                    None => true
                };
                if should_insert {
                    let reg = HpdRegistration {
                        reg_id: r.reg_id,
                        reg_end_date,
                        bbl,
                        bin
                    };
                    regs_by_bin.insert(reg.bin, reg);
                }
            } else {
                println!("Warning: HPD registration {} ({}) has no BIN!", r.reg_id, r.reg_end_date);
            }
            count += 1;
        }

        println!("Loaded {} registrations (skipped {}).", regs_by_bin.len(), count - regs_by_bin.len());

        let mut regs_by_id = HashMap::<u32, HpdRegistration>::with_capacity(regs_by_bin.len());

        for (bin, reg) in regs_by_bin.into_iter() {
            let reg_end_date = reg.reg_end_date;
            let old = regs_by_id.insert(reg.reg_id, reg);
            if let Some(old) = old {
                println!("Warning: HPD registration {} is for {} ({}) and {} ({})!", old.reg_id, old.bin, old.reg_end_date, bin, reg_end_date);
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
