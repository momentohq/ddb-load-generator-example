use tokio::time::Interval;

use crate::{item_generator::ItemGenerator, metrics::Metrics};

pub async fn load_generator_task(
    client: aws_sdk_dynamodb::Client,
    mut rate_limiter: Interval,
    mut item_generator: ItemGenerator,
    metrics: Metrics,
) {
    loop {
        rate_limiter.tick().await;

        tokio::spawn(run_get_command(
            client.clone(),
            metrics.clone(),
            aws_sdk_dynamodb::types::AttributeValue::S(item_generator.next()),
        ));
    }
}

async fn run_get_command(
    client: aws_sdk_dynamodb::Client,
    metrics: Metrics,
    user_id: aws_sdk_dynamodb::types::AttributeValue,
) {
    let start = std::time::Instant::now();
    let reply = client
        .get_item()
        .table_name("users")
        .key("user", user_id.clone())
        .send()
        .await
        .expect("must be able to send request");
    metrics.record_latency(start.elapsed());
    log::debug!("reply: {reply:?}");
    if reply.item.is_none() {
        let request = client
            .put_item()
            .table_name("users")
            .item("user", user_id)
            .item(
                "value",
                aws_sdk_dynamodb::types::AttributeValue::S("test".to_string()),
            );
        match request.send().await {
            Ok(_) => (),
            Err(e) => {
                log::error!("failed to put item: {e:#?}");
            }
        }
    }
}
