mod collect;

use clap::Parser;
use collect::Labeled;
use measure::{MeasureRequest, MeasureResponse};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
pub struct CliArgs {
    /// The ip address of the measure service
    measure_ip: String,

    /// The url the measure service will be calling the http `get` method` on
    target_request_url: String,

    /// The comparison url the measure service will be calling the http `get` method` on
    comparison_url: Option<String>,

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

    let _ = Runtime::new(args.target_request_url.clone(), args.comparison_url.clone(), args.times).start(args).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Runtime {
    results: Labeled,
    comparison_results: Option<Labeled>,
}

impl Runtime {
    fn new(target: String, maybe_comp: Option<String>, times: usize) -> Self {
        Runtime {
            results: Labeled::with_capacity(target, times),
            comparison_results: maybe_comp.map(|comp| Labeled::with_capacity(comp, times)),
        }
    }

    async fn start(mut self, args: CliArgs) -> anyhow::Result<()> {
        self.run(args).await
    }

    async fn run(&mut self, args: CliArgs) -> anyhow::Result<()> {
        let CliArgs {
            measure_ip,
            target_request_url,
            average,
            times,
            delay,
            output_file,
            flood: _,
            comparison_url: _
        } = args;

        let req = ClientBuilder::new()
            .build()?
            .post(&measure_ip)
            .json(&MeasureRequest {
                target: target_request_url.clone(),
            });

        if let Some(ref mut empty) =  self.comparison_results {
            let comparison_req = ClientBuilder::new()
                    .build()?
                    .post(&measure_ip)
                    .json(&MeasureRequest {
                        target: empty.label.clone(),
                    });

                Self::measure(empty, comparison_req, times, delay).await?;
        }

        Self::measure(&mut self.results, req, times, delay).await?;

        if let Some(ref comp) = self.comparison_results {
            Labeled::print_comped(&self.results, comp);
        } else {
            self.results.print();
        }

        if average {
            let target = Labeled::average(&self.results);
            let comp = self.comparison_results.as_ref().map(|comp| Labeled::average(comp));

            print_average(self.results.label.clone(), target);

            if let Some(comp) = comp {
                print_average(self.comparison_results.as_ref().unwrap().label.clone(), comp);
            }
        }

        if let Some(output_file) = output_file {
            if let Some(parent) = std::path::Path::new(&output_file).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::File::create(output_file)?;

            serde_json::to_writer(&mut file, &self)?;
        }

        Ok(())
    }

    async fn measure(
        buf: &mut Vec<MeasureResponse>,
        req: reqwest::RequestBuilder,
        times: usize,
        delay: usize,
    ) -> anyhow::Result<()> {
        for _ in 0..times {
            let cloned = req.try_clone().expect("cloneable request");

            let res = cloned.send().await?.json::<MeasureResponse>().await?;
            
            buf.push(res);

            tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
        }

        Ok(())
    }
}

fn print_average(label: String, measure: MeasureResponse) {
    println!("URL: {:#?}", label);
    println!("Average: {}ms", measure.ttfb_duration.as_millis());
} 