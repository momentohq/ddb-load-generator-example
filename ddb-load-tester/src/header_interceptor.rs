use std::borrow::Cow;

#[derive(Debug)]
pub struct HeaderInterceptor {
    name: Cow<'static, str>,
    value: Cow<'static, str>,
}
impl HeaderInterceptor {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
impl aws_sdk_dynamodb::config::Intercept for HeaderInterceptor {
    fn name(&self) -> &'static str {
        "AddHeader"
    }

    fn modify_before_retry_loop(
        &self,
        context: &mut aws_sdk_dynamodb::config::interceptors::BeforeTransmitInterceptorContextMut<
            '_,
        >,
        _runtime_components: &aws_sdk_dynamodb::config::RuntimeComponents,
        _cfg: &mut aws_sdk_dynamodb::config::ConfigBag,
    ) -> Result<(), aws_sdk_dynamodb::error::BoxError> {
        let headers = context.request_mut().headers_mut();
        headers.insert(self.name.clone(), self.value.clone());
        log::trace!("header'd request: {:?}", context.request());

        Ok(())
    }
}
