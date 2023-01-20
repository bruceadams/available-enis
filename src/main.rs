use std::collections::HashMap;

use anyhow::{Context, Result};
use aws_config::SdkConfig;
use aws_sdk_ec2::{model::NetworkInterfaceStatus, output::DescribeNetworkInterfacesOutput, Client};
use aws_types::region::Region;
use clap::Parser;
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Calls the AWS GetCallerIdentity API to get the
/// caller identity and prints the result.
///
/// There is little reason to run this tool.
/// My goal here is a simple, complete example for a
/// command line program that makes calls to AWS.
///
/// You can set the environment variable `RUST_LOG` to
/// adjust logging, for example `RUST_LOG=trace aws-caller-id`
#[derive(Clone, Debug, Parser)]
#[command(about, author, version)]
struct MyArgs {
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

#[derive(Debug, Eq, Hash, PartialEq)]
enum FlatStatus {
    Associated,
    Attaching,
    Available,
    Detaching,
    InUse,
    None,
    Unknown,
}

fn status_counts(network_interfaces: DescribeNetworkInterfacesOutput) -> HashMap<FlatStatus, u64> {
    let mut counts: HashMap<FlatStatus, u64> = HashMap::new();
    for eni in network_interfaces.network_interfaces().unwrap_or_default() {
        match eni.status() {
            Some(NetworkInterfaceStatus::Associated) => counts.insert(
                FlatStatus::Associated,
                1 + counts.get(&FlatStatus::Associated).unwrap_or(&1),
            ),
            Some(NetworkInterfaceStatus::Attaching) => counts.insert(
                FlatStatus::Attaching,
                1 + counts.get(&FlatStatus::Attaching).unwrap_or(&1),
            ),
            Some(NetworkInterfaceStatus::Available) => counts.insert(
                FlatStatus::Available,
                1 + counts.get(&FlatStatus::Available).unwrap_or(&1),
            ),
            Some(NetworkInterfaceStatus::Detaching) => counts.insert(
                FlatStatus::Detaching,
                1 + counts.get(&FlatStatus::Detaching).unwrap_or(&1),
            ),
            Some(NetworkInterfaceStatus::InUse) => counts.insert(
                FlatStatus::InUse,
                1 + counts.get(&FlatStatus::InUse).unwrap_or(&1),
            ),
            None => counts.insert(
                FlatStatus::None,
                1 + counts.get(&FlatStatus::None).unwrap_or(&1),
            ),
            _ => counts.insert(
                FlatStatus::Unknown,
                1 + counts.get(&FlatStatus::Unknown).unwrap_or(&1),
            ),
        };
    }
    counts
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let args = MyArgs::parse();
    let config = aws_sdk_config(&args).await;
    debug!("Config: {:?}", config);
    let client = Client::new(&config);
    let result = client
        .describe_network_interfaces()
        .send()
        .await
        .context("calling describe_network_interfaces")?;
    debug!("{:#?}", result);
    let counts = status_counts(result);
    println!(" count  status");
    for (key, value) in counts.into_iter() {
        println!("{:6}  {:?}", value, key);
    }
    Ok(())
}
