use std::io::Result;
use std::str;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use async_std::io::ReadExt;
use async_std::io::WriteExt;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use futures::stream::StreamExt;
use std::time::Duration;
use async_std::task;

static PORT_BASE: u32 = 10000;
static PORTS: u32 = 200;
static CONNECTION_COUNT: AtomicU32 = AtomicU32::new(0);
static MESSAGE_COUNT: AtomicU32 = AtomicU32::new(0);

#[async_std::main]
async fn main() -> Result<()> {
    let reporter = task::spawn(async {
        loop {
            println!("Connections = {}, Messages = {}", CONNECTION_COUNT.load(Ordering::Relaxed), MESSAGE_COUNT.load(Ordering::Relaxed));
            task::sleep(Duration::from_secs(1)).await;
        }
    });

    let mut children = vec![];
    for port_offset in 0..PORTS {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port_offset + PORT_BASE)).await?;
        children.push(task::spawn(async move {
            while let Some(tcpstream) = listener.incoming().next().await {
                CONNECTION_COUNT.fetch_add(1, Ordering::Acquire);
                task::spawn(async move {
                    let tcpstream = tcpstream.unwrap();
                    // tcpstream.set_nodelay(true).unwrap();
                    handle_connection(tcpstream).await.unwrap();
                });
            }
        }));
    }
    for child in children {
        child.await;
    }
    reporter.cancel().await.unwrap();
    Ok(())
}

async fn handle_connection(mut tcpstream: TcpStream) -> Result<()> {
    let mut data = vec![0; 128];
    // let mut counter = 0;
    loop {
        tcpstream.read_exact(&mut data[0..1]).await.unwrap();
        let len = data[0] as usize;
        tcpstream.read_exact(&mut data[0..len]).await.unwrap();
        // counter += 1;

        if str::from_utf8(&data[0..len]).unwrap().eq("exit") {
            // MESSAGE_COUNT.fetch_add(counter - 1, Ordering::Acquire);
            break;
        }
        // if counter % 100 == 0 {
            MESSAGE_COUNT.fetch_add(1, Ordering::Acquire);
            // counter = 0;
        // }
        tcpstream.write(&[len as u8]).await.unwrap();
        tcpstream.write(&data[0..len]).await.unwrap();
    }
    Ok(())
}
