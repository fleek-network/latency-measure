mod collect;
mod jobs;

use std::{collections::HashMap, error::Error, fmt::Write};

use clap::Parser;
use indicatif::{ProgressState, ProgressStyle};
use jobs::Jobs;
use measure::{MeasureDurationRequest, MeasureRequest, MeasureResponse};
use reqwest::{ClientBuilder, RequestBuilder};
use serde::{Deserialize, Serialize};
use tabled::builder::Builder;

#[derive(Parser)]
pub struct CliArgs {
    /// The url the measure service will be making the http request to
    target_request_url: Option<String>,

    /// The HTTP method for the http request the measure service will be making to the target url
    #[clap(long)]
    target_request_method: Option<String>,

    /// The HTTP body for the http request the measure service will be making to the target url
    #[clap(long)]
    target_request_body: Option<String>,

    /// The HTTP headers for the http request the measure service will be making to the target url
    #[arg(value_parser = parse_key_val::<String, String>)]
    #[clap(long)]
    target_request_headers: Option<Vec<(String, String)>>,

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
    output_dir: Option<String>,

    /// Creates requests concurrently rather than sequentially
    /// and ignores the delay param
    #[clap(long)]
    flood: bool,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
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
    output_dir: Option<String>,
    average: bool,
    times: usize,
    delay: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    /// mapping from service ip to the results of the target url
    target_results: HashMap<String, Vec<MeasureResponse>>,
    /// mapping from service ip to the results of the comparison url
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
            output_dir: args.output_dir,
        })
    }

    async fn start(mut self) -> anyhow::Result<()> {
        let Jobs {
            services,
            target_url,
            target_method,
            target_body,
            target_headers,
            comparison_url,
        } = self.jobs.clone();

        for service_ip in services {
            println!("running for: {}", service_ip);
            self.run(
                service_ip,
                target_url.clone(),
                target_method.clone(),
                target_body.clone(),
                target_headers.clone(),
                comparison_url.clone(),
            )
            .await?;
        }

        let output = self.output();

        for (ip, results) in output.target_results.iter() {
            let mut builder = Builder::default();
            // Push the header row (0..self.times)
            builder.push_record(
                std::iter::once(String::from(""))
                    .chain((0..self.times).map(|i| (i + 1).to_string())),
            );

            // Push the target url and the results
            builder.push_record(
                std::iter::once(target_url.clone()).chain(
                    results
                        .iter()
                        .map(|res| format!("{}ms", res.ttfb_duration.as_millis())),
                ),
            );

            // Push the comparison url and the results if applicable
            if let Some(ref comp) = output.comparison_results {
                let comp = comp.get(ip).expect("comparison results for this ip");
                builder.push_record(
                    std::iter::once(comparison_url.as_ref().expect("comparison url").clone())
                        .chain(
                            comp.iter()
                                .map(|res| format!("{}ms", res.ttfb_duration.as_millis())),
                        ),
                );
            }

            println!("Results for service ip: {}", ip);
            println!("{}", builder.build());
        }

        if let Some(ref dir) = self.output_dir {
            // theres no other tasks running so blocking is acceptable
            std::fs::create_dir_all(dir)?;

            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut file = std::fs::File::create(format!("{}/{}.json", dir, timestamp))?;

            serde_json::to_writer(&mut file, &output)?;
        }

        Ok(())
    }

    async fn run(
        &mut self,
        service_ip: String,
        target_url: String,
        target_method: String,
        target_body: Option<String>,
        target_headers: Option<HashMap<String, String>>,
        maybe_comp: Option<String>,
    ) -> anyhow::Result<()> {
        if target_body.is_some() && target_method != "POST" {
            return Err(anyhow::anyhow!("body is only supported for POST requests"));
        }

        let req = make_request(
            &service_ip,
            &target_url,
            &target_method,
            &target_headers,
            &target_body,
        )?;

        println!("measuring target ttfb");
        self.results.insert(
            service_ip.clone(),
            Self::measure(req, self.times, self.delay).await?,
        );

        if let Some(ref url) = maybe_comp {
            let comparison_req = make_request(
                &service_ip,
                &url,
                &target_method,
                &target_headers,
                &target_body,
            )?;

            println!("measuring comparison ttfb");
            self.comparison_results
                .as_mut()
                .expect("comparison results")
                .insert(
                    service_ip.clone(),
                    Self::measure(comparison_req, self.times, self.delay).await?,
                );
        }

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
        let pb = indicatif::ProgressBar::new(times as u64);

        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

        for i in 0..times {
            let cloned = req
                .try_clone()
                .ok_or(anyhow::anyhow!("failed to clone request"))?;

            let res = cloned.send().await?.json::<MeasureResponse>().await?;

            buf.push(res);

            pb.set_position(i as u64);

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

fn make_request(
    service_ip: &String,
    target_url: &String,
    target_method: &String,
    target_headers: &Option<HashMap<String, String>>,
    target_body: &Option<String>,
) -> Result<RequestBuilder, reqwest::Error> {
    let req = ClientBuilder::new().build()?;
    let req = if target_method != "GET" {
        req.post(format!("{0}/duration", &service_ip))
            .json(&MeasureDurationRequest {
                target: target_url.clone(),
                method: target_method.clone(),
                headers: target_headers.clone(),
                body: target_body.clone(),
            })
    } else {
        req.post(format!("{0}/ttfb", &service_ip))
            .json(&MeasureRequest {
                target: target_url.clone(),
            })
    };

    Ok(req)
}

fn print_average(label: String, measure: MeasureResponse) {
    println!("URL: {:#?}", label);
    println!("Average: {}ms", measure.ttfb_duration.as_millis());
}
