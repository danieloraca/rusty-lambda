use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let command = event.payload.command;
    println!("Incoming command: {}", command);

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Hello dan with command {}.", command),
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
