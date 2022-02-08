use anyhow::{Error, Result};
use cloudflare::{
    endpoints::dns::{
        CreateDnsRecord, CreateDnsRecordParams, DnsContent, DnsRecord, ListDnsRecords,
        ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams,
    },
    framework::{
        async_api::{ApiClient, Client},
        auth::Credentials,
        response::ApiSuccess,
        Environment, HttpApiClientConfig,
    },
};
use log::warn;
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Deserialize, Debug)]
pub(crate) struct RecordConfig {
    pub zone_id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CloudflareConfig {
    token: String,
    pub records: Vec<RecordConfig>,
}

pub(crate) fn new_client(config: &CloudflareConfig) -> Result<Client> {
    Client::new(
        Credentials::UserAuthToken {
            token: config.token.clone(),
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )
}

#[derive(Debug)]
pub(crate) struct DnsZones {
    records: Vec<DnsRecord>,
}

impl DnsZones {
    pub(crate) async fn new(client: &Client, records: &[RecordConfig]) -> Result<DnsZones> {
        let mut cache: Vec<DnsRecord> = Vec::new();
        let mut cached_zones: Vec<&str> = Vec::new();

        for r in records {
            if !cached_zones.contains(&r.zone_id.as_str()) {
                let ApiSuccess { result, .. } = client
                    .request(&ListDnsRecords {
                        zone_identifier: &r.zone_id,
                        params: ListDnsRecordsParams::default(),
                    })
                    .await?;
                cache.extend(result);
                cached_zones.push(&r.zone_id);
            }
        }

        Ok(DnsZones { records: cache })
    }

    fn find(&self, config: &RecordConfig) -> Option<&DnsRecord> {
        self.records
            .iter()
            .find(|&r| r.zone_id == config.zone_id && r.name == config.name)
    }
}

#[derive(Debug)]
pub(crate) enum Task {
    Create {
        zone_id: String,
        name: String,
    },
    Update {
        zone_id: String,
        id: String,
        name: String,
    },
}

impl Task {
    pub(crate) fn new(
        current_ip: &Ipv4Addr,
        zones: &DnsZones,
        record_config: &RecordConfig,
    ) -> Option<Task> {
        match zones.find(record_config) {
            Some(r) => {
                if let DnsContent::A { content } = r.content {
                    if content == *current_ip {
                        None
                    } else {
                        Some(Task::Update {
                            zone_id: r.zone_id.clone(),
                            id: r.id.to_string(),
                            name: record_config.name.clone(),
                        })
                    }
                } else {
                    warn! {"Matched a record, but it was not an A record as expected"}
                    None
                }
            }
            None => Some(Task::Create {
                zone_id: record_config.zone_id.clone(),
                name: record_config.name.clone(),
            }),
        }
    }

    pub(crate) async fn run(&self, client: &Client, ip: &Ipv4Addr) -> Result<String> {
        match self {
            Task::Create { zone_id, name } => {
                match client
                    .request(&CreateDnsRecord {
                        zone_identifier: zone_id,
                        params: CreateDnsRecordParams {
                            ttl: Some(1),
                            priority: None,
                            proxied: Some(false),
                            name,
                            content: DnsContent::A { content: *ip },
                        },
                    })
                    .await
                {
                    Ok(_) => Ok(format! {"Created record: {}", &name}),
                    Err(e) => Err(Error::new(e)),
                }
            }
            Task::Update { zone_id, id, name } => {
                match client
                    .request(&UpdateDnsRecord {
                        zone_identifier: zone_id,
                        identifier: id,
                        params: UpdateDnsRecordParams {
                            ttl: Some(1),
                            proxied: Some(false),
                            name,
                            content: DnsContent::A { content: *ip },
                        },
                    })
                    .await
                {
                    Ok(_) => Ok(format! {"Updated record: {}", &name}),
                    Err(e) => Err(Error::new(e)),
                }
            }
        }
    }
}
