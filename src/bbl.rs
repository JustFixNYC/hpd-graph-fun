use std::convert::TryFrom;

// https://en.wikipedia.org/wiki/Borough,_Block_and_Lot

#[repr(u8)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Boro {
    Manhattan = 1,
    Bronx = 2,
    Brooklyn = 3,
    Queens = 4,
    StatenIsland = 5,
}

impl TryFrom<u8> for Boro {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Boro::Manhattan),
            2 => Ok(Boro::Bronx),
            3 => Ok(Boro::Brooklyn),
            4 => Ok(Boro::Queens),
            5 => Ok(Boro::StatenIsland),
            _ => Err("Invalid boro ID"),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct BBL {
    boro: Boro,
    block: u32,
    lot: u16,
}

impl BBL {
    pub fn from_numbers(boro: u8, block: u32, lot: u16) -> Result<BBL, &'static str> {
        Ok(BBL {
            boro: Boro::try_from(boro)?,
            block,
            lot,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}{:0>5}{:0>4}", self.boro as u8, self.block, self.lot)
    }
}

#[test]
fn test_to_string_works() {
    assert_eq!(
        BBL {
            boro: Boro::StatenIsland,
            block: 1,
            lot: 2
        }
        .to_string(),
        "5000010002".to_owned()
    );

    assert_eq!(
        BBL {
            boro: Boro::Queens,
            block: 12345,
            lot: 6789
        }
        .to_string(),
        "4123456789".to_owned()
    );
}
