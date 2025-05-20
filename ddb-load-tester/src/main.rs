use std::{sync::atomic::AtomicUsize, time::Duration};

use args::Args;
use aws_config::BehaviorVersion;
use clap::Parser;
use header_interceptor::HeaderInterceptor;
use item_generator::ItemGenerator;
use load_generator_task::load_generator_task;
use metrics::Metrics;
use proxy_interceptor::ProxyInterceptor;
use proxy_interceptor_for_lambda::ProxyInterceptorForLambda;
use tokio::task::JoinSet;

mod args;
mod header_interceptor;
mod item_generator;
mod load_generator_task;
mod metrics;
mod proxy_interceptor;
mod proxy_interceptor_for_lambda;

fn main() {
    let args = Args::parse();

    let mut logger_builder = env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("debug")
            .default_write_style_or("always"),
    );
    logger_builder.init();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.threads)
        .enable_all()
        .thread_name_fn(|| {
            static I: AtomicUsize = AtomicUsize::new(0);
            format!(
                "w-{:02}",
                I.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            )
        })
        .build()
        .expect("must be able to build a runtime")
        .block_on(amain(args))
}

async fn amain(args: Args) {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let mut config =
        aws_sdk_dynamodb::config::Builder::from(&config).interceptor(HeaderInterceptor::new(
            "x-momento-authorization".to_string(),
            std::env::var("MOMENTO_AUTH_TOKEN").expect("must set MOMENTO_AUTH_TOKEN"),
        ));
    if let Some(service_log) = &args.service_log {
        log::info!("using service log level: {service_log}");
        config = config.interceptor(HeaderInterceptor::new(
            "x-momento-log".to_string(),
            service_log.clone(),
        ));
    }
    let config = if let Some(accelerator_url) = &args.accelerator_url {
        log::info!("using accelerator url: {accelerator_url}");
        if args
            .scenario
            .as_ref()
            .map(|s| s == "lambda")
            .unwrap_or_default()
        {
            // lambda requires a special case interceptor
            config.interceptor(ProxyInterceptorForLambda::new(accelerator_url.clone()))
        } else {
            config.interceptor(ProxyInterceptor::new(accelerator_url.clone()))
        }
    } else {
        config
    };
    let config = config.build();
    let metrics = Metrics::configure(&args);

    let item_generator = ItemGenerator::new(args.seed, args.items, args.item_key_length);
    let mut set = JoinSet::new();
    for _ in 0..args.threads {
        let mut rate_limiter =
            tokio::time::interval((Duration::from_secs(1) / args.tps) * args.threads as u32);
        rate_limiter.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let client = aws_sdk_dynamodb::Client::from_conf(config.clone());
        set.spawn(load_generator_task(
            client,
            rate_limiter,
            item_generator.clone(),
            metrics.clone(),
        ));
    }

    set.join_next()
        .await
        .expect("it should join")
        .expect("it should succeed");
}
