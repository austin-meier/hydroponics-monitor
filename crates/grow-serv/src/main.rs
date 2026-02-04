use tokio::net::UdpSocket;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("127.0.0.1:4200").await?; 
    let mut buf = [0; 512];
    loop {
        let (len, _addr) = sock.recv_from(&mut buf).await?;
        let buf = buf[..len].to_vec();

        tokio::spawn(async move {
            process(buf).await
        });
    }
}

async fn process(data: Vec<u8>) {
    println!("{:?}", data);
}
