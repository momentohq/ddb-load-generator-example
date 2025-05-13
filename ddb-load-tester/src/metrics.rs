use std::time::Duration;

use goodmetrics::GaugeDimensions;

use crate::args::Args;

#[derive(Clone)]
pub struct Metrics {
    latency: goodmetrics::HistogramHandle,
}
impl Metrics {
    pub fn configure(args: &Args) -> Self {
        configure_metrics(args)
    }

    pub fn record_latency(&self, amount: Duration) {
        self.latency.observe(amount.as_nanos() as i64);
    }
}

fn configure_metrics(args: &Args) -> Metrics {
    fn make_metrics(args: &Args) -> Metrics {
        let factory = goodmetrics::default_gauge_factory();
        Metrics {
            latency: factory.dimensioned_gauge_histogram(
                "ddb_load_tester",
                "latency",
                GaugeDimensions::new([(
                    "target",
                    args.scenario
                        .clone()
                        .unwrap_or_else(|| "unset".to_string())
                        .to_string(),
                )]),
            ),
        }
    }

    let (endpoint, authorization) = if let Some(metrics_endpoint) = &args.metrics_endpoint {
        log::info!("using metrics endpoint: {metrics_endpoint}");
        if let Some(metrics_authorization) = &args.metrics_authorization {
            log::info!("using metrics authorization: {metrics_authorization}");
            (metrics_endpoint, metrics_authorization)
        } else {
            panic!("you must set --metrics-authorization if you set --metrics-endpoint");
        }
    } else if let Some(_metrics_authorization) = &args.metrics_authorization {
        panic!("you must set --metrics-endpoint if you set --metrics-authorization");
    } else {
        log::info!("not using metrics");
        // they won't do anything
        return make_metrics(args);
    };

    // 1. Configure your delivery destination:
    let downstream = goodmetrics::downstream::OpenTelemetryDownstream::new_with_dimensions(
        goodmetrics::downstream::get_client(
            endpoint,
            || Some(tokio_rustls::rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            }),
            goodmetrics::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient::with_origin,
        ).expect("i can make a channel"),
        // TODO: fix goodmetrics to take impl Into<String>
        Some(("api-token", authorization.parse().expect("must be able to parse header"))),
        [
                // Chronosphere requires a standard Otel Collor `service.instance.id`
                // dimension in order to ingest metrics, otherwise they are rejected.
                // We don't really "need" a distinct value, we just need something, which
                // will default to `unknown`. If we need something in the future, we can
                // add that as necessary.
                (
                    "service.instance.id",
                    std::env::var("HOSTNAME").unwrap_or("laptop".to_string()),
                ),
            ],
    );

    // 2. Connect the downstream to the gauge factory:
    let (aggregated_batch_sender, aggregated_batch_receiver) = tokio::sync::mpsc::channel(2);
    tokio::task::spawn(downstream.send_batches_forever(aggregated_batch_receiver));
    tokio::task::spawn(
        goodmetrics::default_gauge_factory()
            .clone()
            .report_gauges_forever(
                Duration::from_secs(10),
                aggregated_batch_sender,
                goodmetrics::downstream::OpentelemetryBatcher,
            ),
    );

    make_metrics(args)
}
