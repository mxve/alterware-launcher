use reqwest::header::HeaderMap;
use serde_json::Value;
use simple_log::*;
use std::time::{Duration, Instant};

/// Wrapper to make a quick request and get body
pub async fn quick_request(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Making a quick request to: {}", url);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(url)
        .header("User-Agent", crate::global::USER_AGENT.to_string())
        .send()
        .await;

    if let Err(e) = &res {
        error!("Failed to get {}: {}", url, e);
        return Err(format!("Failed to get {} {}", url, e).into());
    }

    let res = res.unwrap();
    match res.text().await {
        Ok(text) => {
            info!("Successfully received response from: {}", url);
            Ok(text)
        }
        Err(e) => {
            warn!("Failed to get response text from {}: {}", url, e);
            Err(e.into())
        }
    }
}

/// Check if server is using Cloudflare based on headers
fn is_cloudflare(headers: &HeaderMap) -> bool {
    headers.contains_key("cf-ray")
        || headers.contains_key("cf-cache-status")
        || headers
            .get("server")
            .is_some_and(|v| v.as_bytes().starts_with(b"cloudflare"))
}

/// Make a request for rating purposes, measuring latency and detecting Cloudflare
pub async fn rating_request(
    url: &str,
    timeout: Duration,
) -> Result<(Duration, bool), Box<dyn std::error::Error>> {
    info!(
        "Making a rating request to: {} with timeout {:?}",
        url, timeout
    );
    let client = reqwest::Client::builder().timeout(timeout).build()?;

    let start = Instant::now();
    let res = client
        .get(url)
        .header("User-Agent", crate::global::USER_AGENT.to_string())
        .send()
        .await;

    let latency = start.elapsed();

    if let Err(e) = &res {
        error!("Failed to get {}: {} (after {:?})", url, e, latency);
        return Err(format!("Failed to get {} {} (after {:?})", url, e, latency).into());
    }

    let res = res.unwrap();
    let headers = res.headers().clone();
    let is_cloudflare = is_cloudflare(&headers);

    // We don't need the response body for rating
    if let Err(e) = res.text().await {
        warn!(
            "Failed to get response text from {}: {} (after {:?})",
            url, e, latency
        );
        return Err(e.into());
    }

    info!(
        "Successfully rated {} in {:?} (cloudflare: {})",
        url, latency, is_cloudflare
    );
    Ok((latency, is_cloudflare))
}

/// Retrieve client ASN and region
pub async fn get_location_info() -> (u32, String) {
    let response = quick_request(crate::global::IP2ASN).await;
    if let Ok(as_data_str) = response {
        if let Ok(as_data) = serde_json::from_str::<Value>(&as_data_str) {
            let as_number = as_data
                .get("as_number")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let region = as_data
                .get("region")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            return (as_number, region);
        }
    }
    (0, "Unknown".to_string())
}
