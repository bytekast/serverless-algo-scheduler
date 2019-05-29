extern crate models;
extern crate reqwest;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde_json::Value;

use models::Schedule;
use rusoto_core::Region;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use std::env;

fn main() {
    lambda!(router);
}

fn router(event: Value, context: Context) -> Result<Value, HandlerError> {
    let function = env::var("FUNCTION").expect("Unknown function");
    match function.as_ref() {
        "scheduler" => scheduler(event, context),
        _ => Ok(Value::Null), // do nothing
    }
}

fn scheduler(event: Value, _: Context) -> Result<Value, HandlerError> {
    println!("{:?}", event);
    let schedules_to_run = Schedule::get_enabled_schedules();
    println!("Schedules to Run: {}", schedules_to_run.len());
    for schedule in schedules_to_run {
        match serde_json::to_string(&schedule) {
            Ok(json) => {
                let job_sqs_url =
                    env::var("JOB_QUEUE_URL").expect("env variable JOB_QUEUE_URL not found");
                send_sqs(job_sqs_url.to_owned(), json)
            }
            Err(error) => println!("{}", error.to_string()),
        }
    }

    Ok(Value::Null)
}

fn send_sqs(sqs_url: String, message: String) {
    let client = SqsClient::new(Region::UsEast1);
    let request = SendMessageRequest {
        message_body: message,
        queue_url: sqs_url,
        ..Default::default()
    };
    println!("{:?}", client.send_message(request).sync().ok());
}
