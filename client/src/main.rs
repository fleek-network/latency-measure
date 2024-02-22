mod collect;

use clap::Parser;
use collect::Labeled;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use measure::{MeasureRequest, MeasureResponse};
use reqwest::ClientBuilder;

#[derive(Parser)]
pub struct CliArgs {
    /// The ip address of the measure service
    measure_ip: String,

    /// The url the measure service will be calling the http `get` method` on
    target_request_url: String,

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

    let _ = Runtime::new(args.times).start(args).await?;

    Ok(())
}

struct Runtime {
    results: Vec<Labeled>,
}

impl Runtime {
    fn new(times: usize) -> Self {
        Runtime {
            results: Vec::with_capacity(times),
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
            flood,
            ..
        } = args;

        let req = ClientBuilder::new()
            .build()?
            .post(measure_ip)
            .json(&MeasureRequest {
                target: target_request_url.clone(),
            });

        if flood {
            let mut futs = (0..times)
                .map(|_| {
                    let cloned = req.try_clone().expect("cloneable request");

                    tokio::spawn(
                        async move { cloned.send().await?.json::<MeasureResponse>().await },
                    )
                })
                .collect::<FuturesOrdered<_>>();

            while let Some(res) = futs.next().await {
                match res {
                    Ok(res) => match res {
                        Ok(res) => {
                            let labeled = Labeled::new(res, "flood".to_string());
                            labeled.print();
                            self.results.push(labeled);
                        }
                        Err(e) => {
                            println!("network error: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("failed to join handle: {}", e);
                    }
                }
            }
        } else {
            for i in 0..times {
                let cloned = req.try_clone().expect("cloneable request");

                let res = cloned.send().await?.json::<MeasureResponse>().await?;

                let labeled = Labeled::new(res, i.to_string());
                labeled.print();
                self.results.push(labeled);

                tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
            }
        }

        if average {
            let summed = Labeled::average(&self.results, times);

            let labeled = Labeled::new(summed, "average".to_string());
            labeled.print();
            self.results.push(labeled);
        }

        if let Some(output_file) = output_file {
            if let Some(parent) = std::path::Path::new(&output_file).parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(output_file)?;

            serde_json::to_writer(&mut file, &self.results)?;
        }

        Ok(())
    }
}
