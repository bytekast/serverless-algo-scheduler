# Serverless Algorithm Scheduler


This demo project shows how to build and deploy a "real-world" Serverless application. It demonstrates how to build a microservice that implement the following apis and features:

### API Endpoint `POST /user/{username}/schedules`

This api endpoint accepts a payload that is persisted in a table in [DynamoDB](https://aws.amazon.com/dynamodb/). Here is a sample input:
```json
{
  "algorithm": "rowell/Hello/0.2.0",
  "apikey": "simXXXXXXXXXXXXXXXX",
  "input": "test input 123",
  "run_interval_minutes": 5,
  "enabled": true
}
```

```bash
curl -i -H "Content-Type: application/json" -X POST https://xxxxxxx.execute-api.us-east-1.amazonaws.com/dev/user/rowell/schedules -d '{ "algorithm": "rowell/Hello/0.2.0", "apikey": "simXXXXXXXXXXX", "input": "test input 123", "run_interval_minutes": 5, "enabled": false}'
```

Essentially, this payload creates a job to run an algorithm in the [Algorithmia Marketplace](https://algorithmia.com/) every 5 minutes with the provided `input` text.

### API Endpoint `GET /user/{username}/schedules`

Returns the schedules created by the user.

```bash
curl -X GET https://xxxxxxx.execute-api.us-east-1.amazonaws.com/dev/user/rowell/schedules
```


### Scheduler Background Job

This function wakes up every minute, queries [DynamoDB](https://aws.amazon.com/dynamodb/) for schedules that need to be run at this moment and sends the jobs to a Job [SQS](https://aws.amazon.com/sqs/) queue.


### Runner Event Listener


This function subscribes to the Job [SQS queue](https://aws.amazon.com/sqs/) and runs whenever a message is inserted. It receives the event, calls the [Algorithmia algorithm](https://algorithmia.com/developers/algorithm-development/algorithm-basics/your-first-algo), waits for the response and sends the result to a Result [SQS Queue](https://aws.amazon.com/sqs/).


## 📦 Prerequisites

Install [Rust](https://www.rust-lang.org/tools/install).

Install [Node](https://www.npmjs.com/get-npm).

Install the [serverless framework](https://serverless.com/framework/) cli.

Install [Docker](https://docs.docker.com/install/). (Docker is only required to build native binaries compatible with [AWS Lambda](https://aws.amazon.com/lambda/))

Then then run the following in your terminal

```bash
sls plugin install --name serverless-pseudo-parameters

sls plugin install --name serverless-rust
```


You also need to setup your AWS credentials/profiles in the `~/.aws/credentials` file.

```
[dev]
aws_access_key_id = XXXXXXXXXXXXXX
aws_secret_access_key = XXXXXXXXXXXXXX
region = us-east-1


[prod]
aws_access_key_id = XXXXXXXXXXXXXX
aws_secret_access_key = XXXXXXXXXXXXXX
region = us-east-1
```


## 🦀 Build

Run `cargo build`


## 🛵 Deploy

First, make sure docker is running.

To deploy, simply run `sls deploy`.

To deploy in a specific stage, provide the stage parameter `sls deploy --stage prod`.

When finished, you should see a similar output:

```bash
Serverless: Stack update finished...
Service Information
service: algo-scheduler
stage: dev
region: us-east-1
stack: algo-scheduler-dev
resources: 35
api keys:
  algo-scheduler-dev: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
endpoints:
  GET - https://xxxxxxxxxx.execute-api.us-east-1.amazonaws.com/dev/user/{username}/schedules
  POST - https://xxxxxxxxxx.execute-api.us-east-1.amazonaws.com/dev/user/{username}/schedules
functions:
  get-user-schedules: algo-scheduler-dev-get-user-schedules
  save-user-schedule: algo-scheduler-dev-save-user-schedule
  scheduler: algo-scheduler-dev-scheduler
  authorizer: algo-scheduler-dev-authorizer
  runner: algo-scheduler-dev-runner
layers:
  None
```
