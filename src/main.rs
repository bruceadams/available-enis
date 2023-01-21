use anyhow::{anyhow, Context, Result};
use aws_config::SdkConfig;
use aws_sdk_ec2::{model::NetworkInterfaceStatus, output::DescribeNetworkInterfacesOutput, Client};
use aws_types::region::Region;
use clap::Parser;
use futures::future::join_all;
use futures::prelude::*;
use std::collections::HashMap;
use tracing::{debug, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Summarize the status of every AWS Elastic Network Interface, eni.
/// Optionally, delete every ENI with a status of "available".
///
/// You can set the environment variable `RUST_LOG` to
/// adjust logging, for example `RUST_LOG=trace aws-caller-id`
#[derive(Clone, Debug, Parser)]
#[command(about, author, version)]
struct MyArgs {
    /// Delete "available" ENIs.
    #[arg(long, short)]
    delete: bool,
    /// AWS profile to use.
    ///
    /// This overrides the standard (and complex!) AWS profile handling.
    #[arg(long, short)]
    profile: Option<String>,

    /// AWS region to target.
    ///
    /// This override the standard (and complex!) AWS region handling.
    #[arg(long, short)]
    region: Option<String>,
}

async fn aws_sdk_config(args: &MyArgs) -> SdkConfig {
    let base = aws_config::from_env();
    let with_profile = match &args.profile {
        None => base,
        Some(profile_name) => base.profile_name(profile_name),
    };
    let with_overrides = match &args.region {
        None => with_profile,
        Some(region_name) => with_profile.region(Region::new(region_name.clone())),
    };
    with_overrides.load().await
}

fn status_counts<'a>(
    network_interfaces: &DescribeNetworkInterfacesOutput,
) -> HashMap<&'a str, u64> {
    let mut counts = HashMap::new();
    for eni in network_interfaces.network_interfaces().unwrap_or_default() {
        let key = match eni.status() {
            Some(status) => match status {
                NetworkInterfaceStatus::Associated => "associated",
                NetworkInterfaceStatus::Attaching => "attaching",
                NetworkInterfaceStatus::Available => "available",
                NetworkInterfaceStatus::Detaching => "detaching",
                NetworkInterfaceStatus::InUse => "in-use",
                _ => "unknown",
            },
            None => "none",
        };
        counts.insert(key, 1 + counts.get(&key).unwrap_or(&0));
    }
    counts
}

/// Attempt to delete every "available" ENI concurrently.
async fn delete_available<'a>(
    client: &Client,
    network_interfaces: &DescribeNetworkInterfacesOutput,
) -> Result<()> {
    // We hope for success, but will return the last error we saw, if any.
    let mut return_result = Ok(());

    let available_network_interface_ids = network_interfaces
        .network_interfaces()
        .unwrap_or_default()
        .iter()
        .filter_map(|eni| {
            if eni.status() == Some(&NetworkInterfaceStatus::Available) {
                let network_interface_id = eni.network_interface_id();
                if network_interface_id.is_none() {
                    error!("Ignoring available ENI which has no network_interface_id.");
                    return_result = Err(anyhow!("available ENI has no network_interface_id"));
                }
                network_interface_id
            } else {
                None
            }
        });

    let futures = available_network_interface_ids.map(|network_interface_id| {
        client
            .delete_network_interface()
            .network_interface_id(network_interface_id)
            .send()
            .map(move |result| (network_interface_id, result))
    });

    let results = join_all(futures).await;

    for (network_interface_id, result) in results {
        match result {
            Ok(_) => println!("Deleted {}", network_interface_id),
            Err(error) => {
                let error = error.into();
                error!("Delete failed for {}: {:?}", network_interface_id, error);
                return_result = Err(error);
            }
        }
    }
    return_result
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let args = MyArgs::parse();
    let config = aws_sdk_config(&args).await;
    debug!("Config: {:#?}", config);
    let client = Client::new(&config);
    let result = client
        .describe_network_interfaces()
        .send()
        .await
        .context("calling describe_network_interfaces")?;
    debug!("{:#?}", result);
    let counts = status_counts(&result);
    println!(" count  status");
    for (key, value) in counts.into_iter() {
        println!("{:6}  {}", value, key);
    }
    if args.delete {
        delete_available(&client, &result)
            .await
            .context("deleting available enis")?
    }
    Ok(())
}
