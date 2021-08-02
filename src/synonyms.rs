use std::collections::HashMap;
use std::rc::Rc;

fn get_synonyms() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![(
        "PINNACLE",
        vec![
            "DAVID ROSE",
            "EDDIE LJESNJANIN",
            "EDWARD SUAZO",
            "MARC BARHORIN",
            "ABDIN RADONCIC",
            "ABIDIN RADONCIC",
            "DAVID RADONCIC",
            "DAVID RADONIC",
            "ELINOR ARZT",
            "RASIM TOSKIC",
        ],
    )]
}

pub struct Synonyms {
    map: HashMap<String, Rc<String>>,
}

impl Synonyms {
    pub fn new() -> Self {
        let mut map: HashMap<String, Rc<String>> = HashMap::new();

        for (str_canonical, synonyms) in get_synonyms() {
            let canonical = Rc::new(str_canonical.to_string());
            for synonym in synonyms {
                map.insert(synonym.to_string(), Rc::clone(&canonical));
            }
        }

        Synonyms { map }
    }

    pub fn get(&self, value: &String) -> Option<Rc<String>> {
        match self.map.get(value) {
            Some(value) => Some(Rc::clone(&value)),
            None => None,
        }
    }
}
