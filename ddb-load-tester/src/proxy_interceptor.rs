/// I have to modify the request uri after it is signed. It is the Function's job to replace
/// the request uri with x-uri.
#[derive(Debug)]
pub struct ProxyInterceptor {
    proxy_uri: String,
    rename_auth_headers_with_hahaha_prefix_to_work_around_lambda_function_url_sigv4_mangling: bool,
}
impl ProxyInterceptor {
    pub fn new(
        value: String,
        rename_auth_headers_with_hahaha_prefix_to_work_around_lambda_function_url_sigv4_mangling: bool,
    ) -> Self {
        Self {
            proxy_uri: value,
            rename_auth_headers_with_hahaha_prefix_to_work_around_lambda_function_url_sigv4_mangling,
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
        if self.rename_auth_headers_with_hahaha_prefix_to_work_around_lambda_function_url_sigv4_mangling {
            let headers = context
                .request_mut()
                .headers_mut();
            let headers_clone = headers.clone();
            for (k, _v) in headers_clone.into_iter() {
                let v = headers.remove(k);
                headers.insert(format!("hahaha-{k}"), v.unwrap_or_default());
            }
        }

        log::debug!("sending request: {:#?}", context.request());

        Ok(())
    }
}
