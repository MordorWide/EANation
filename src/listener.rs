use openssl::ssl::{
    Ssl, SslAcceptor,
};

use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{split, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_openssl::SslStream;
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tokio_util::udp::UdpFramed;
use tracing::{debug, info, error};

use crate::client_connection::{
    ClientConnection, ClientConnectionDescriptor, ClientSenderType, ProtoType, SendDataType,
};
use crate::crypto::{create_ssl_acceptor, CryptoMode};
use crate::handler::Handler;
use crate::packet::DataPacketCodec;
use crate::sharedstate::SharedState;
pub struct Listener;

impl Listener {
    pub async fn start_tcp(
        addr: &str,
        port: u16,
        crypto: CryptoMode,
        handler: Arc<dyn Handler>,
        shared_state: Arc<SharedState>,
    ) -> tokio::task::JoinHandle<()> {
        // Create TCP listener
        let listener = match TcpListener::bind(format!("{}:{}", addr, port)).await {
            Ok(listener) => listener,
            Err(e) => {
                error!(target: "listener", "Failed to bind TCP listener on {}:{} - {}", addr, port, e);
                return tokio::spawn(async {});
            }
        };
        info!(target: "listener", "TCP listener started on {}:{}", addr, port);

        match crypto {
            CryptoMode::Plain => tokio::spawn(async move {
                loop {
                    if let Ok((socket, addr)) = listener.accept().await {
                        let handler = handler.clone();
                        let shared_state = shared_state.clone();
                        tokio::spawn(async move {
                            handle_plain_tcp_connection(socket, port, addr, handler, shared_state)
                                .await;
                        });
                    }
                }
            }),
            CryptoMode::Tls { priv_key, pub_key } => {
                // Create TLS acceptor (skipping full implementation for brevity)
                let acceptor = create_ssl_acceptor(priv_key, pub_key);
                tokio::spawn(async move {
                    loop {
                        if let Ok((socket, addr)) = listener.accept().await {
                            let handler = handler.clone();
                            let shared_state = shared_state.clone();
                            let acceptor = acceptor.clone();
                            tokio::spawn(async move {
                                handle_tls_tcp_connection(
                                    socket,
                                    port,
                                    addr,
                                    acceptor,
                                    handler,
                                    shared_state,
                                )
                                .await;
                            });
                        }
                    }
                })
            }
        }
    }

    pub async fn start_udp(
        addr: &str,
        port: u16,
        handler: Arc<dyn Handler>,
        shared_state: Arc<SharedState>,
    ) -> tokio::task::JoinHandle<()> {
        let socket = match UdpSocket::bind(format!("{}:{}", addr, port)).await {
            Ok(socket) => socket,
            Err(e) => {
            error!(target: "listener", "Failed to bind UDP listener on {}:{} - {}", addr, port, e);
            return tokio::spawn(async {});
            }
        };
        info!(target: "listener", "UDP listener started on {}:{}", addr, port);

        let atomic_socket = Arc::new(socket);
        shared_state.udp_sockets.insert(port, atomic_socket.clone());

        tokio::spawn(async move {
            let mut framed = UdpFramed::new(atomic_socket, DataPacketCodec);
            while let Some(frame) = framed.next().await {
                match frame {
                    Ok((data_packet, addr)) => {
                        let ccon = ClientConnectionDescriptor::new(
                            ProtoType::Udp,
                            handler.handler_type(),
                            port,
                            addr.ip().to_string(),
                            addr.port(),
                        );
                        debug!(target: "net", "[{}->SERVER]: {:?}", ccon.to_string(), data_packet);
                        match handler
                            .handle_packet(data_packet, ccon, shared_state.clone())
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => debug!(target: "net", "Failed to handle packet: {}", e),
                        }
                    }
                    Err(e) => debug!(target: "net", "Error reading from stream: {}", e),
                }
            }
        })
    }
}

async fn handle_plain_tcp_connection(
    socket: TcpStream,
    socket_port: u16,
    addr: SocketAddr,
    handler: Arc<dyn Handler>,
    shared_state: Arc<SharedState>,
) {
    let (read_stream, mut write_stream) = split(socket);

    let ccon_descriptor = ClientConnectionDescriptor::new(
        ProtoType::Tcp,
        handler.handler_type(),
        socket_port,
        addr.ip().to_string(),
        addr.port(),
    );
    debug!(target: "listener", "Accepted plain TCP connection from {}", ccon_descriptor.to_string());

    let (cn_tx, mut cn_rx) = mpsc::channel::<SendDataType>(32);
    // Register writer socket
    let cconn = ClientConnection::new(ccon_descriptor.to_string(), ClientSenderType::Tcp(cn_tx));

    shared_state
        .connections
        .insert(ccon_descriptor.clone(), cconn);

    // Spawn a task to handle outgoing packets for this client
    let outgoing_pkg_sstate = shared_state.clone();
    let outgoing_pkg_ccon = ccon_descriptor.clone();
    tokio::spawn(async move {
        while let Some(data) = cn_rx.recv().await {
            match data {
                SendDataType::Data(packet) => {
                    // Send packet to client
                    debug!(target: "net", "[Server=>{}]: {:?}", outgoing_pkg_ccon.to_string(), packet);
                    if let Err(err) = write_stream.write_all(&packet.to_bytes()).await {
                        debug!(target: "net", "Failed to send message to client {}: {:?}", outgoing_pkg_ccon.to_string(), err);
                        break;
                    }
                    write_stream.flush().await.unwrap();
                }
                SendDataType::Close => {
                    // Stop receiving data
                    let _ = write_stream.shutdown().await;
                    cn_rx.close();
                    break;
                }
            }
        }
        // Remove client from shared state when the task ends
        outgoing_pkg_sstate.connections.remove(&outgoing_pkg_ccon);
    });

    // Process incoming packets using Frame from Tokio
    let mut framed = FramedRead::new(read_stream, DataPacketCodec);
    while let Some(frame) = framed.next().await {
        match frame {
            Ok(data_packet) => {
                debug!(target: "net", "[{}->SERVER]: {:?}", ccon_descriptor.to_string(), data_packet);
                // Handle the packet
                match handler
                    .handle_packet(data_packet, ccon_descriptor.clone(), shared_state.clone())
                    .await
                {
                    Ok(_) => {}
                    Err(e) => debug!(target: "net", "Failed to handle packet: {}", e),
                }
            }
            Err(e) => debug!(target: "net", "Error reading from stream: {}", e),
        }
    }
    // Cleanup connection
    let con_str = ccon_descriptor.to_string();
    handler
        .connection_closed(ccon_descriptor.clone(), shared_state.clone())
        .await;

    if let Some((_, tx_channel)) = shared_state.connections.remove(&ccon_descriptor) {
        tx_channel.send(SendDataType::Close).await;
    }

    debug!(target: "listener", "Plain TCP Connection closed from {}", con_str);
}

