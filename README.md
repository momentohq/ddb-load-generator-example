# Load generator for dynamodb proxies

This application uses the AWS dynamodb sdk to drive load on a dynamodb endpoint.

It supports an `--accelerator-url`. This url is applied to the outbound SDK request
_after_ it has been signed with sigv4. This essentially grants the receiver of this
request permission to execute this request as you, on your behalf.
The originally requested URL is set on an extra pos-signature header, `x-uri`. This
has to be replaced by the proxy. If you make any mistake in the proxy, the request
will fail signature validation. The proxy is only allowed to make _exactly_ this
request as you.

Broadly, there are 3 expected scenarios:
* `--scenario dynamodb --accelerator-url https://dynamodb.us-west-2.amazonaws.com/` (or an account_id.ddb alias)
* `--scenario lambda --accelerator-url https://your_function_url.lambda-url.us-west-2.on.aws/`
* `--scenario functions --accelerator-url https://api.cache.cell-us-west-2-1.prod.a.momentohq.com/functions/fls/ddbaccelerator`

The `lambda` scenario has special case handling. Since Lambda function urls mangle
sigv4 headers, the accelerator-url interceptor renames the request headers before
sending the request. The lambda has to replace `x-uri` as well as de-prefix the
original headers in order to proxy the request to dynamodb.

The Lambda for this is in this workspace. The Dynamodb scenario is of course natively
included in this load tester. Finally, the Function for this is found in [the functions examples](https://github.com/momentohq/functions/blob/main/momento-functions/examples/dynamodb-accelerator.rs).
