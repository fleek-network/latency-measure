use crate::CliArgs;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

/// The Jobs struct is used to store the parsed jobs from the config file
/// on creation it tries to read from the outputs.json file created by the aws deployment,
/// otherwise you can pass in the service IPs via the CLI
#[derive(Debug, Clone)]
pub struct Jobs {
    // The ips of the measure services
    pub services: Vec<String>,
    // The parsed url of the target request
    pub target_url: String,
    // The parsed method of the target request
    pub target_method: String,
    // The parsed body of the target request
    pub target_body: Option<String>,
    // The parsed headers of the target request
    pub target_headers: Option<HashMap<String, String>>,
    // The parsed url of the comparison request
    pub comparison_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Inner {
    instance_latency_service: String,
}

impl CliArgs {
    pub fn jobs(&self) -> anyhow::Result<Jobs> {
        println!("target_request_url: {:?}", self.target_request_url);
        Ok(Jobs {
            services: self.services.clone().unwrap_or(try_read_service_ips()?),
            target_method: self.target_request_method.clone().unwrap_or("GET".to_string()),
            target_body: self.target_request_body.clone(),
            target_headers: self.target_request_headers.clone().map(|v| v.into_iter().collect()),
            comparison_url: self.comparison_url.clone(),
            target_url: self
                .target_request_url
                .clone()
                .unwrap_or(try_get_deployed_url()?),
        })
    }
}

pub fn try_read_service_ips() -> anyhow::Result<Vec<String>> {
    const SERVICES: &str = "../ts/outputs.json";

    let file = fs::File::open(SERVICES)?;
    let inner: HashMap<String, Inner> = serde_json::from_reader(file)
        .context("failed to parse the json from outputs.json from deployment, please pass in a service ip to the CLI or complete the deployment process")?;

    Ok(inner
        .into_values()
        .map(|i| {
            let formatted = format!("http://{}:3000", i.instance_latency_service);

            println!("found service ip: {}", formatted);

            formatted
        })
        .collect::<Vec<_>>())
}

pub fn try_get_deployed_url() -> anyhow::Result<String> {
    println!("trying to get deployed url");
    const CID: &str = "../ts/CID.txt";

    let cid =
        fs::read_to_string(CID).context("error trying to read ts/CID.txt file from deployment, either pass in a target_url or a complete the deploymnet process")?;

    Ok(format!(
        "https://fleek-test.network/services/1/ipfs/{}",
        cid
    ))
}
