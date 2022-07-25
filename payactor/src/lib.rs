use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_examples_payments::*;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct PayactorActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for PayactorActor {

    /// Returns a greeting, "Hello World", in the response body.
    /// If the request contains a query parameter 'name=NAME', the
    /// response is changed to "Hello NAME"
    async fn handle_request(
        &self,
        _ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        let count = form_urlencoded::parse(req.query_string.as_bytes())
            .find(|(n, _)| n == "count")
            .map(|(_, v)| v.parse::<u32>().unwrap())
            .ok_or(RpcError::Other("parse count err".to_string()))?;

        let ok  = self.checkout(_ctx, count).await?;

        Ok(HttpResponse {
            body: format!("success paid txid {}", ok).as_bytes().to_vec(),
            ..Default::default()
        })
    }
}

impl PayactorActor {
    async fn checkout(&self, ctx: &Context, amount: u32) -> RpcResult<String> {
        // verify that items are in stock,
        // and apply other validation rules on order
        // self.verify_order(order);
        // calculate the order amount and tax
        let payment_request = &AuthorizePaymentRequest { amount:10, ..AuthorizePaymentRequest::default() };

        // submit request to Payments provider
        let provider = PaymentsSender::new();
        let auth_response = provider.authorize_payment(ctx, payment_request).await?;
        if  !auth_response.success {
            /* handle not authorized */
            return Result::Err(RpcError::from("auth response failed".to_string()));
        }
            // ask Payaments provider to complete the transaction
        let confirmation = provider.complete_payment(ctx, &CompletePaymentRequest {
                auth_code: auth_response.auth_code.unwrap(),
                ..Default::default()
        }).await?;
        if !confirmation.success {
            /* handle payment failed */
            return Result::Err(RpcError::from("auth response failed".to_string()));
        }
        Ok(confirmation.txid)
    }
}

