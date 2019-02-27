extern crate chrono;
extern crate clap;
extern crate csv;
extern crate failure;
extern crate hashids;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use chrono::{NaiveDate, NaiveDateTime};
use clap::{App, Arg};
use csv::{Reader, Writer};
use hashids::HashIds;
use serde::Deserializer;
use std::{fs, io};

const RESTRICTED_ZIP_CODES: [&'static str; 17] = [
    "036", "692", "878", "059", "790", "879", "063", "821", "884", "102", "823", "890", "203",
    "830", "893", "556", "831",
];
const NULL_ZIP_CODE: &'static str = "000";

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Data Analysis Action Group")
        .arg(
            Arg::with_name("DIRECTORY")
                .help("The directory containing the purchases csvs")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("SALT")
                .help("The salt to use for id hashing")
                .required(true)
                .index(2),
        )
        .get_matches();
    let mut writer = Writer::from_writer(io::stdout());
    let hash_ids = HashIds::new_with_salt(matches.value_of("SALT").unwrap().to_string())?;
    for entry in fs::read_dir(matches.value_of("DIRECTORY").unwrap())? {
        let mut reader = Reader::from_path(entry?.path())?;
        for result in reader.deserialize() {
            let mut record: Record = result?;
            record.clean(&hash_ids);
            writer.serialize(record)?;
            break;
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    person_id: PersonId,
    postal_code: String,
    product_id: u64,
    event_id: Option<u64>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    product: String,
    event: String,
    division: String,
    registration_status: String,
    total_cost: f64,
    total_paid: f64,
    total_paid_refund: f64,
    total_paid_waived: f64,
    status: String,
    #[serde(deserialize_with = "deserialize_uc_datetime")]
    processed_at: NaiveDateTime,
    quantity: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum PersonId {
    Dirty(i64),
    Clean(String),
}

impl Record {
    fn clean(&mut self, hash_ids: &HashIds) {
        match &self.person_id {
            PersonId::Dirty(n) => self.person_id = PersonId::Clean(hash_ids.encode(&vec![*n])),
            PersonId::Clean(_) => {}
        }
        self.postal_code = self.postal_code.chars().take(3).collect::<String>();
        if RESTRICTED_ZIP_CODES.iter().any(|&s| s == self.postal_code) {
            self.postal_code = NULL_ZIP_CODE.to_string();
        }
    }
}

fn deserialize_uc_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
}
