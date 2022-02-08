mod cloudflare;
mod matrix;

use anyhow::{anyhow, Result};
use clap::Parser;
use log::{debug, error, info};
use serde::Deserialize;
use std::fs;

/// Tool to keep Cloudflare DNS records up to date with a dynamic/residential IP address.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Config file to load
    #[clap(short, long)]
    config: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    pub cloudflare: cloudflare::CloudflareConfig,
    pub matrix: matrix::MatrixConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string(args.config)?)?;

    let matrix_client = matrix::login(&config.matrix).await?;

    debug! {"Getting public IP..."};
    let ip = match public_ip::addr_v4()
        .await
        .ok_or(anyhow! {"Failed to get public IP"})
    {
        Ok(ip) => {
            info! {"Detected public IP: {:?}", ip};
            if config.matrix.verbose {
                matrix::send_message(
                    &config.matrix,
                    &matrix_client,
                    format! {"Public IP: {}", ip}.as_str(),
                )
                .await?;
            }
            Ok(ip)
        }
        Err(e) => {
            matrix::send_message(
                &config.matrix,
                &matrix_client,
                format! {"Error: {}", e}.as_str(),
            )
            .await?;
            Err(e)
        }
    }?;

    let client = cloudflare::new_client(&config.cloudflare)?;
    let zones = cloudflare::DnsZones::new(&client, &config.cloudflare.records).await?;

    let mut results: Vec<Result<String>> = Vec::new();
    for c in config
        .cloudflare
        .records
        .into_iter()
        .map(|r| cloudflare::Task::new(&ip, &zones, &r))
        .flatten()
    {
        results.push(c.run(&client, &ip).await)
    }

    for result in &results {
        match result {
            Ok(message) => {
                info! {"{}", message};
                matrix::send_message(&config.matrix, &matrix_client, message.as_str()).await?;
            }
            Err(e) => {
                error! {"{}", e};
                matrix::send_message(
                    &config.matrix,
                    &matrix_client,
                    format! {"Error: {}", e}.as_str(),
                )
                .await?;
            }
        }
    }

    match results.iter().filter(|&r| r.is_err()).count() {
        0 => Ok(()),
        a => Err(anyhow! {"{} update(s) have failed", a}),
    }
}
