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
