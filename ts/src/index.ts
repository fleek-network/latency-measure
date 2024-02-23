// @ts-nocheck
import { AwsStackV2 } from "./stack";
import { FleekSdk, PersonalAccessTokenService } from "@fleekxyz/sdk";
import * as cdk from 'aws-cdk-lib';
import * as fs from 'node:fs/promises';
require('dotenv').config();

const deployFleekFunction = async () => {
    
    const fleekPat = new PersonalAccessTokenService({personalAccessToken: process.env.FLEEK_PAT!,projectId: process.env.FLEEK_PROJECT_ID});
    const fleekSdk = new FleekSdk({accessTokenService: fleekPat});
    
    const fleekFunction = `const main = (params) => {\n  return ${Math.floor(Math.random() * 1000)}\n}`;


    const result = await fleekSdk.ipfs().add({
        path: "function",
        content: fleekFunction
    })

await fs.writeFile("./ts/CID.txt", new TextEncoder().encode(result.cid));
}

deployFleekFunction();

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