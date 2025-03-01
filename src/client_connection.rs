use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::packet::DataPacket;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Fesl,
    Theater,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtoType {
    Tcp,
    Udp,
    RemoteUdp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientConnectionDescriptor {
    pub proto_type: ProtoType,
    pub service_type: ServiceType,
    pub host_port: u16,
    pub client_ip: String,
    pub client_port: u16,
}

impl ClientConnectionDescriptor {
    pub fn new(
        proto_type: ProtoType,
        service_type: ServiceType,
        host_port: u16,
        client_ip: String,
        client_port: u16,
    ) -> Self {
        Self {
            proto_type,
            service_type,
            host_port,
            client_ip,
            client_port,
        }
    }
    // from string
    pub fn from_string(client_str: &String) -> Self {
        // format: <proto_type>+<client_type>@<host_port>://<client_ip>:<client_port>
        let parts: Vec<&str> = client_str.split('@').collect();
        let proto_parts: Vec<&str> = parts[0].split('+').collect();
        let proto_type = match proto_parts[0] {
            "tcp" => ProtoType::Tcp,
            "udp" => ProtoType::Udp,
            "remoteudp" => ProtoType::RemoteUdp,
            _ => panic!("Invalid proto type"),
        };
        let client_type = match proto_parts[1] {
            "fesl" => ServiceType::Fesl,
            "theater" => ServiceType::Theater,
            _ => panic!("Invalid client type"),
        };
        let addr_parts: Vec<&str> = parts[1].split("://").collect();
        let host_port = addr_parts[0].parse::<u16>().unwrap();
        let client_parts: Vec<&str> = addr_parts[1].split(':').collect();
        let client_ip = client_parts[0].to_string();
        let client_port = client_parts[1].parse::<u16>().unwrap();
        Self {
            proto_type,
            service_type: client_type,
            host_port,
            client_ip,
            client_port,
        }
    }

    // to string
    pub fn to_string(&self) -> String {
        let proto_str = match self.proto_type {
            ProtoType::Tcp => "tcp",
            ProtoType::Udp => "udp",
            ProtoType::RemoteUdp => "remoteudp",
        };
        let client_str = match self.service_type {
            ServiceType::Fesl => "fesl",
            ServiceType::Theater => "theater",
        };
        format!(
            "{}+{}@{}://{}:{}",
            proto_str, client_str, self.host_port, self.client_ip, self.client_port
        )
    }
}

#[derive(Debug, Clone)]
pub enum SendDataType {
    Data(DataPacket),
    Close,
}

#[derive(Debug)]
pub enum ClientSenderType {
    Tcp(mpsc::Sender<SendDataType>),
    Udp(UdpSocket),
}

#[derive(Debug)]
pub struct ClientConnection {
    pub client_str: String,
    pub sender: ClientSenderType,
}

impl ClientConnection {
    pub fn new(client_str: String, sender: ClientSenderType) -> Self {
        Self { client_str, sender }
    }

    pub async fn send(&self, data: SendDataType) {
        match &self.sender {
            ClientSenderType::Tcp(sender) => {
                let _ = sender.send(data).await;
            }
            ClientSenderType::Udp(socket) => {
                match data {
                    SendDataType::Data(packet) => {
                        // Get UDP socket
                        let ccd = ClientConnectionDescriptor::from_string(&self.client_str);
                        let addr = format!("{}:{}", ccd.client_ip, ccd.client_port);
                        let _ = socket.send_to(&packet.to_bytes(), addr).await;
                    }
                    SendDataType::Close => {
                        // Do nothing
                    }
                }
            }
        }
    }
}
