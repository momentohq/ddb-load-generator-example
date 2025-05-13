use std::{collections::HashMap, env, str::FromStr};

use lambda_http::{http::request::Parts, Body, Error, Request, Response};
use redis::Commands;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let (
        Parts {
            method: _,
            uri: _,
            version: _,
            headers,
            extensions: _,
            ..
        },
        body,
    ) = event.into_parts();
    let body = match body {
        Body::Empty => vec![],
        Body::Text(s) => s.into_bytes(),
        Body::Binary(b) => b,
    };

    let mut hahaha_real_headers = HashMap::new();
    let mut headers_of_unknown_provenance = HashMap::new();
    for (k, v) in headers {
        if let Some(k) = k {
            let k = k.to_string();
            let v = v.to_str()?.to_string();
            if let Some(stripped) = k.strip_prefix("hahaha-") {
                hahaha_real_headers.insert(stripped.to_string(), v);
            } else {
                headers_of_unknown_provenance.insert(k, v);
            }
        }
    }
    headers_of_unknown_provenance.extend(hahaha_real_headers);
    // and now we have replaced the clobbered headers from who knows where with the real
    // ones we hahaha-'d in the client to work around what aws would do to the headers otherwise.
    let mut headers = headers_of_unknown_provenance;

    tracing::info!("headers: {headers:?}");

    // The target header comes from the AWS SDK and is the api call being made.
    let action = headers
        .get("x-amz-target")
        .ok_or("missing x-amz-target header")?
        .clone();

    // The x-uri header is the custom header we added to the request _after it was signed_,
    // as we changed the request's target uri to _this Function_.
    let proxy_uri = headers.get("x-uri").ok_or("missing x-uri header")?.clone();
    headers.insert(
        "host".to_string(),
        lambda_http::http::Uri::from_str(&proxy_uri)?
            .host()
            .unwrap_or_default()
            .to_string(),
    );

    let response = match action.as_str() {
        "DynamoDB_20120810.GetItem" => handle_get_item(body, headers, &proxy_uri).await?,
        other => {
            // We could invalidate the cache on a putitem to the same key, but that's omitted here for brevity.
            handle_all_other_ddb_calls(other, body, headers, &proxy_uri).await?
        }
    };

    Ok(response)
}

/// DynamoDB value type for keys
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum KeyValue {
    /// S value
    #[serde(rename = "S")]
    String(String),
    /// N value
    #[serde(rename = "N")]
    Number(i64),
    /// B value
    #[serde(rename = "B")]
    Binary(Vec<u8>),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CachedResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
impl From<CachedResponse> for lambda_http::http::Response<lambda_http::Body> {
    fn from(cached_response: CachedResponse) -> Self {
        let mut response = lambda_http::http::Response::builder().status(cached_response.status);
        for (key, value) in cached_response.headers {
            response = response.header(key, value);
        }
        response
            .body(cached_response.body.into())
            .expect("must be able to create response")
    }
}
impl From<lambda_http::http::Response<Vec<u8>>> for CachedResponse {
    fn from(response: lambda_http::http::Response<Vec<u8>>) -> Self {
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.into_body().into_iter().collect();
        CachedResponse {
            status,
            headers,
            body,
        }
    }
}

async fn handle_get_item(
    body: Vec<u8>,
    headers: impl IntoIterator<Item = (String, String)>,
    proxy_uri: &str,
) -> Result<lambda_http::http::Response<lambda_http::Body>, Error> {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct GetItemRequest {
        #[serde(rename = "TableName")]
        table_name: String,
        #[serde(rename = "Key")]
        key: HashMap<String, KeyValue>,
    }

    let request: GetItemRequest = serde_json::from_slice(&body)?;
    tracing::info!("GetItem {request:?}");

    let cache_key: String = serde_json::to_string(&request)?;

    let client = redis::Client::open(
        // like "rediss://ddbcache-2rmqrn.serverless.usw2.cache.amazonaws.com:6379/#insecure",
        env::var("REDIS_URL").expect("must set REDIS_URL environment variable"),
    )?;
    let mut connection = client.get_connection()?;

    Ok(match connection.get(&cache_key)? {
        Some(hit) => {
            let hit: Vec<u8> = hit;
            let hit = serde_json::from_slice::<CachedResponse>(&hit)?;
            tracing::info!("Cache hit for {cache_key}");
            hit.into()
        }
        None => {
            tracing::info!("Cache miss for {cache_key} -> {proxy_uri}");
            let response = reqwest::Client::new()
                .post(proxy_uri)
                .headers(
                    headers
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                k.try_into().expect("header must be header"),
                                v.try_into().expect("header value must be header value"),
                            )
                        })
                        .collect(),
                )
                .body(body)
                .send()
                .await?;

            let status: u16 = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response.bytes().await?.to_vec();
            let response = CachedResponse {
                status,
                headers,
                body,
            };

            let _: () = connection.set_ex(&cache_key, serde_json::to_vec(&response)?, 60)?;
            response.into()
        }
    })
}

async fn handle_all_other_ddb_calls(
    action: &str,
    body: Vec<u8>,
    headers: impl IntoIterator<Item = (String, String)>,
    proxy_uri: &str,
) -> Result<lambda_http::http::Response<lambda_http::Body>, Error> {
    tracing::info!("other action: {action} -> {proxy_uri}");
    let request = reqwest::Client::new()
        .post(proxy_uri)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.try_into().expect("header must be header"),
                        v.try_into().expect("header value must be header value"),
                    )
                })
                .collect(),
        )
        .body(body);
    tracing::info!("request: {request:?}");
    let response = request.send().await?;
    let status: u16 = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = response.bytes().await?.to_vec();
    let response = CachedResponse {
        status,
        headers,
        body,
    };
    Ok(response.into())
}
