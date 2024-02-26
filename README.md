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

## 3rd Party Dependencies
### Fleek Platform

This tool will upload functions to IPFS using the Fleek Platform SDK.  For this you will need to do the following:

1. Create an account at https://app.fleek.xyz/
2. Create an API key using the Fleek CLI as per the instructions at https://docs.fleek.xyz/docs/SDK. Note you will need to create a `PersonalAccessTokenService` token type since this will be run from the backend.
3. Set the following env var to use the Fleek API key

`FLEEK_PAT=`

### AWS

This test also uses AWS for EC2 creation to run the clients from various regions.  As such you will need to create your own AWS account and then get an API key from there. Set the following environment variables to configure the AWS API keys.

1. Create an account on AWS
2. Create API keys to be used
3. Add the following env vars

```
AWS_ACCOUNT_ID=
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
```

4. If this is your first time running the AWS CDK against your AWS account, you will need to call the bootstrap command for each region you want to operate this in. E.g., for US East 1 call `cdk bootstrap aws://<account number from step 1>/us-east-1`.

5. Create an SSH key and make sure you upload it to each region so your script can use the same SSH key for the instances in different regions.

Note, currently all of these variable must be set and this tool has not been testing using other forms of AWS API key confiuration like using the `.aws` directory.

Other useful commands:

```
cdk deploy deploy this stack to your default AWS account/region
cdk diff compare deployed stack with current state
cdk synth emits the synthesized CloudFormation template
```

## Deploy:

You will need to install the aws-cdk cli to your machine and set your ts/.env after you can use ts/deploy.sh,
this will automatically deploy the service to the ec2 instances as a system service and expose port 3000

```
    yarn global add aws-cdk
    chmod +x ts/deploy.sh
    ts/deploy.sh
```

## Run:

[cli-args](client/src/main.rs#L11)
To run the benchmarks against the deployed EC2 & an example Fleek function, simply `cargo run` in the [Client](client/) directory and it will automatically 
run against the deployed values

You can also do `cargo run -- --help` to see the CLI args, which have featrues such as a optional comparsion URL and configurable paramters.
