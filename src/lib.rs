use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_io_timeout::TimeoutStream;

use hyper::{service::Service, Uri};
use hyper::client::connect::{Connect, Connected, Connection};

mod stream;

use stream::TimeoutConnectorStream;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A connector that enforces as connection timeout
#[derive(Debug, Clone)]
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

impl<T> Service<Uri> for TimeoutConnector<T>
where
    T: Service<Uri>,
    T::Response: AsyncRead + AsyncWrite + Send + Unpin,
    T::Future: Send + 'static,
    T::Error: Into<BoxError>,
{
    type Response = TimeoutConnectorStream<T::Response>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.connector.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let read_timeout = self.read_timeout.clone();
        let write_timeout = self.write_timeout.clone();
        let connecting = self.connector.call(dst);

        if self.connect_timeout.is_none() {
            let fut = async move {
                let io = connecting.await.map_err(Into::into)?;

                let mut tm = TimeoutConnectorStream::new(TimeoutStream::new(io));
                tm.set_read_timeout(read_timeout);
                tm.set_write_timeout(write_timeout);
                Ok(tm)
            };

            return Box::pin(fut);
        }

        let connect_timeout = self.connect_timeout.expect("Connect timeout should be set");
        let timeout = timeout(connect_timeout, connecting);

        let fut = async move {
            let connecting = timeout.await.map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))?;
            let io = connecting.map_err(Into::into)?;

            let mut tm = TimeoutConnectorStream::new(TimeoutStream::new(io));
            tm.set_read_timeout(read_timeout);
            tm.set_write_timeout(write_timeout);
            Ok(tm)
        };

        Box::pin(fut)
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

impl<T> Connection for TimeoutConnector<T> {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;
    use std::time::Duration;

    use hyper::Client;
    use hyper::client::HttpConnector;

    use super::TimeoutConnector;

    #[test]
    fn test_timeout_connector() {
        let mut rt = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            // 10.255.255.1 is a not a routable IP address
            let url = "http://10.255.255.1".parse().unwrap();

            let http = HttpConnector::new();
            let mut connector = TimeoutConnector::new(http);
            connector.set_connect_timeout(Some(Duration::from_millis(1)));

            let client = Client::builder().build::<_, hyper::Body>(connector);

            client.get(url).await
        });

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
        let mut rt = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            let url = "http://example.com".parse().unwrap();

            let http = HttpConnector::new();
            let mut connector = TimeoutConnector::new(http);
            // A 1 ms read timeout should be so short that we trigger a timeout error
            connector.set_read_timeout(Some(Duration::from_millis(1)));

            let client = Client::builder().build::<_, hyper::Body>(connector);

            client.get(url).await
        });

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
