use std::{io::Read, thread, time::Duration};

use anyhow::{Context, Result, anyhow};
use curl::easy::{Easy, List, ReadError};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

const UPDATE_INTERVAL: Duration = Duration::from_secs(600);

#[derive(Clone)]
struct Credentials {
    email: String,
    api_key: String,
    domain: String,
}

impl Credentials {
    fn from_env() -> Result<Self> {
        let email = std::env::var("CLOUDFLARE_EMAIL")
            .context("Set CLOUDFLARE_EMAIL to your Cloudflare account email")?;
        let api_key = std::env::var("CLOUDFLARE_API_KEY")
            .context("Set CLOUDFLARE_API_KEY to your Cloudflare API key")?;
        let domain = std::env::var("CLOUDFLARE_DOMAIN")
            .context("Set CLOUDFLARE_DOMAIN to the zone you want to update")?;

        Ok(Self {
            email,
            api_key,
            domain,
        })
    }
}

#[derive(Deserialize)]
struct ZoneResponse {
    result: Vec<Zone>,
}

#[derive(Deserialize)]
struct Zone {
    id: String,
}

#[derive(Deserialize, Clone)]
struct DnsRecord {
    id: String,
    content: String,
}

#[derive(Deserialize)]
struct RecordResponse {
    result: Vec<DnsRecord>,
}

#[derive(Serialize)]
struct UpdatePayload {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Deserialize)]
struct UpdateResponse {
    success: bool,
}

fn main() -> Result<()> {
    dotenv().ok();
    let creds = Credentials::from_env()?;

    loop {
        sync_dns(&creds)?;
        thread::sleep(UPDATE_INTERVAL);
    }
}

fn sync_dns(creds: &Credentials) -> Result<()> {
    let zone_id = fetch_zone_id(creds)?;
    let record = fetch_record(&zone_id, creds)?;
    let current_ip = fetch_public_ip()?;

    if record.content.trim() == current_ip {
        println!("No update required. IP: {}", current_ip);
        return Ok(());
    }

    println!(
        "Updating DNS from {} to {} for {}",
        record.content.trim(),
        current_ip,
        creds.domain
    );

    update_record(&zone_id, &record.id, &current_ip, creds)?;
    println!("DNS record updated.");
    Ok(())
}

fn fetch_zone_id(creds: &Credentials) -> Result<String> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones?name={}&status=active",
        creds.domain
    );

    let body = perform_get(&url, auth_headers(creds)?)?;
    let parsed: ZoneResponse =
        serde_json::from_slice(&body).context("Failed to parse zone lookup response")?;

    parsed
        .result
        .first()
        .map(|z| z.id.clone())
        .ok_or_else(|| anyhow!("Zone not found for domain {}", creds.domain))
}

fn fetch_record(zone_id: &str, creds: &Credentials) -> Result<DnsRecord> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=A&name={}",
        zone_id, creds.domain
    );

    let body = perform_get(&url, auth_headers(creds)?)?;
    let parsed: RecordResponse =
        serde_json::from_slice(&body).context("Failed to parse DNS record response")?;

    parsed
        .result
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("A record not found for domain {}", creds.domain))
}

fn fetch_public_ip() -> Result<String> {
    let mut response = Vec::new();
    let mut handle = Easy::new();
    handle.url("https://checkip.amazonaws.com")?;

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let ip = String::from_utf8(response)?;
    Ok(ip.trim().to_string())
}

fn update_record(zone_id: &str, record_id: &str, ip: &str, creds: &Credentials) -> Result<()> {
    let payload = UpdatePayload {
        kind: "A".to_string(),
        name: creds.domain.clone(),
        content: ip.to_string(),
        ttl: 1,
        proxied: false,
    };

    let body = serde_json::to_vec(&payload)?;
    let mut body_reader = body.as_slice();
    let mut response = Vec::new();

    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    let mut handle = Easy::new();
    handle.url(&url)?;
    handle.put(true)?;
    handle.http_headers(auth_headers(creds)?)?;
    handle.upload(true)?;
    handle.in_filesize(body.len() as u64)?;

    {
        let mut transfer = handle.transfer();
        // Map std::io::Error into curl's ReadError so the upload can abort cleanly.
        transfer.read_function(|into| body_reader.read(into).map_err(|_| ReadError::Abort))?;
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let update: UpdateResponse =
        serde_json::from_slice(&response).context("Failed to parse update response")?;

    if update.success {
        Ok(())
    } else {
        Err(anyhow!(
            "Cloudflare API reported failure: {}",
            String::from_utf8_lossy(&response)
        ))
    }
}

fn perform_get(url: &str, headers: List) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(url)?;
    handle.http_headers(headers)?;
    handle.get(true)?;

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|chunk| {
            data.extend_from_slice(chunk);
            Ok(chunk.len())
        })?;
        transfer.perform()?;
    }

    Ok(data)
}

fn auth_headers(creds: &Credentials) -> Result<List> {
    let mut list = List::new();
    list.append(&format!("X-Auth-Email: {}", creds.email))?;
    list.append(&format!("X-Auth-Key: {}", creds.api_key))?;
    list.append("Content-Type: application/json")?;
    Ok(list)
}
