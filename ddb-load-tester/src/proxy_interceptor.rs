use std::borrow::Cow;

/// I have to modify the request uri after it is signed. It is the Function's job to replace
/// the request uri with x-uri.
#[derive(Debug)]
pub struct ProxyInterceptor {
    proxy_uri: String,
    auth_header_name: Cow<'static, str>,
    auth_header_value: Cow<'static, str>,
}
impl ProxyInterceptor {
    pub fn new(
        value: String,
        auth_header_name: Cow<'static, str>,
        auth_header_value: Cow<'static, str>,
    ) -> Self {
        Self {
            proxy_uri: value,
            auth_header_name,
            auth_header_value,
        }
    }
}
impl aws_sdk_dynamodb::config::Intercept for ProxyInterceptor {
    fn name(&self) -> &'static str {
        "Proxy"
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
        log::trace!("replacing {requested} with {proxy}", proxy = self.proxy_uri);
        // Set the request uri to the proxy uri. This is after the request is signed, so this request
        // is proxyable and secure against modification. Make sure you trust the proxy to make this request!
        *context.request_mut().uri_mut() = self
            .proxy_uri
            .clone()
            .try_into()
            .expect("must be a valid uri");

        // Proxy uses x-uri to replace the original uri when it needs to forward the request
        context
            .request_mut()
            .headers_mut()
            .insert("x-uri", requested);

        // Include the auth header for the proxy
        context.request_mut().headers_mut().insert(
            self.auth_header_name.clone(),
            self.auth_header_value.clone(),
        );

        Ok(())
    }
}
