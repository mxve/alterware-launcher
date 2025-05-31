use crate::global::CDN_HOSTS;
use crate::http;
use futures::future::join_all;
use simple_log::*;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

static CURRENT_CDN: Mutex<Option<Arc<Server>>> = Mutex::new(None);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Region {
    Europe,
    Global,
}

#[derive(Clone, Copy, Debug)]
pub struct Server {
    pub host: &'static str,
    pub rating: u8,
    pub latency: Option<std::time::Duration>,
    pub region: Region,
}

impl Server {
    pub const fn new(host: &'static str, region: Region) -> Self {
        Server {
            host,
            rating: 255,
            latency: None,
            region,
        }
    }

    pub fn url(&self) -> String {
        format!("https://{}/", self.host)
    }

    async fn rate(&mut self, asn: u32, is_initial: bool) {
        let timeout = if is_initial {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(5000)
        };
        match http::rating_request(&self.url(), timeout).await {
            Ok((latency, is_cloudflare)) => {
                self.latency = Some(latency);
                // Always use complete rating calculation with all available information
                self.rating = self.calculate_rating(latency, is_cloudflare, asn);

                info!(
                    "Server {} rated {} ({}ms, rating: {}, cloudflare: {}, region: {:?})",
                    self.host,
                    self.rating,
                    latency.as_millis(),
                    self.rating,
                    is_cloudflare,
                    self.region
                );
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", self.host, e);
                self.rating = 0;
                self.latency = None;
            }
        }
    }

    fn calculate_initial_rating(&self, latency: std::time::Duration) -> u8 {
        let mut rating: f32 = 255.0;
        let ms = latency.as_millis() as f32;
        let latency_mult = (200.0 / ms.max(200.0)).powf(0.5);
        rating *= latency_mult;
        rating.clamp(1.0, 255.0) as u8
    }

    fn calculate_rating(&self, latency: std::time::Duration, is_cloudflare: bool, asn: u32) -> u8 {
        let mut rating = self.calculate_initial_rating(latency);

        // Additional factors for full rating
        if is_cloudflare {
            // 3320/DTAG: bad cf peering
            // 5483/Magyar Telekom: sub. of DTAG
            if asn == 3320 || asn == 5483 {
                rating = (rating as f32 * 0.1) as u8;
            }
        }

        if self.region == Region::Global {
            rating = (rating as f32 * 1.1).min(255.0) as u8;
        }

        rating
    }
}

pub struct Hosts {
    pub servers: Vec<Server>,
    pub active_index: RwLock<Option<usize>>,
}

impl Hosts {
    /// create new rated hosts instance
    pub async fn new() -> Self {
        let mut hosts = Hosts {
            servers: CDN_HOSTS.to_vec(),
            active_index: RwLock::new(None),
        };

        let asn = crate::http::get_asn().await;
        hosts.rate(asn, true).await;

        if hosts.servers.iter().all(|server| server.rating == 0) {
            info!("All CDN servers failed with 500ms timeout, retrying with 5000ms timeout");
            hosts.rate(asn, false).await;
        }

        hosts
    }

    /// get the URL of the currently active CDN
    pub fn active_url(&self) -> Option<String> {
        CURRENT_CDN.lock().unwrap().as_ref().map(|s| s.url())
    }

    /// set the next best host based on ratings
    pub fn next(&self) -> bool {
        if self.servers.is_empty() {
            return false;
        }

        // find best host by rating, fifo if equal
        if let Some((idx, _)) = self
            .servers
            .iter()
            .enumerate()
            .max_by_key(|(idx, server)| (server.rating, -(*idx as i32)))
        {
            let server = &self.servers[idx];
            *CURRENT_CDN.lock().unwrap() = Some(Arc::new(*server));
            *self.active_index.write().unwrap() = Some(idx);
            true
        } else {
            false
        }
    }

    /// rate and order all servers, then select the best one
    pub async fn rate(&mut self, asn: u32, is_initial: bool) {
        let rating_futures: Vec<_> = self
            .servers
            .iter_mut()
            .map(|server| server.rate(asn, is_initial))
            .collect();

        join_all(rating_futures).await;

        // reset state and select best host
        *self.active_index.write().unwrap() = None;
        *CURRENT_CDN.lock().unwrap() = None;
        self.next();
    }

    /// Get the best CDN URL for use
    pub fn get_master_url(&self) -> Option<String> {
        self.active_url()
    }
}
