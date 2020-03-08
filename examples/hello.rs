use async_std::prelude::StreamExt as _;
use futures::FutureExt as _;

async fn async_main() -> Result<(), anyhow::Error> {
    let dispatcher = ashttp::dispatcher::Dispatcher {};

    let listener = async_std::net::TcpListener::bind("127.0.0.1:8080").await?;
    eprintln!("HTTP Server has started: 127.0.0.1:8080");

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let connection = ashttp::connection::Connection::new(stream, dispatcher.clone());
        async_std::task::spawn(connection.map(|result| {
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            }
        }));
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let future = async_main();
    async_std::task::block_on(future)?;
    Ok(())
}
