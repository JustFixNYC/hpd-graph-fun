use chrono::{Duration, NaiveDate};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

use super::bbl::BBL;

#[derive(Deserialize)]
struct RawHpdRegistration<'a> {
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
    reg_end_date: &'a str,
}

#[derive(Debug)]
pub struct HpdRegistration {
    pub reg_id: u32,
    pub bbl: BBL,
    pub bin: Option<u32>,
    pub reg_end_date: NaiveDate,
}

pub struct HpdRegistrationMap {
    regs_by_id: HashMap<u32, Vec<HpdRegistration>>,
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(
        mut rdr: csv::Reader<T>,
        max_expiration_age: Duration,
    ) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;
        let mut regs_by_id = HashMap::<u32, Vec<HpdRegistration>>::new();
        let today = chrono::offset::Local::today().naive_local();
        let mut raw_record = csv::StringRecord::new();
        let headers = rdr.headers()?.clone();

        while rdr.read_record(&mut raw_record)? {
            let r: RawHpdRegistration = raw_record.deserialize(Some(&headers))?;
            let reg_end_date =
                NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y").unwrap();
            let bbl = BBL::from_numbers(r.boro, r.block, r.lot).unwrap();
            let age = today - reg_end_date;
            if age < max_expiration_age {
                let reg = HpdRegistration {
                    reg_id: r.reg_id,
                    reg_end_date,
                    bbl,
                    bin: r.bin,
                };
                let regs = regs_by_id.entry(r.reg_id).or_insert_with(|| vec![]);
                regs.push(reg);
            }
            count += 1;
        }

        eprintln!(
            "Loaded {} registrations (skipped {}).",
            regs_by_id.len(),
            count - regs_by_id.len()
        );

        Ok(HpdRegistrationMap { regs_by_id })
    }

    pub fn is_expired_or_invalid(&self, id: u32) -> bool {
        self.regs_by_id.get(&id).is_none()
    }

    pub fn get_by_id(&self, id: u32) -> Option<&Vec<HpdRegistration>> {
        self.regs_by_id.get(&id)
    }
}
