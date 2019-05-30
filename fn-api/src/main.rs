extern crate serde;
extern crate serde_json;
extern crate uuid;

extern crate models;

use std::env;

use lambda_http::http::StatusCode;
use lambda_http::{lambda, Body, IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};

use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput};
use serde_json::json;
use uuid::Uuid;

use models::Schedule;

fn main() {
    lambda!(router)
}

fn router(request: Request, context: Context) -> Result<impl IntoResponse, HandlerError> {
    println!("{:?}", request);
    match (request.method().as_str(), request.uri().path()) {
        ("GET", _) => get_user_schedules(request, context),
        ("POST", _) => save_user_schedules(request, context),
        _ => not_allowed(request, context),
    }
}

fn not_allowed(_req: Request, _c: Context) -> Result<Response<Body>, HandlerError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from(()))
        .expect("err creating response"))
}

fn save_user_schedules(request: Request, _: Context) -> Result<Response<Body>, HandlerError> {
    let username = request.path_parameters().get("username").map(String::from);
    match serde_json::from_slice::<Schedule>(request.body().as_ref()) {
        Ok(mut schedule) => match schedule.run_interval_minutes {
            1...60 => {
                schedule.id = Some(Uuid::new_v4());
                schedule.username = username.clone();
                schedule.enabled = schedule.enabled.or(Some(true));

                let client = DynamoDbClient::new(Region::UsEast1);
                let item = Schedule::from_schedule(schedule.to_owned());

                let input = PutItemInput {
                    table_name: env::var("USER_SCHEDULE_TABLE_NAME")
                        .expect("env variable USER_SCHEDULE_TABLE_NAME not found"),
                    item,
                    ..Default::default()
                };

                match client.put_item(input).sync() {
                    Ok(_) => Ok(Response::builder()
                        .status(StatusCode::CREATED)
                        .body(serde_json::json!(schedule).to_string().into())
                        .expect("err creating response")),
                    Err(error) => Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(error.to_string().into())
                        .expect("err creating response")),
                }
            }
            _ => Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Valid run interval minutes are 1 to 60".into())
                .expect("err creating response")),
        },
        Err(e) => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.to_string().into())
            .expect("err creating response")),
    }
}

fn get_user_schedules(request: Request, _: Context) -> Result<Response<Body>, HandlerError> {
    println!("{:?}", request);
    let username = request.path_parameters().get("username").map(String::from);
    Ok(json!(Schedule::get_schedules_by_username(username.unwrap())).into_response())
}
