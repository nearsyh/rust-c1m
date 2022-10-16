use std::io::Result;

use async_std::io::ReadExt;
use std::time::Duration;
use async_std::io::WriteExt;
use async_std::net::TcpStream;
use async_std::task;
use rand::prelude::*;

static PORT_BASE: u32 = 10000;
static PORTS: u32 = 200;
static CONNECTION_NUMBER: u32 = 2000000;
static MESSAGE_NUMBER: u32 = 100000;
static SLEEP_MS: u64 = 60000;

#[async_std::main]
async fn main() -> Result<()> {
    let mut children = vec![];
    for port_offset in 0..PORTS {
        let port = PORT_BASE + port_offset;
        for _ in 0..(CONNECTION_NUMBER/PORTS) {
            let mut rng = rand::thread_rng();
            let random: f64 = rng.gen();
            children.push(task::spawn(async move {
                task::sleep(Duration::from_millis((random * 60000f64) as u64)).await;

                let mut tcpstream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
                let mut data = vec![0; 128];
                // tcpstream.set_nodelay(true).unwrap();
                for _ in 0..MESSAGE_NUMBER {
                    tcpstream.write(&[12]).await.unwrap();
                    tcpstream.write_all(b"Hello World!").await.unwrap();
                    // tcpstream.flush().await.unwrap();
                    tcpstream.read_exact(&mut data[0..1]).await.unwrap();
                    let len = data[0] as usize;
                    tcpstream.read_exact(&mut data[0..len]).await.unwrap();
                    task::sleep(Duration::from_millis(SLEEP_MS)).await;
                }
                tcpstream.write_all(&[4]).await.unwrap();
                tcpstream.write_all(b"exit").await.unwrap();
            }));
        }
    }
    for child in children {
        child.await;
    }
    Ok(())
}
