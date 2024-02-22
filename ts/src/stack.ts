import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as iam from 'aws-cdk-lib/aws-iam';

const instanceName = `Latency-Service`;

type AwsStackV2Props = cdk.StackProps & {
  keyName?: string;
}
export class AwsStackV2 extends cdk.Stack {
  constructor(scope: Construct, id: string, props: AwsStackV2Props) {
    super(scope, id, props);

    const vpc = new ec2.Vpc(this, 'FleekNetworkTestNetworkVpc', {
      maxAzs: 2,
    });

    // PRD
    // 8 vcpu, 32 GiB ram, 12.5/10 Gbps
    // const instanceSize = 'm7a.2xlarge';

      // BENCHMARKING
    // 4 vcpu, 16 GiB ram, 10 Gbps
    // smallest instance size with 10 Gbps
    // $0.172/hr as of October 25 2023 14:53 ETC
    const instanceSize = 'm5a.xlarge';


    const securityGroup = new ec2.SecurityGroup(this, 'DeveloperAccessSG', {
      vpc,
      securityGroupName: "network-tester",
    });

    // allow ssh and http inbound on 3000
    securityGroup.addIngressRule(ec2.Peer.anyIpv4(), ec2.Port.tcp(22));
    securityGroup.addIngressRule(ec2.Peer.anyIpv4(), ec2.Port.tcp(3000));

    const role = new iam.Role(this, 'InstanceRole', {
      assumedBy: new iam.ServicePrincipal('ec2.amazonaws.com'),
    });

    role.addManagedPolicy(iam.ManagedPolicy.fromAwsManagedPolicyName('AmazonSSMManagedInstanceCore'));
    role.addManagedPolicy(iam.ManagedPolicy.fromAwsManagedPolicyName('CloudWatchAgentServerPolicy'));
    role.addManagedPolicy(iam.ManagedPolicy.fromAwsManagedPolicyName('AmazonEC2ContainerRegistryReadOnly'));

    const cfnPolicy = new iam.Policy(this, 'cloudformation-cfn-init-policy', {
      roles: [role],
      statements: [
        new iam.PolicyStatement({
          actions: ['cloudformation:DescribeStackResource', 'cloudformation:SignalResource'],
          resources: [this.stackId],
        }),
      ],
    });

    const rootVolume: ec2.BlockDevice = {
      // obtain device name from ami
      // aws ec2 describe-images --region us-east - 1 --image - ids ami-03f6c2c562b3df715
      deviceName: '/dev/sda1',
      volume: ec2.BlockDeviceVolume.ebs(100, {
        deleteOnTermination: true,
        volumeType: ec2.EbsDeviceVolumeType.GP3,
      }),
    };

    const machineImage = ec2.MachineImage.fromSsmParameter(
      '/aws/service/canonical/ubuntu/server/focal/stable/current/amd64/hvm/ebs-gp2/ami-id',
      {os: ec2.OperatingSystemType.LINUX}
    )

    const instance = new ec2.Instance(this, instanceName, {
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PUBLIC,
      },
      role,
      keyName: props.keyName,
      instanceType: new ec2.InstanceType(instanceSize),
      machineImage: machineImage,
      associatePublicIpAddress: true,
      blockDevices: [rootVolume],
      propagateTagsToVolumeOnCreation: true,
      resourceSignalTimeout: cdk.Duration.minutes(15),
    });

    const logicalId = instance.stack.getLogicalId(instance.node.defaultChild as cdk.CfnElement);

    instance.userData.addCommands(
      // install dependencies
      "apt-get update -yq",
      "apt-get install lsof git build-essential cmake clang pkg-config libssl-dev protobuf-compiler gcc gcc-multilib libprotobuf-dev protobuf-compiler python2 -yq",
      "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
      "export PATH=\"/root/.cargo/bin:$PATH\"",
      "rustup install stable",
      // install service
      "git clone https://github.com/fleek-network/latency-measure",
      "cd latency-measure/measure && cargo install --path .",
      "cd ../ && cp measure.service.template /etc/systemd/system/measure.service",
      "sed -i 's|REPLACE_WITH_PATH_TO_BIN|/root/.cargo/bin/measure|g' /etc/systemd/system/measure.service",
      "sudo chmod 644 /etc/systemd/system/measure.service",
      "systemctl enable measure",
      "systemctl daemon-reload",
      "systemctl start measure",
      // cfn-signal
      "curl https://bootstrap.pypa.io/pip/2.7/get-pip.py --output get-pip.py",
      "python2 get-pip.py",
      "pip2 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-latest.tar.gz",
      `python2 /usr/local/bin/cfn-signal -e $? --stack ${this.stackName} --resource ${logicalId} --region ${this.region}`,
    );

    new cdk.CfnOutput(this, `instance: ${instanceName}`, { value: instance.instancePublicIp });
  }
}