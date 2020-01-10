use std::env;
use std::time::Duration;

use hyper::{Client, body::HttpBody as _};
use tokio::io::{self, AsyncWriteExt as _};

//use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use hyper_timeout::TimeoutConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            println!("Example: client https://example.com");
            return Ok(());
        }
    };

    let url = url.parse::<hyper::Uri>().unwrap();

    // This example uses `HttpsConnector`, but you can also use hyper `HttpConnector`
    //let http = HttpConnector::new();
    let https = HttpsConnector::new();
    let mut connector = TimeoutConnector::new(https);
    connector.set_connect_timeout(Some(Duration::from_secs(5)));
    connector.set_read_timeout(Some(Duration::from_secs(5)));
    connector.set_write_timeout(Some(Duration::from_secs(5)));
    let client = Client::builder().build::<_, hyper::Body>(connector);

    let mut res = client.get(url).await?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    while let Some(chunk) = res.body_mut().data().await {
        let chunk = chunk?;
        io::stdout()
            .write_all(&chunk)
            .await?
    }
    Ok(())
}
