extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_io_timeout;
extern crate hyper;

use std::time::Duration;
use std::io;

use futures::Future;

use tokio::timer::Timeout;
use tokio_io_timeout::TimeoutStream;

use hyper::client::connect::{Connect, Connected, Destination};

/// A connector that enforces as connection timeout
#[derive(Debug)]
pub struct TimeoutConnector<T> {
    /// A connector implementing the `Connect` trait
    connector: T,
    /// Amount of time to wait connecting
    connect_timeout: Option<Duration>,
    /// Amount of time to wait reading response
    read_timeout: Option<Duration>,
    /// Amount of time to wait writing request
    write_timeout: Option<Duration>,
}

impl<T: Connect> TimeoutConnector<T> {
    /// Construct a new TimeoutConnector with a given connector implementing the `Connect` trait
    pub fn new(connector: T) -> Self {
        TimeoutConnector {
            connector: connector,
            connect_timeout: None,
            read_timeout: None,
            write_timeout: None,
        }
    }
}

impl<T: Connect> Connect for TimeoutConnector<T>
where
    T: Connect<Error = io::Error> + 'static,
    T::Future: 'static,
{
    type Transport = TimeoutStream<T::Transport>;
    type Error = T::Error;
    type Future = Box<Future<Item = (Self::Transport, Connected), Error = Self::Error> + Send>;

    fn connect(&self, dst: Destination) -> Self::Future {

        let read_timeout = self.read_timeout.clone();
        let write_timeout = self.write_timeout.clone();
        let connecting = self.connector.connect(dst);

        if self.connect_timeout.is_none() {
            return Box::new(connecting.map(move |(io, c)| {
                let mut tm = TimeoutStream::new(io);
                tm.set_read_timeout(read_timeout);
                tm.set_write_timeout(write_timeout);
                (tm, c)
            }));
        }

        let connect_timeout = self.connect_timeout.expect("Connect timeout should be set");
        let timeout = Timeout::new(connecting, connect_timeout);

        Box::new(timeout.then(move |res| match res {
            Ok((io, c)) => {
                let mut tm = TimeoutStream::new(io);
                tm.set_read_timeout(read_timeout);
                tm.set_write_timeout(write_timeout);
                Ok((tm, c))
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
        }))
    }
}

impl<T> TimeoutConnector<T> {
    /// Set the timeout for connecting to a URL.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_connect_timeout(&mut self, val: Option<Duration>) {
        self.connect_timeout = val;
    }

    /// Set the timeout for the response.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_read_timeout(&mut self, val: Option<Duration>) {
        self.read_timeout = val;
    }

    /// Set the timeout for the request.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_write_timeout(&mut self, val: Option<Duration>) {
        self.write_timeout = val;
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;
    use std::time::Duration;
    use futures::future;
    use tokio::runtime::current_thread::Runtime;
    use hyper::Client;
    use hyper::client::HttpConnector;
    use super::TimeoutConnector;

    #[test]
    fn test_timeout_connector() {
        let mut rt = Runtime::new().unwrap();
        let res = rt.block_on(future::lazy(|| {
            // 10.255.255.1 is a not a routable IP address
            let url = "http://10.255.255.1".parse().unwrap();

            let http = HttpConnector::new(1);
            let mut connector = TimeoutConnector::new(http);
            connector.set_connect_timeout(Some(Duration::from_millis(1)));

            let client = Client::builder().build::<_, hyper::Body>(connector);

            client.get(url)
        }));

        match res {
            Ok(_) => panic!("Expected a timeout"),
            Err(e) => {
                if let Some(io_e) = e.source().unwrap().downcast_ref::<io::Error>() {
                    assert_eq!(io_e.kind(), io::ErrorKind::TimedOut);
                } else {
                    panic!("Expected timeout error");
                }
            }
        }
    }

    #[test]
    fn test_read_timeout() {
        let mut rt = Runtime::new().unwrap();
        let res = rt.block_on(future::lazy(|| {
            let url = "http://example.com".parse().unwrap();

            let http = HttpConnector::new(1);
            let mut connector = TimeoutConnector::new(http);
            // A 1 ms read timeout should be so short that we trigger a timeout error
            connector.set_read_timeout(Some(Duration::from_millis(1)));

            let client = Client::builder().build::<_, hyper::Body>(connector);

            client.get(url)
        }));

        match res {
            Ok(_) => panic!("Expected a timeout"),
            Err(e) => {
                if let Some(io_e) = e.source().unwrap().downcast_ref::<io::Error>() {
                    assert_eq!(io_e.kind(), io::ErrorKind::TimedOut);
                } else {
                    panic!("Expected timeout error");
                }
            }
        }
    }
}
