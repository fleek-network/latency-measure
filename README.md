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
You will need to install the aws-cdk cli to your machine, set your env after you can use ts/deploy.sh, 
this will automatically deploy the service to the instances as a system service and expose port 3000

```
    yarn global add aws-cdk
    chmod +x ts/deploy.sh
    ts/deploy.sh
```

### Run:
[cli-args](client/src/main.rs#L11)
To run the benchmarks against a url and deployed EC2 instances from above use the following commands, after your scores will be placed in the newly created 'scores' directory, as well they will be printed to the 
```
    chmod +x run.sh
    ./run.sh <url> <cli-args>
```

otherwise you can just run the client/server directly configuring the ip, by just starting the [Service](service/) 
and using the [Client](client/) against that

see `cargo run -- --help` for more infomation on either crate

### Results:
The data is written in human friendly format to stdout throught the execution, however by default the `run` script writes JSON files per IP in the `scores-<timestamp>` directory.

The names of the files are the ip addresses of the ec2 instances that measured the ttfb.

An exmaple of viewing some of this data is:
Getting all ttfb durations (nanos -> secs + secs)
`ls -A1 scores-123123123 | xargs -t -I{}  jq '.[].inner | .ttfb_duration.nanos/1e9 + ttfb_duration.secs' scores-123123123/{}`

Example Object
```json
{
  "label": "6",
  "inner": {
    // the ip address of the server fulfilling the request
    "ip": "x.x.x.x",
    "dns_lookup_duration": {
      "secs": 0,
      "nanos": 307913
    },
    "tcp_connect_duration": {
      "secs": 0,
      "nanos": 7250313
    },
    "http_get_send_duration": {
      "secs": 0,
      "nanos": 3860
    },
    "ttfb_duration": {
      "secs": 0,
      "nanos": 23473720
    },
    "tls_handshake_duration": {
      "secs": 0,
      "nanos": 7736479
    }
  }
}
```
