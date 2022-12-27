mod cloudflare;

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::Deserialize;
use std::{fs, path::PathBuf};

/// Tool to keep Cloudflare DNS records up to date with a dynamic/residential IP address.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Config file to load
    #[clap(short, long, value_parser)]
    config: PathBuf,
}

#[derive(Deserialize, Debug)]
struct Config {
    pub cloudflare: cloudflare::CloudflareConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string(args.config)?)?;
    log::info!(
        "{} records are configured to be monitored",
        config.cloudflare.records.len()
    );

    log::debug!("Getting public IP...");
    let ip = match public_ip::addr_v4()
        .await
        .ok_or_else(|| anyhow!("Failed to get public IP"))
    {
        Ok(ip) => {
            log::info!("Detected public IP: {:?}", ip);
            Ok(ip)
        }
        Err(e) => Err(e),
    }?;

    let client = cloudflare::new_client(&config.cloudflare)?;
    let zones = cloudflare::DnsZones::new(&client, &config.cloudflare.records).await?;

    let mut result = Ok(());

    for c in config
        .cloudflare
        .records
        .into_iter()
        .filter_map(|r| cloudflare::Task::new(&ip, &zones, &r))
    {
        match c.run(&client, &ip).await {
            Ok(msg) => {
                log::info!("{}", msg);
            }
            Err(e) => {
                log::error!("{}", e);
                result = Err(anyhow!("At least one update failed"));
            }
        }
    }

    result
}
