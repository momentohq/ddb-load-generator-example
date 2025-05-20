/// I have to modify the request uri after it is signed. It is the Function's job to replace
/// the request uri with x-uri.
#[derive(Debug)]
pub struct ProxyInterceptorForLambda {
    proxy_uri: String,
}
impl ProxyInterceptorForLambda {
    pub fn new(value: String) -> Self {
        Self { proxy_uri: value }
    }
}
impl aws_sdk_dynamodb::config::Intercept for ProxyInterceptorForLambda {
    fn name(&self) -> &'static str {
        "ProxyForLambda"
    }

    fn modify_before_transmit(
        &self,
        context: &mut aws_sdk_dynamodb::config::interceptors::BeforeTransmitInterceptorContextMut<
            '_,
        >,
        _runtime_components: &aws_sdk_dynamodb::config::RuntimeComponents,
        _cfg: &mut aws_sdk_dynamodb::config::ConfigBag,
    ) -> Result<(), aws_sdk_dynamodb::error::BoxError> {
        let requested = context.request().uri().to_string();
        log::debug!("replacing {requested} with {proxy}", proxy = self.proxy_uri);
        *context.request_mut().uri_mut() = self
            .proxy_uri
            .clone()
            .try_into()
            .expect("must be a valid uri");

        context
            .request_mut()
            .headers_mut()
            .insert("x-uri", requested);

        // Lambda eats sigv4 headers, so we need to rename them (then unrename in the lambda)
        let headers = context.request_mut().headers_mut();
        let headers_clone = headers.clone();
        for (k, _v) in headers_clone.into_iter() {
            let v = headers.remove(k);
            headers.insert(format!("hahaha-{k}"), v.unwrap_or_default());
        }

        Ok(())
    }
}
