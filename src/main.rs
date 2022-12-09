use invoice_billing_server;

#[tokio::main]
async fn main() {
    invoice_billing_server::axum().await;
}
