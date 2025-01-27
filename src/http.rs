use simple_log::*;

/// Wrapper to make a quick request and get body
pub async fn quick_request(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Making a quick request to: {}", url);
    let client = reqwest::Client::new();
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
