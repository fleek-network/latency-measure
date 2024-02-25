set -oe allexport

source .env
cdk deploy --outputs-file=outputs.json --all