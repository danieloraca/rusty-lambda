use lambda_runtime::{tracing, Error};
use serde_json::Value;
use url::Url;

pub fn original_image_info(payload: &Value) -> Result<(String, String), Error> {
    tracing::info!("Payload: {}", payload);

    let body_json: Value = serde_json::from_str(
        payload["body"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid body"))?,
    )
    .map_err(|e| -> Error { anyhow::anyhow!("Error parsing JSON: {}", e).into() })?;

    tracing::info!("Body JSON: {}", body_json);

    let image_url: &str = body_json["image_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No image url found in payload"))?;
    tracing::info!("Image URL: {}", image_url);

    let image_new_size: &str = body_json["image_new_size"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No image new size found in payload"))?;
    tracing::info!("Image new size: {}", image_new_size);

    return Ok((image_url.to_string(), image_new_size.to_string()));
}

pub fn get_bucket_name() -> String {
    let bucket_name: String =
        std::env::var("THE_BUCKET_NAME").expect("THE_BUCKET_NAME must be set");
    tracing::info!("Bucket name: {}", bucket_name);

    return bucket_name;
}

pub fn get_image_name(image_url: &str) -> Result<String, Error> {
    let image_name: String = match Url::parse(image_url) {
        Ok(parsed_url) => {
            if let Some(name) = parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
            {
                name.to_string()
            } else {
                return Err("No image name found in the URL.".into());
            }
        }
        Err(e) => {
            return Err(format!("Failed to parse URL: {}", e).into());
        }
    };

    return Ok(image_name);
}
