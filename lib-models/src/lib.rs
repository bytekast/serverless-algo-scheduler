#[macro_use]
extern crate maplit;

use chrono::prelude::*;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, QueryInput, ScanInput};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Schedule {
    pub id: Option<Uuid>,
    pub enabled: Option<bool>,
    pub run_interval_minutes: u32,
    pub apikey: String,
    pub algorithm: String,
    pub input: String,
    pub username: Option<String>,
}

#[allow(dead_code)]
impl Schedule {
    pub fn to_schedule(item: &HashMap<String, AttributeValue, RandomState>) -> Schedule {
        Schedule {
            id: item
                .get("id")
                .and_then(|x| x.s.as_ref())
                .map(|x| Uuid::parse_str(&x).expect("error parsing id")),
            enabled: item.get("enabled").and_then(|x| x.bool.to_owned()),
            run_interval_minutes: item
                .get("run_interval_minutes")
                .and_then(|x| x.n.as_ref())
                .map(|x| {
                    x.parse::<u32>()
                        .expect("error parsing run_interval_minutes")
                })
                .expect("missing required run_interval_minutes"),
            apikey: item
                .get("apikey")
                .and_then(|x| x.s.to_owned())
                .expect("missing required apikey"),
            algorithm: item
                .get("algorithm")
                .and_then(|x| x.s.to_owned())
                .expect("missing required algorithm"),
            input: item
                .get("input")
                .and_then(|x| x.s.to_owned())
                .expect("missing required input"),
            username: item.get("username").and_then(|x| x.s.to_owned()),
        }
    }

    pub fn from_schedule(schedule: Schedule) -> HashMap<String, AttributeValue> {
        hashmap! {
            String::from("username") =>  AttributeValue { s: schedule.username, ..Default::default() },
            String::from("id") => AttributeValue { s: schedule.id.map(|uid| uid.to_string()), ..Default::default() },
            String::from("enabled") =>  AttributeValue { bool: schedule.enabled, ..Default::default() },
            String::from("run_interval_minutes") =>  AttributeValue { n: Some(schedule.run_interval_minutes.to_string()), ..Default::default() },
            String::from("apikey") =>  AttributeValue { s: Some(schedule.apikey), ..Default::default() },
            String::from("algorithm") =>  AttributeValue { s: Some(schedule.algorithm), ..Default::default() },
            String::from("input") =>  AttributeValue { s: Some(schedule.input), ..Default::default() },
        }
    }

    pub fn get_schedules_to_run() -> Vec<Schedule> {
        let attribute_map = hashmap! {
            String::from(":enabled") => AttributeValue { bool: Some(true), ..Default::default() }
        };

        let scan_input = ScanInput {
            consistent_read: Some(true),
            filter_expression: Some(String::from("enabled = :enabled")),
            expression_attribute_values: Some(attribute_map),
            table_name: env::var("USER_SCHEDULE_TABLE_NAME")
                .expect("env variable USER_SCHEDULE_TABLE_NAME not found"),
            ..Default::default()
        };

        let client = DynamoDbClient::new(Region::UsEast1);
        let minutes = Local::now().minute();
        match client.scan(scan_input).sync() {
            Ok(output) => match output.items {
                None => vec![],
                Some(items) => items
                    .iter()
                    .map(Schedule::to_schedule)
                    .filter(|x| minutes % x.run_interval_minutes == 0)
                    .collect(),
            },
            Err(error) => {
                println!("{}", error.to_string());
                vec![]
            }
        }
    }

    pub fn get_schedules_by_username(username: String) -> Vec<Schedule> {
        let attribute_map = hashmap! {
            String::from(":username") => AttributeValue { s: Some(username), ..Default::default() }
        };

        let query_input = QueryInput {
            consistent_read: Some(true),
            key_condition_expression: Some(String::from("username = :username")),
            expression_attribute_values: Some(attribute_map),
            table_name: env::var("USER_SCHEDULE_TABLE_NAME")
                .expect("env variable USER_SCHEDULE_TABLE_NAME not found"),
            ..Default::default()
        };

        let client = DynamoDbClient::new(Region::UsEast1);
        match client.query(query_input).sync() {
            Ok(output) => match output.items {
                None => vec![],
                Some(items) => items.iter().map(Schedule::to_schedule).collect(),
            },
            Err(error) => {
                println!("{}", error.to_string());
                vec![]
            }
        }
    }
}