async fn handle_tls_tcp_connection(
    socket: TcpStream,
    socket_port: u16,
    addr: SocketAddr,
    acceptor: SslAcceptor,
    handler: Arc<dyn Handler>,
    shared_state: Arc<SharedState>,
) {
    // Perform SSL handshake safely
    let ssl = match Ssl::new(acceptor.context()) {
        Ok(ssl) => ssl,
        Err(e) => {
            error!(target: "listener", "Failed to create SSL object: {}", e);
            return;
        }
    };
    let mut stream = match SslStream::new(ssl, socket) {
        Ok(stream) => stream,
        Err(e) => {
            error!(target: "listener", "Failed to create SSL stream: {}", e);
            return;
        }
    };
    // The handshake is performed here -> invalid data will fail the handshake so we can reject ill-formed connections here
    if let Err(e) = Pin::new(&mut stream).accept().await {
        debug!(target: "listener", "SSL handshake failed. The incoming data may not be SSLv3 conformant: {}", e);
        return;
    }

    // Split the stream into read and write halves
    let (read_stream, mut write_stream) = split(stream);

    let ccon_descriptor = ClientConnectionDescriptor::new(
        ProtoType::Tcp,
        handler.handler_type(),
        socket_port,
        addr.ip().to_string(),
        addr.port(),
    );
    debug!(target: "listener", "Accepted TLS TCP connection from {}", ccon_descriptor.to_string());

    let (cn_tx, mut cn_rx) = mpsc::channel::<SendDataType>(32);

    // Register writer socket
    let cconn = ClientConnection::new(ccon_descriptor.to_string(), ClientSenderType::Tcp(cn_tx));

    shared_state
        .connections
        .insert(ccon_descriptor.clone(), cconn);

    // Spawn a task to handle outgoing packets for this client
    let outgoing_pkg_sstate = shared_state.clone();
    let outgoing_pkg_ccon = ccon_descriptor.clone();
    tokio::spawn(async move {
        while let Some(data) = cn_rx.recv().await {
            match data {
                SendDataType::Data(packet) => {
                    // Send packet to client
                    debug!(target: "net", "[Server=>{}]: {:?}", outgoing_pkg_ccon.to_string(), packet);
                    if let Err(err) = write_stream.write_all(&packet.to_bytes()).await {
                        debug!(target: "net", "Failed to send message to client {}: {:?}", outgoing_pkg_ccon.to_string(), err);
                        break;
                    }
                    write_stream.flush().await.unwrap();
                }
                SendDataType::Close => {
                    // Stop receiving data
                    let _ = write_stream.shutdown().await;
                    cn_rx.close();
                    break;
                }
            }
        }
        // Remove client from shared state when the task ends
        outgoing_pkg_sstate.connections.remove(&outgoing_pkg_ccon);
    });

    // Process incoming packets using Frame from Tokio
    let mut framed = FramedRead::new(read_stream, DataPacketCodec);
    while let Some(frame) = framed.next().await {
        match frame {
            Ok(data_packet) => {
                debug!(target: "net", "[{}->SERVER]: {:?}", ccon_descriptor.to_string(), data_packet);
                // Handle the packet
                match handler
                    .handle_packet(data_packet, ccon_descriptor.clone(), shared_state.clone())
                    .await
                {
                    Ok(_) => {}
                    Err(e) => debug!(target: "net", "Failed to handle packet: {}", e),
                }
                //let _ = handler.handle_packet(data_packet, ccon_descriptor.clone(), shared_state.clone()).await;
            }
            Err(e) => debug!(target: "net", "Error reading from stream: {}", e),
        }
    }
    // Cleanup connection
    let con_str = ccon_descriptor.to_string();
    handler
        .connection_closed(ccon_descriptor.clone(), shared_state.clone())
        .await;

    if let Some((_, tx_channel)) = shared_state.connections.remove(&ccon_descriptor) {
        tx_channel.send(SendDataType::Close).await;
    }

    debug!(target: "listener", "TLS TCP Connection closed from {}", con_str);
}
