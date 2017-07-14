extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
extern crate hyper_timeout;

use std::env;
use std::time::Duration;

use futures::Future;
use futures::stream::Stream;

use hyper::Client;

//use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use hyper_timeout::TimeoutConnector;

fn main() {

    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    let url = url.parse::<hyper::Uri>().unwrap();

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    // This example uses `HttpsConnector`, but you can also use the default hyper `HttpConnector`
    //let connector = HttpConnector::new(4, &handle);
    let connector = HttpsConnector::new(4, &handle).unwrap();
    let mut tm = TimeoutConnector::new(connector, &handle);
    tm.set_connect_timeout(Some(Duration::from_secs(5)));
    tm.set_read_timeout(Some(Duration::from_secs(5)));
    tm.set_write_timeout(Some(Duration::from_secs(5)));
    let client = Client::configure().connector(tm).build(&handle);

    let get = client.get(url).and_then(|res| {
        println!("Response: {}", res.status());
        println!("Headers: \n{}", res.headers());

        res.body().concat2()
    });

    let got = core.run(get).unwrap();
    let output = String::from_utf8_lossy(&got);
    println!("{}", output);
}
