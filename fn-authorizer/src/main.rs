use lambda_runtime::{error::HandlerError, lambda, Context};
use serde_derive::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct AuthorizationEvent {
    #[serde(rename = "methodArn")]
    method_arn: String,
}

fn main() {
    lambda!(handler)
}

fn handler(event: AuthorizationEvent, _: Context) -> Result<Value, HandlerError> {
    Ok(json!({
        "principalId": "anonymous", // TODO
        "policyDocument": {
            "Version": "2012-10-17",
            "Statement": {
                "Action": "execute-api:Invoke",
                "Effect": "Allow",
                "Resource": event.method_arn
            }
        },
        "context": {}
    }))
}
