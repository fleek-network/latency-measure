mod collect;
mod jobs;

use std::{collections::HashMap, path::Path};

use clap::Parser;
use jobs::Jobs;
use measure::{MeasureRequest, MeasureResponse};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
pub struct CliArgs {
    /// The url the measure service will be calling the http `get` method` on
    target_request_url: Option<String>,

    /// The comparison url the measure service will be calling the http `get` method` on
    #[clap(long = "comp")]
    comparison_url: Option<String>,

    /// The ip address of the measure services
    #[clap(long)]
    services: Option<Vec<String>>,

    /// Compute and print the average of the results
    #[clap(short, long)]
    average: bool,

    /// The number of times to get a latencty measurement from service
    #[clap(short, long, default_value_t = 10)]
    times: usize,

    /// The delay in milliseconds between each measurement
    #[clap(short, long, default_value_t = 500)]
    delay: usize,

    /// The output file to write the json results to
    #[clap(short, long)]
    output_file: Option<String>,

    /// Creates requests concurrently rather than sequentially
    /// and ignores the delay param
    #[clap(long)]
    flood: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let _ = Runtime::new(args)?.start().await?;

    Ok(())
}

#[derive(Debug)]
struct Runtime {
    jobs: Jobs,
    results: HashMap<String, Vec<MeasureResponse>>,
    comparison_results: Option<HashMap<String, Vec<MeasureResponse>>>,
    output_file: Option<String>,
    average: bool,
    times: usize,
    delay: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    target_results: HashMap<String, Vec<MeasureResponse>>,
    comparison_results: Option<HashMap<String, Vec<MeasureResponse>>>,
}

impl Runtime {
    fn new(args: CliArgs) -> anyhow::Result<Self> {
        Ok(Runtime {
            jobs: args.jobs()?,
            results: HashMap::new(),
            comparison_results: args.comparison_url.map(|_| HashMap::new()),
            average: args.average,
            times: args.times,
            delay: args.delay,
            output_file: args.output_file,
        })
    }

    async fn start(mut self) -> anyhow::Result<()> {
        let Jobs {
            services,
            target_url,
            comparison_url,
        } = self.jobs.clone();

        for service_ip in services {
            println!("running for: {}", service_ip);
            self.run(service_ip, target_url.clone(), comparison_url.clone())
                .await?;
        }

        let output = self.output();
        println!("{:#?}", output);

        if let Some(ref output_file) = self.output_file {
            if let Some(parent) = std::path::Path::new(output_file).parent() {
                // theres no other tasks running so blocking is acceptable
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::File::create(output_file)?;

            serde_json::to_writer(&mut file, &output)?;
        }

        Ok(())
    }

    async fn run(
        &mut self,
        service_ip: String,
        target_url: String,
        maybe_comp: Option<String>,
    ) -> anyhow::Result<()> {
        let req = ClientBuilder::new()
            .build()?
            .post(&service_ip)
            .json(&MeasureRequest { target: target_url.clone() });

        if let Some(ref url) = maybe_comp {
            let comparison_req = ClientBuilder::new()
                .build()?
                .post(&service_ip)
                .json(&MeasureRequest { target: url.clone() });

            self.comparison_results.as_mut().expect("comparison results").insert(
                service_ip.clone(),
                Self::measure(comparison_req, self.times, self.delay).await?,
            );
        }

        self.results.insert(
            service_ip.clone(),
            Self::measure(req, self.times, self.delay).await?,
        );

        if self.average {
            let target = collect::average(
                self.results
                    .get(&service_ip)
                    .expect("results for this ip")
                    .iter(),
                self.times,
            );

            print_average(target_url, target);

            match self.comparison_results {
                Some(ref comp) => {
                    let comp = collect::average(
                        comp.get(&service_ip).expect("results for this ip").iter(),
                        self.times,
                    );
                    
                    print_average(maybe_comp.expect("comparison url"), comp);
                }
                None => (),
            };
        }

        Ok(())
    }

    async fn measure(
        req: reqwest::RequestBuilder,
        times: usize,
        delay: usize,
    ) -> anyhow::Result<Vec<MeasureResponse>> {
        let mut buf = Vec::with_capacity(times);

        for _ in 0..times {
            let cloned = req.try_clone().ok_or(anyhow::anyhow!("failed to clone request"))?;

            let res = cloned.send().await?.json::<MeasureResponse>().await?;

            buf.push(res);

            tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
        }

        Ok(buf)
    }

    fn output(&self) -> Output {
        Output {
            target_results: self.results.clone(),
            comparison_results: self.comparison_results.clone(),
        }
    }
}

fn print_average(label: String, measure: MeasureResponse) {
    println!("URL: {:#?}", label);
    println!("Average: {}ms", measure.ttfb_duration.as_millis());
}
