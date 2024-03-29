use anyhow::{anyhow, Context, Result};
use aws_config::{identity::IdentityCache, BehaviorVersion, SdkConfig};
use aws_sdk_ec2::{types::NetworkInterface, types::NetworkInterfaceStatus, Client};
use aws_types::region::Region;
use clap::Parser;
use futures::{future::join_all, prelude::*};
use std::{collections::HashMap, time::Duration};
use tracing::{debug, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Summarize the status of every AWS Elastic Network Interface, ENI.
/// Optionally, delete every ENI with a status of "available".
///
/// You can set the environment variable `RUST_LOG` to
/// adjust logging, for example `RUST_LOG=trace available-enis`
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
    let base = aws_config::defaults(BehaviorVersion::latest()).identity_cache(
        IdentityCache::lazy()
            .load_timeout(Duration::from_secs(90))
            .build(),
    );
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

async fn get_network_interfaces(client: &Client) -> Result<Vec<NetworkInterface>> {
    let mut result = client
        .describe_network_interfaces()
        .send()
        .await
        .context("calling describe_network_interfaces")?;
    debug!("{:#?}", result);
    let mut network_interfaces = result.network_interfaces().to_vec();

    while let Some(next_token) = result.next_token() {
        result = client
            .describe_network_interfaces()
            .next_token(next_token)
            .send()
            .await
            .context("calling describe_network_interfaces")?;
        debug!("{:#?}", result);
        network_interfaces.extend_from_slice(result.network_interfaces())
    }

    Ok(network_interfaces)
}

fn print_status_counts(network_interfaces: &Vec<NetworkInterface>) {
    let mut counts = HashMap::new();
    for eni in network_interfaces {
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

    let mut statuses: Vec<&str> = counts.keys().copied().collect();
    statuses.sort();
    println!("    count  status");
    println!("   ------  ---------");
    for status in statuses {
        println!("{:9}  {}", counts.get(status).unwrap_or(&0), status);
    }
    if counts.len() > 1 {
        println!("   ------  ---------");
        println!("{:9}  total", network_interfaces.len());
    }
}

/// Attempt to delete every "available" ENI concurrently.
async fn delete_available<'a>(
    client: &Client,
    network_interfaces: &[NetworkInterface],
) -> Result<()> {
    // We hope for success, but will return the last error we saw, if any.
    let mut return_result = Ok(());

    let available_network_interface_ids = network_interfaces.iter().filter_map(|eni| {
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
    let network_interfaces = get_network_interfaces(&client).await?;
    print_status_counts(&network_interfaces);
    if args.delete {
        delete_available(&client, &network_interfaces)
            .await
            .context("deleting available enis")?
    }
    Ok(())
}
