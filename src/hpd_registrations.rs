use std::collections::HashMap;
use std::error::Error;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct HpdRegistration {
    #[serde(alias = "RegistrationID")]
    reg_id: u32,

    #[serde(alias = "BoroID")]
    boro: u8,
}

pub struct HpdRegistrationMap {
    registrations: HashMap<u32, HpdRegistration>,
}

impl HpdRegistrationMap {
    pub fn from_csv<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Box<dyn Error>> {
        let registrations = HashMap::new();

        for result in rdr.deserialize() {
            let _record: HpdRegistration = result?;
        }

        Ok(HpdRegistrationMap {
            registrations
        })
    }
}
