# Available AWS Elastice Network Interfaces

Summarize the status of every AWS Elastic Network Interface, eni.
Optionally, delete every ENI with a status of "available".

This is a very narrow tool that pretty much does one thing:
cleanup stray Elastice Network Interfaces that seem to fall
out of a very complex terraform configuration that we build
and tear down regularly.

This is _very_ fast. The deletes all run concurrently.

## Built in help

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