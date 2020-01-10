//extern crate futures;
//extern crate hyper;
//extern crate hyper_tls;
//extern crate hyper_timeout;
//
//use std::env;
//use std::io::{self, Write};
//use std::time::Duration;
//
//use futures::Future;
//use futures::stream::Stream;
//
//use hyper::{rt, Client};
//
////use hyper::client::HttpConnector;
//use hyper_tls::HttpsConnector;
//
//use hyper_timeout::TimeoutConnector;
//
//fn main() {
//
//    let url = match env::args().nth(1) {
//        Some(url) => url,
//        None => {
//            println!("Usage: client <url>");
//            println!("Example: client https://example.com");
//            return;
//        }
//    };
//
//    let url = url.parse::<hyper::Uri>().unwrap();
//
//    rt::run(rt::lazy(|| {
//        // This example uses `HttpsConnector`, but you can also use hyper `HttpConnector`
//        //let connector = HttpConnector::new(1);
//        let https = HttpsConnector::new(1).unwrap();
//        let mut connector = TimeoutConnector::new(https);
//        connector.set_connect_timeout(Some(Duration::from_secs(5)));
//        connector.set_read_timeout(Some(Duration::from_secs(5)));
//        connector.set_write_timeout(Some(Duration::from_secs(5)));
//        let client = Client::builder().build::<_, hyper::Body>(connector);
//
//        client.get(url).and_then(|res| {
//            println!("Response: {}", res.status());
//
//            res
//                .into_body()
//                // Body is a stream, so as each chunk arrives...
//                .for_each(|chunk| {
//                    io::stdout()
//                        .write_all(&chunk)
//                        .map_err(|e| {
//                            panic!("example expects stdout is open, error={}", e)
//                        })
//                })
//        })
//        .map_err(|err| {
//            println!("Error: {}", err);
//        })
//    }));
//}
fn main() {}
