extern crate chrono;
extern crate clap;
extern crate csv;
#[macro_use]
extern crate failure;
extern crate hashids;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use clap::{App, Arg};
use csv::{Reader, Writer};
use hashids::HashIds;
use serde::Deserializer;
use std::collections::HashMap;
use std::{fs, io};

const RESTRICTED_ZCTAS: [&'static str; 17] = [
    "036", "692", "878", "059", "790", "879", "063", "821", "884", "102", "823", "890", "203",
    "830", "893", "556", "831",
];
const NULL_ZCTA: &'static str = "000";

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Data Analysis Action Group")
        .arg(
            Arg::with_name("PEOPLE")
                .help("The csv file containing the people")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PURCHASES")
                .help("The directory containing the purchases csvs")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("SALT")
                .help("The salt to use for id hashing")
                .required(true)
                .index(3),
        )
        .get_matches();
    let mut writer = Writer::from_writer(io::stdout());
    let mut reader = Reader::from_path(matches.value_of("PEOPLE").unwrap())?;
    let people = reader.deserialize().collect::<Result<Vec<Person>, _>>()?;
    let record_builder = RecordBuilder::new(people, matches.value_of("SALT").unwrap())?;
    for entry in fs::read_dir(matches.value_of("PURCHASES").unwrap())? {
        let mut reader = Reader::from_path(entry?.path())?;
        for result in reader.deserialize() {
            let purchase: Purchase = result?;
            match record_builder.with_purchase(purchase) {
                Ok(record) => writer.serialize(record)?,
                Err(err) => eprintln!("skipping purchase: {}", err),
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Purchase {
    person_id: i64,
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

#[derive(Debug, Deserialize)]
struct Person {
    id: i64,
    birth_date: Option<NaiveDate>,
    gender: String,
    postal_code: String,
}

#[derive(Debug, Serialize)]
struct Record {
    person_id: String,
    gender: String,
    birth_year: Option<i32>,
    zcta: String,
    product_id: u64,
    product: String,
    event_id: Option<u64>,
    event: String,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    division: String,
    registration_status: String,
    total_cost: f64,
    total_paid: f64,
    total_paid_refund: f64,
    total_paid_waived: f64,
    status: String,
    processed_at: NaiveDateTime,
    quantity: u64,
}

#[derive(Debug)]
struct RecordBuilder {
    hash_ids: HashIds,
    people: HashMap<i64, Person>,
}

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "missing person with id: {}", id)]
    MissingPerson { id: i64 },
}

impl RecordBuilder {
    fn new(people: Vec<Person>, salt: &str) -> Result<RecordBuilder, failure::Error> {
        Ok(RecordBuilder {
            people: people
                .into_iter()
                .map(|person| (person.id, person))
                .collect(),
            hash_ids: HashIds::new_with_salt(salt.to_string())?,
        })
    }

    fn with_purchase(&self, purchase: Purchase) -> Result<Record, Error> {
        let person = self
            .people
            .get(&purchase.person_id)
            .ok_or(Error::MissingPerson {
                id: purchase.person_id,
            })?;
        Ok(Record {
            person_id: self.hash_ids.encode(&vec![person.id]),
            gender: person.gender.clone(),
            birth_year: person.birth_date.map(|date| date.year()),
            zcta: postal_code_to_zcta(&person.postal_code),
            product_id: purchase.product_id,
            product: purchase.product,
            event_id: purchase.event_id,
            event: purchase.event,
            start: purchase.start,
            end: purchase.end,
            division: purchase.division,
            registration_status: purchase.registration_status,
            total_cost: purchase.total_cost,
            total_paid: purchase.total_paid,
            total_paid_refund: purchase.total_paid_refund,
            total_paid_waived: purchase.total_paid_waived,
            status: purchase.status,
            processed_at: purchase.processed_at,
            quantity: purchase.quantity,
        })
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

fn postal_code_to_zcta(postal_code: &str) -> String {
    let mut zcta = postal_code.chars().take(3).collect::<String>();
    if RESTRICTED_ZCTAS
        .iter()
        .any(|&restricted| zcta == restricted)
    {
        zcta = NULL_ZCTA.to_string();
    }
    zcta
}
