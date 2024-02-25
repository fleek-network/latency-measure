## Overview

Deploy ec2 instances across the world and record thier relative ttfb measurments

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
To run the benchmarks against the deployed EC2 & an example Fleek function, simply `cargo run` in the [Client](client/) directory and it will automatically 
run against the deployed values

You can also do `cargo run -- --help` to see the CLI args, which have additional fewtures such as pretty print comparsions, saving the results as a json and other configurable paramters

