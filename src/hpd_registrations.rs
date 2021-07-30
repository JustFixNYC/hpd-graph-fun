use serde::Deserialize;
use std::error::Error;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
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

pub struct HpdRegistrationMap {
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let mut count = 0;

        for result in rdr.deserialize() {
            let r: RawHpdRegistration = result?;
            let reg_end_date = NaiveDate::parse_from_str(&r.reg_end_date.as_ref(), "%m/%d/%Y");
            if let Err(err) = reg_end_date {
                println!("Unable to parse registration end date: {}", err);
            }
            count += 1;
        }

        println!("Parsed {} registrations.", count);

        Ok(HpdRegistrationMap {})
    }
}
