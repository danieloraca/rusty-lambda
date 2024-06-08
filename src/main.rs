use aws_config;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use reqwest::get;
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
    let payload: Value = event.payload;

    let body_json: Value = serde_json::from_str(
        payload["body"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid body"))?,
    )
    .map_err(|e| -> Error { anyhow::anyhow!("Error parsing JSON: {}", e).into() })?;

    let image_url = body_json["image_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No image url found in payload"))?;

    let image_new_size = body_json["image_new_size"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No image new size found in payload"))?;

    let bucket_name = std::env::var("THE_BUCKET_NAME")?;
    let region = std::env::var("THE_REGION")?;
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let resp = get(image_url).await?;
    let image_bytes = resp.bytes().await?;

    let file_name = "uploaded_image.jpg";
    client
        .put_object()
        .bucket(&bucket_name)
        .key(file_name)
        .body(ByteStream::from(image_bytes.to_vec()))
        .send()
        .await?;

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Uploaded image {}", image_url),
    };

    let resp_json = serde_json::to_string(&resp)?;

    tracing::info!("Response in CloudWatch: {}", resp_json);

    Ok(serde_json::json!({
        "statusCode": 200,
        "body": resp_json,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    tracing::info!("Starting the function");

    run(service_fn(function_handler)).await
}
