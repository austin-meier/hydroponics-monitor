use tokio::net::UdpSocket;
use std::{sync::Arc};
use tokio_postgres::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (db_client, connection) = tokio_postgres::connect(
        "host=192.168.0.47 user=planter password=planter dbname=plants",
        tokio_postgres::NoTls,).await?;
    let shared_client = Arc::new(db_client);

    /* spawn off the connection handler */
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let sock = UdpSocket::bind("127.0.0.1:4200").await?;
    let mut buf = [0; 512];
    loop {
        let (len, _addr) = sock.recv_from(&mut buf).await?;
        let buf = buf[..len].to_vec();
        let task_client = Arc::clone(&shared_client);

        tokio::spawn(async move {
            process(&task_client, buf).await
        });
    }
}

async fn process(db_client: &Client, data: Vec<u8>) {
    let result = db_client.query("SELECT * FROM SEEDS", &[]).await.unwrap();
    println!("{:?}", result);
    println!("{:?}", data);
}
