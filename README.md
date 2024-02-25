## Overview

Deploy ec2 instances across the world, setup the service, and execute TTFB recording the scores.

- [Service](service/)
  - The binary being deployed to the EC2 instances, sets up a http server that runs arbitrary get requests and reports the TTFB.
- [Client](client/)
  - In charge of sending requests to the EC2 instances and the lambda comparison and recording them to the db
  - Can do concurrent or sequential requests
- [TS](ts/)
  - The deployment script for the EC2 instances in various geographic locations, and spawns a [Service](service/) system service and exposes port 3000

All crates have `cargo run -- --help` for usage

### Deploy:

You will need to install the aws-cdk cli to your machine and set your ts/.env after you can use ts/deploy.sh,
this will automatically deploy the service to the ec2 instances as a system service and expose port 3000

```
    yarn global add aws-cdk
    chmod +x ts/deploy.sh
    ts/deploy.sh
```

### Run:

[cli-args](client/src/main.rs#L11)
To run the benchmarks against a the EC2 & Fleek function deployment, simply `cargo run` in the [Client](client/) directory and it will automatically 
run against the deployment and the deployed function.

You can also do `cargo run -- --help` to see the CLI args, which have featrues such as a optional comparsion URL and configurable paramters.

### Results:

The data is written in human friendly format to stdout throught the execution, however by default the `test-against-deployed-ec2` script writes JSON files per IP in the `scores-<timestamp>` directory.

The names of the files are the ip addresses of the ec2 instances that measured the ttfb.

Example JSON Object

```json
{
  "results": {
    "label": "target",
    "inner": [
      {
        "ip": "0.0.0.0",
        "dns_lookup_duration": {
          "secs": 0,
          "nanos": 0
        },
        "tcp_connect_duration": {
          "secs": 0,
          "nanos": 0
        },
        "http_get_send_duration": {
          "secs": 0,
          "nanos": 0
        },
        "ttfb_duration": {
          "secs": 0,
          "nanos": 0
        },
        "tls_handshake_duration": {
          "secs": 0,
          "nanos": 0
        }
      }
    ]
  },
  "comparison_results": {
    "label": "comp",
    "inner": [
      {
        "ip": "0.0.0.0",
        "dns_lookup_duration": {
          "secs": 0,
          "nanos": 0
        },
        "tcp_connect_duration": {
          "secs": 0,
          "nanos": 0
        },
        "http_get_send_duration": {
          "secs": 0,
          "nanos": 0
        },
        "ttfb_duration": {
          "secs": 0,
          "nanos": 0
        },
        "tls_handshake_duration": {
          "secs": 0,
          "nanos": 0
        }
      }
    ]
  }
}
```
