mod util;

use std::str::Bytes;

use async_stream::stream;
use aws_config::{self, BehaviorVersion};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use futures_util::StreamExt;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use reqwest::get;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct LambdaResponse {
    req_id: String,
    msg: String,
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let payload: Value = event.payload;

    let (image_url, _image_new_size) = util::original_image_info(&payload)?;
    let bucket_name: String = util::get_bucket_name();
    let image_name: String = util::get_image_name(&image_url)?;

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);
    tracing::info!("Client created");

    let response = get(&image_url).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                client
                    .put_object()
                    .bucket(&bucket_name)
                    .key(&image_name)
                    .body(ByteStream::from(chunk.to_vec()))
                    .send()
                    .await?;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    // let mut stream = get(image_url).await?.bytes_stream();
    // while let Some(chunk) = stream.next().await {
    //     match chunk {
    //         Ok(chunk) => {
    //             client
    //                 .put_object()
    //                 .bucket(&bucket_name)
    //                 .key(image_name)
    //                 .body(ByteStream::from(chunk.to_vec()))
    //                 .send()
    //                 .await?;
    //         }
    //         Err(e) => {
    //             return Err(e.into());
    //         }
    //     }
    // }

    // let s = stream! {
    //     let mut body = resp.bytes_stream();
    //     while let Some(chunk) = body.next().await {
    //         match chunk {
    //             Ok(chunk) => yield chunk,
    //             Err(e) => yield Err(e.into()),
    //         }
    //     }
    // };

    // let image_bytes = resp.bytes().await?;

    // client
    //     .put_object()
    //     .bucket(&bucket_name)
    //     .key(image_name)
    //     .body(ByteStream::from(image_bytes.to_vec()))
    //     .send()
    //     .await?;

    // Prepare the response
    let resp = LambdaResponse {
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
