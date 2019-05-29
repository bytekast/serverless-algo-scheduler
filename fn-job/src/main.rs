extern crate models;
extern crate reqwest;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde_json::Value;

use aws_lambda_events::event::sqs::SqsEvent;
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
        "runner" => runner(event, context),
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

fn runner(event: Value, _: Context) -> Result<Value, HandlerError> {
    let sqs_event: SqsEvent = serde_json::from_str(event.to_string().as_ref()).unwrap();
    println!("{:?}", sqs_event);
    let result_sqs_url =
        env::var("RESULT_QUEUE_URL").expect("env variable RESULT_QUEUE_URL not found");
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("Unable to create client");
    for record in sqs_event.records {
        match record.body {
            Some(body) => {
                let schedule: Schedule = serde_json::from_str(body.as_ref()).unwrap();
                let url = format!(
                    "https://api.test.algorithmia.com/v1/algo/{}",
                    schedule.algorithm
                );
                let res = client
                    .post(&url)
                    .header("Content-Type", "text/plain")
                    .header("Authorization", schedule.apikey)
                    .body(schedule.input)
                    .send();
                match res {
                    Ok(mut res) => {
                        let respose_text = res.text().ok().unwrap();
                        println!("{:?}", respose_text);
                        send_sqs(result_sqs_url.to_owned(), respose_text)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            None => (),
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
