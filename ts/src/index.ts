import { AwsStackV2 } from "./stack";
import * as cdk from 'aws-cdk-lib';
require('dotenv').config();

const app = new cdk.App();

const regions = [
    'us-east-1', // n virginia
    'us-west-1', // n california
    'ap-south-1', // mumbai
    'ap-northeast-1', // tokyo
    'ap-southeast-1', // singapore
    'ca-central-1', // canada
    'eu-central-1', // frankfurt
    'eu-west-1', // ireland
    'sa-east-1', // sao paulo
];

regions.forEach(region => {
    new AwsStackV2(app, `FleekNetworkTestStack-${region}`, {
        env: {
            account: process.env.AWS_ACCOUNT_ID,
            region: region,
        }
    });
})