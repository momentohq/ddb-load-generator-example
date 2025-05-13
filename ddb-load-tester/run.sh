#!/bin/bash

METRICS_ENDPOINT=your.opentelemetry.endpoint
METRICS_AUTHORIZATION=your_opentelemetry_authorization_token

RUST_LOG=info MOMENTO_AUTH_TOKEN=your_momento_token \
cargo run --release -- \
  --threads 1 \
  --tps 1000 \
  --metrics-endpoint $METRICS_ENDPOINT \
  --metrics-authorization $METRICS_AUTHORIZATION \
  --seed 31 \
  --items 10000 \
  --item-key-length 128 \
  $@
#  --scenario functions \
#  --accelerator-url https://api.cache.developer-kenny-dev.preprod.a.momentohq.com/functions/fls/ddbaccelerator
