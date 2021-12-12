use tide::http::mime;
use tide::prelude::*;
use tide::Request;
use tide::Response;
//use tide_websockets::{Message, WebSocket};

#[derive(Clone, Serialize, Deserialize)]
struct Payload {
    data: String,
}

async fn hello(_req: Request<()>) -> tide::Result {
    let data: String = "hello".into();
    let payload = Payload { data };
    let p = serde_json::to_string(&payload);
    println!("endpoint: hello");
    match p {
        Err(_e) => {
            let response = Response::builder(500)
                .body("{\"status\": 0}")
                .content_type(mime::JSON)
                .build();
            Ok(response)
        }
        Ok(json) => {
            let response = Response::builder(200)
                .body(json)
                .content_type(mime::JSON)
                .build();
            Ok(response)
        }
    }
}

pub async fn init_http_app(bing_addr: &str, port: u16) -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/hello").get(hello);
    app.listen(format!("{}:{}", bing_addr, port)).await?;
    Ok(())
}
