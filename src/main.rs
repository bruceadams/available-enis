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

/// Wrapper for Option<&NetworkInterfaceStatus> to use as a key in a HashMap
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum FlatStatus {
    Status(NetworkInterfaceStatus),
    None,
}

// TODO: Write a Display for FlatStatus.
// To use the `{}` marker, the trait `fmt::Display` must be implemented
// manually for the type.
impl std::fmt::Display for FlatStatus {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "{}",
            match self {
                FlatStatus::Status(status) => match status {
                    NetworkInterfaceStatus::Associated => "associated",
                    NetworkInterfaceStatus::Attaching => "attaching",
                    NetworkInterfaceStatus::Available => "available",
                    NetworkInterfaceStatus::Detaching => "detaching",
                    NetworkInterfaceStatus::InUse => "in-use",
                    _ => "unknown",
                },
                FlatStatus::None => "none",
            }
        )
    }
}

fn status_counts<'a>(
    network_interfaces: DescribeNetworkInterfacesOutput,
) -> HashMap<FlatStatus, u64> {
    let mut counts = HashMap::new();
    for eni in network_interfaces.network_interfaces().unwrap_or_default() {
        let key = match eni.status() {
            Some(status) => FlatStatus::Status(status.clone()),
            None => FlatStatus::None,
        };
        let plus_one = 1 + counts.get(&key).unwrap_or(&0);
        counts.insert(key, plus_one);
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
        println!("{:6}  {}", value, key);
    }
    Ok(())
}
