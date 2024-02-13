Overview
- Measure
    - The binary being deployed to the EC2 instances, sets up a http server that runs arbitrary get requests and reports the TTFB.
- Orchestrator
    - In charge of sending requests to the EC2 instances and the lambda comparison and recording them to the db
- TS
    - The deployment script for the EC2 instances in various geographic locations