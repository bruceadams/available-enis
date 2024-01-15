# Available AWS Elastic Network Interfaces

Summarize the status of every AWS Elastic Network Interface, eni.
Optionally, delete every ENI with a status of "available".

This is a very narrow tool that pretty much does one thing:
cleanup stray Elastic Network Interfaces that seem to fall
out of a very complex terraform configuration that we build
and tear down regularly.

This is _very_ fast. The deletes all run concurrently.

## Built in help

### Terse

```
available-enis -h
Count and optionally delete available AWS Elastic Networks

Usage: available-enis [OPTIONS]

Options:
  -d, --delete             Delete "available" ENIs
  -p, --profile <PROFILE>  AWS profile to use
  -r, --region <REGION>    AWS region to target
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

### Complete

```
available-enis --help
Summarize the status of every AWS Elastic Network Interface, eni.
Optionally, delete every ENI with a status of "available".

You can set the environment variable `RUST_LOG` to adjust logging, for
example `RUST_LOG=trace aws-caller-id`

Usage: available-enis [OPTIONS]

Options:
  -d, --delete
          Delete "available" ENIs

  -p, --profile <PROFILE>
          AWS profile to use.

          This overrides the standard (and complex!) AWS profile handling.

  -r, --region <REGION>
          AWS region to target.

          This override the standard (and complex!) AWS region handling.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Installing

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-installer.sh | sh
```

### Install prebuilt binaries via powershell script

```sh
irm https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-installer.ps1 | iex
```

### Install prebuilt binaries via Homebrew

```sh
brew install bruceadams/homebrew-utilities/available-enis
```

### Install prebuilt binaries via cargo binstall

```sh
cargo binstall available-enis
```

## Download

|  File  | Platform | Checksum |
|--------|----------|----------|
| [available-enis-aarch64-apple-darwin.tar.xz](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-aarch64-apple-darwin.tar.xz) | macOS Apple Silicon | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-aarch64-apple-darwin.tar.xz.sha256) |
| [available-enis-x86_64-apple-darwin.tar.xz](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-apple-darwin.tar.xz) | macOS Intel | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-apple-darwin.tar.xz.sha256) |
| [available-enis-x86_64-pc-windows-msvc.zip](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-pc-windows-msvc.zip) | Windows x64 | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-pc-windows-msvc.zip.sha256) |
| [available-enis-x86_64-unknown-linux-gnu.tar.xz](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-unknown-linux-gnu.tar.xz) | Linux x64 | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-unknown-linux-gnu.tar.xz.sha256) |
| [available-enis-x86_64-unknown-linux-musl.tar.xz](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-unknown-linux-musl.tar.xz) | musl Linux x64 | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-unknown-linux-musl.tar.xz.sha256) |
| [available-enis-x86_64-pc-windows-msvc.msi](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-pc-windows-msvc.msi) | Windows x64 | [checksum](https://github.com/bruceadams/available-enis/releases/latest/download/available-enis-x86_64-pc-windows-msvc.msi.sha256) |

## Building

This is a straightforward [Rust](https://www.rust-lang.org/) project.
`cargo build` should _just work_.

## Background

I've been using the following Bash which uses the AWS CLI.
This script is slow, deleting around two ENIs per second
and isn't especially informative.

Writing a solid program in Rust is maybe over-engineering the problem.
I enjoyed having a practical use case in front of me to write another
Rust CLI. I'm pleased that the tool defaults to making no changes,
simply reporting counts of ENI statuses it finds.

```bash
#!/usr/bin/env bash

set -euo pipefail

available_enis=$(
    aws ec2 describe-network-interfaces |
        jq -r '.NetworkInterfaces[] | select( .Status == "available" ) | .NetworkInterfaceId'
)

if [[ "$available_enis" ]]; then
    echo "Found $(echo -n "$available_enis" | wc -l) available enis"

    for eni in $available_enis; do
        echo "$eni"
        aws ec2 delete-network-interface --network-interface-id "$eni"
    done
else
    echo No available enis found
fi
```
