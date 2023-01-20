# AWS Caller ID

This tool exists as a concrete, complete example of
a Rust command line tool that interacts with AWS.

I do not think this running tool is very useful.
My plan is to use this a sort-of template when
writing AWS command line tools in the future.

## Built in help

```
aws-caller-id -h
Print AWS caller identity

Usage: aws-caller-id [OPTIONS]

Options:
  -p, --profile <PROFILE>  AWS profile to use
  -r, --region <REGION>    AWS region to target
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

```
aws-caller-id --help
Calls the AWS GetCallerIdentity API to get the caller identity and prints
the result.

There is little reason to run this tool. My goal here is a simple, complete
example for a command line program that makes calls to AWS.

You can set the environment variable `RUST_LOG` to adjust logging, for
example `RUST_LOG=trace aws-caller-id`

Usage: aws-caller-id [OPTIONS]

Options:
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