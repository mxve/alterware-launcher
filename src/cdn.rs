use crate::global::CDN_HOSTS;
use crate::http;
use futures::future::join_all;
use simple_log::*;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

static CURRENT_CDN: Mutex<Option<Arc<Server>>> = Mutex::new(None);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Region {
    Africa,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
    Global,
    Unknown,
}

impl Region {
    pub fn from_str(region_str: &str) -> Self {
        match region_str {
            "Africa" => Region::Africa,
            "Asia" => Region::Asia,
            "Europe" => Region::Europe,
            "NorthAmerica" => Region::NorthAmerica,
            "Oceania" => Region::Oceania,
            "SouthAmerica" => Region::SouthAmerica,
            _ => Region::Unknown,
        }
    }

    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match self {
            Region::Europe => Some((54.0, 15.0)),
            Region::Asia => Some((35.0, 105.0)),
            Region::NorthAmerica => Some((45.0, -100.0)),
            Region::SouthAmerica => Some((-15.0, -60.0)),
            Region::Africa => Some((0.0, 20.0)),
            Region::Oceania => Some((-25.0, 140.0)),
            Region::Global => None,
            Region::Unknown => None,
        }
    }

    pub fn distance_to(&self, other: Region) -> f64 {
        if *self == Region::Global || other == Region::Global {
            return 0.0;
        }

        if *self == other {
            return 0.0;
        }

        let (lat1, lon1) = match self.coordinates() {
            Some(coords) => coords,
            None => return 20000.0,
        };

        let (lat2, lon2) = match other.coordinates() {
            Some(coords) => coords,
            None => return 20000.0,
        };

        // haversine
        let r = 6371.0;
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
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

    async fn rate(&mut self, asn: u32, user_region: Region, is_initial: bool) {
        let timeout = if is_initial {
            Duration::from_millis(1000)
        } else {
            Duration::from_millis(5000)
        };
        match http::rating_request(&self.url(), timeout).await {
            Ok((latency, is_cloudflare)) => {
                self.latency = Some(latency);
                self.rating = self.calculate_rating(latency, is_cloudflare, asn, user_region);

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

    fn rate_latency(&self, latency: std::time::Duration) -> u8 {
        let ms = latency.as_millis() as f32;

        let rating = if ms <= 50.0 {
            240.0
        } else if ms <= 100.0 {
            240.0 - (ms - 50.0) * 1.0
        } else if ms <= 200.0 {
            190.0 - (ms - 100.0) * 0.5
        } else if ms <= 500.0 {
            140.0 - (ms - 200.0) * 0.033
        } else {
            100.0
        };

        rating.clamp(1.0, 255.0) as u8
    }

    fn calculate_rating(
        &self,
        latency: std::time::Duration,
        is_cloudflare: bool,
        asn: u32,
        user_region: Region,
    ) -> u8 {
        let mut rating = self.rate_latency(latency);

        // Additional factors for full rating
        if is_cloudflare {
            // 3320/DTAG: bad cf peering
            // 5483/Magyar Telekom: sub. of DTAG
            if asn == 3320 || asn == 5483 {
                rating = (rating as f32 * 0.1) as u8;
            }
        }

        let distance_km = user_region.distance_to(self.region);
        let region_multiplier = if self.region == Region::Global {
            1.4
        } else if distance_km == 0.0 {
            1.3
        } else if user_region == Region::Unknown {
            1.0
        } else if distance_km <= 2000.0 {
            1.25
        } else if distance_km <= 5000.0 {
            1.15
        } else if distance_km <= 10000.0 {
            1.05
        } else {
            1.0
        };

        rating = (rating as f32 * region_multiplier).min(255.0) as u8;

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

        let (asn, region_str) = crate::http::get_location_info().await;
        let user_region = Region::from_str(&region_str);
        info!(
            "Detected user region as {:?} (region: {})",
            user_region, region_str
        );

        hosts.rate(asn, user_region, true).await;

        if hosts.servers.iter().all(|server| server.rating == 0) {
            info!("All CDN servers failed with 1000ms timeout, retrying with 5000ms timeout");
            hosts.rate(asn, user_region, false).await;
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

        // find best host by rating, then by latency
        if let Some((idx, _)) = self.servers.iter().enumerate().max_by_key(|(_, server)| {
            (
                server.rating,
                server
                    .latency
                    .map_or(0, |l| u64::MAX - l.as_millis() as u64),
            )
        }) {
            let server = &self.servers[idx];
            *CURRENT_CDN.lock().unwrap() = Some(Arc::new(*server));
            *self.active_index.write().unwrap() = Some(idx);
            true
        } else {
            false
        }
    }

    /// rate and order all servers, then select the best one
    pub async fn rate(&mut self, asn: u32, user_region: Region, is_initial: bool) {
        let rating_futures: Vec<_> = self
            .servers
            .iter_mut()
            .map(|server| server.rate(asn, user_region, is_initial))
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

/// CDN rating function for --rate flag
pub async fn rate_cdns_and_display() {
    use colored::Colorize;

    let (asn, region_str) = crate::http::get_location_info().await;
    let user_region = Region::from_str(&region_str);

    if user_region == Region::Unknown {
        println!(
            "User region: {} (using Global server preference)",
            "Unknown".bright_red()
        );
    } else {
        println!("User region: {:?}", user_region);
    }
    println!("Rating CDNs...");

    let mut hosts = Hosts {
        servers: CDN_HOSTS.to_vec(),
        active_index: RwLock::new(None),
    };

    hosts.rate(asn, user_region, true).await;

    if hosts.servers.iter().all(|server| server.rating == 0) {
        println!("Retrying with longer timeout...");
        hosts.rate(asn, user_region, false).await;
    }

    println!();
    for server in hosts.servers.iter() {
        let latency_str = server
            .latency
            .map_or("timeout".to_string(), |l| format!("{} ms", l.as_millis()));

        println!(
            "{}: rating {}, latency {}",
            server.host.bright_white(),
            server.rating.to_string().bright_cyan(),
            latency_str
        );
    }

    // Show selected CDN
    if hosts.next() {
        if let Some(best_url) = hosts.active_url() {
            println!();
            println!("Selected: {}", best_url.bright_green());
        }
    } else {
        println!();
        println!("{}", "No available CDN servers".bright_red());
    }
}
