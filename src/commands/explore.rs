use crate::communication::ipc::{IPCClient, IPCRequest, IPCResponse};
use crate::communication::tcp::{TCPRequest, TCPResponse};
use crate::config::Config;
use anyhow::{Result, anyhow};
use ipnet::Ipv4Net;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

pub async fn explore(config: &Config<'_>) -> Result<()> {
    println!("Exploring currently served files on the local network");
    let shared_files = explore_files(config).await?;

    for (file_key, (session_id, addr, (file, chunks))) in shared_files {
        println!("{file_key}({file})({chunks} chunks) at {session_id}({addr})");
    }

    Ok(())
}

pub async fn explore_files(
    config: &Config<'_>,
) -> Result<BTreeMap<String, (String, IpAddr, (String, usize))>> {
    let port = config.port;
    let local_session_id = match IPCClient::request(IPCRequest::Ping).unwrap_or(IPCResponse::Ok) {
        IPCResponse::Pong(id) => Some(id),
        _ => None,
    };
    let interfaces = NetworkInterface::show()?;

    let mut servers = BTreeMap::new();

    for iface in interfaces {
        for addr in iface.addr {
            if let network_interface::Addr::V4(v4) = addr {
                let net =
                    Ipv4Net::with_netmask(v4.ip, v4.netmask.expect("Should have a net mask"))?;
                if net.prefix_len() != 24 {
                    continue; // Hacky way to skip networks that don't look like LAN
                }

                let (tx, mut rx) = mpsc::channel(100);

                for ip in generate_subnet_ips(net.network(), net.prefix_len())? {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        if let Ok((session_id, files)) = send_discovery_packet(ip, port).await {
                            tx.send((ip, session_id, files))
                                .await
                                .unwrap_or_else(|_| {});
                        }
                    });
                }

                drop(tx);

                while let Some((addr, session_id, files)) = rx.recv().await {
                    if (Some(&session_id) == local_session_id.as_ref())
                        || servers.contains_key(&session_id)
                    {
                        continue;
                    }

                    servers.insert(session_id, (addr, files));
                }
            }
        }
    }

    let mut results = BTreeMap::new();

    for (session_id, (addr, files)) in servers {
        for (file_key, file) in files {
            results.insert(file_key, (session_id.clone(), addr, file));
        }
    }
    Ok(results)
}

async fn send_discovery_packet(
    ip: IpAddr,
    port: u16,
) -> Result<(String, BTreeMap<String, (String, usize)>)> {
    let result = timeout(Duration::from_millis(100), async move {
        // println!("Sending discovery packet to {ip}");
        let mut stream = TcpStream::connect(format!("{ip}:{port}")).await?;
        stream
            .write_all(
                serde_json::to_string(&TCPRequest::Discovery)
                    .expect("Should be able to serialize")
                    .as_bytes(),
            )
            .await?;

        let mut buffer = String::with_capacity(1024);
        stream.read_to_string(&mut buffer).await?;
        let response = serde_json::from_str::<TCPResponse>(&buffer)?;
        if let TCPResponse::Discovery { session_id, files } = response {
            Ok((session_id, files))
        } else {
            Err(anyhow!("Unexpected response from server"))
        }
    })
    .await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) | Err(_) => {
            // println!("Timeout or error on {ip}");
            Err(anyhow!("Timeout or error in send_discovery_packet"))
        }
    }
}

fn generate_subnet_ips(
    network_ip: Ipv4Addr,
    prefix_length: u8,
) -> Result<Vec<IpAddr>, anyhow::Error> {
    if prefix_length > 32 {
        return Err(anyhow!("Invalid prefix length"));
    }

    let mask = if prefix_length == 0 {
        0
    } else {
        u32::MAX << (32 - prefix_length)
    };
    let network = network_ip.to_bits() & mask;

    let mut ips = Vec::new();
    let total_ips = 1 << (32 - prefix_length);

    for i in 1..total_ips - 1 {
        // Exclude the network and broadcast addresses
        let ip = Ipv4Addr::from_bits(network | i);
        ips.push(IpAddr::V4(ip));
    }

    Ok(ips)
}
