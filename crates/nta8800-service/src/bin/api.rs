use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("OES_API_PORT")
        .unwrap_or_else(|_| "3007".to_string())
        .parse()?;
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Open Energy Studio API listening on http://{address}");
    axum::serve(listener, nta8800_service::app()).await?;
    Ok(())
}
