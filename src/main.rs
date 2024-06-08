use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // Extract some useful info from the request
    let payload = event.payload;

    let body_json: Value = serde_json::from_str(
        payload["body"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid body"))?,
    )
    .map_err(|e| -> Error { anyhow::anyhow!("Error parsing JSON: {}", e).into() })?;

    let image_url = body_json["image_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No image url found in payload"))?;

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Hello danstack with command {}.", image_url),
    };

    let resp_json = serde_json::to_string(&resp)?;

    Ok(serde_json::json!({
        "statusCode": 200,
        "body": resp_json,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
