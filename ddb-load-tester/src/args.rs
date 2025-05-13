use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// what these metrics should be logged as
    #[arg(long)]
    pub scenario: Option<String>,
    /// Number of threads to use
    #[arg(long, default_value = "4")]
    pub threads: usize,
    /// Request rate limit per second
    #[arg(long, default_value = "4")]
    pub tps: u32,
    /// Randomness seed to generate items
    #[arg(long, default_value = "31")]
    pub seed: u64,
    /// Item count
    #[arg(long, default_value = "1000")]
    pub items: u64,
    /// Item key length
    #[arg(long, default_value = "10")]
    pub item_key_length: usize,
    /// service log level
    #[arg(long)]
    pub service_log: Option<String>,
    /// where the dynamodb accelerator is at
    /// ex: https://api.cache.developer-kenny-dev.preprod.a.momentohq.com/functions/fls/ddbaccelerator
    #[arg(long)]
    pub accelerator_url: Option<String>,
    /// The authorization header for sending metrics to an opentelemetry endpoint
    #[arg(long)]
    pub metrics_authorization: Option<String>,
    /// The authorization header name for sending metrics to an opentelemetry endpoint
    #[arg(long, default_value = "api-token")]
    pub metrics_authorization_header_name: String,
    /// The opentelemetry endpoint to which to send metrics
    #[arg(long)]
    pub metrics_endpoint: Option<String>,
}
