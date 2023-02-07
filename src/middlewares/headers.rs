use axum::{http::Request, middleware::Next, response::Response};

pub async fn check_headers<B>(mut req: Request<B>, next: Next<B>) -> Response {
    let mut headers = Vec::new();

    req.headers().clone().into_iter().for_each(|(key, value)| {

        let key = key.unwrap().to_string();
        let value = value.to_str().unwrap().to_string();

        headers.push((key, value));
        
    });

    req.extensions_mut().insert(headers);

    next.run(req).await
}
