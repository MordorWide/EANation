use indexmap::IndexMap;
use std::{fmt, str};
use tokio::net::UdpSocket;

use std::sync::Arc;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait SenderType: Clone {
    async fn send(&self, packet: DataPacket) -> Result<(), &'static str>;
}

#[async_trait::async_trait]
impl SenderType for mpsc::Sender<DataPacket> {
    async fn send(&self, packet: DataPacket) -> Result<(), &'static str> {
        if let Ok(_) = self.send(packet).await {
            return Ok(());
        } else {
            return Err("Failed to send packet");
        }
    }
}

#[async_trait::async_trait]
impl SenderType for Arc<UdpSocket> {
    async fn send(&self, packet: DataPacket) -> Result<(), &'static str> {
        let packet_bytes = packet.to_bytes();
        let socket = self.as_ref();
        if let Ok(_) = socket.send(&packet_bytes).await {
            return Ok(());
        } else {
            return Err("Failed to send packet");
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum PacketType {
    FESL,
    THEATRE,
}

#[derive(PartialEq, Eq, Clone)]
pub enum PacketMode {
    FeslPingOrTheaterResponse,
    FeslSinglePacketResponse,
    FeslMultiPacketResponse,
    FeslSinglePacketRequest,
    FeslMultiPacketRequest,
    TheaterRequest,
}

#[derive(PartialEq, Eq, Clone)]
pub enum DataMode {
    FESL_FSYS,
    FESL_PNOW,
    FESL_ACCT,
    FESL_RECP,
    FESL_ASSO,
    FESL_PRES,
    FESL_RANK,
    FESL_XMSG,
    FESL_MTRX,

    THEATER_CONN,
    THEATER_USER,
    THEATER_ECNL,
    THEATER_EGAM,
    THEATER_GDAT,
    THEATER_LLST,
    THEATER_LDAT, // Only response
    THEATER_GLST,
    THEATER_CGAM,
    THEATER_PENT,
    THEATER_EGRQ,
    THEATER_QENT,
    THEATER_EGRS,
    THEATER_EGEG,
    THEATER_UBRA,
    THEATER_UGAM,
    THEATER_RGAM,
    THEATER_PLVT,
    THEATER_UGDE,
    THEATER_PING,

    THEATER_ECHO, // UDP
}

impl DataMode {
    pub fn value(&self) -> &str {
        match *self {
            DataMode::FESL_FSYS => "fsys",
            DataMode::FESL_PNOW => "pnow",
            DataMode::FESL_ACCT => "acct",
            DataMode::FESL_RECP => "recp",
            DataMode::FESL_ASSO => "asso",
            DataMode::FESL_PRES => "pres",
            DataMode::FESL_RANK => "rank",
            DataMode::FESL_XMSG => "xmsg",
            DataMode::FESL_MTRX => "mtrx",

            DataMode::THEATER_CONN => "CONN",
            DataMode::THEATER_USER => "USER",
            DataMode::THEATER_ECNL => "ECNL",
            DataMode::THEATER_EGAM => "EGAM",
            DataMode::THEATER_GDAT => "GDAT",
            DataMode::THEATER_LLST => "LLST",
            DataMode::THEATER_LDAT => "LDAT", // Only response
            DataMode::THEATER_GLST => "GLST",
            DataMode::THEATER_CGAM => "CGAM",
            DataMode::THEATER_PENT => "PENT",
            DataMode::THEATER_EGRQ => "EGRQ",
            DataMode::THEATER_QENT => "QENT",
            DataMode::THEATER_EGRS => "EGRS",
            DataMode::THEATER_EGEG => "EGEG",
            DataMode::THEATER_UBRA => "UBRA",
            DataMode::THEATER_UGAM => "UGAM",
            DataMode::THEATER_RGAM => "RGAM",
            DataMode::THEATER_PLVT => "PLVT",
            DataMode::THEATER_UGDE => "UGDE",
            DataMode::THEATER_PING => "PING",
            DataMode::THEATER_ECHO => "ECHO", // UDP
        }
    }
    pub fn from_value(value: &str) -> Result<Self, &'static str> {
        match value {
            "fsys" => Ok(DataMode::FESL_FSYS),
            "pnow" => Ok(DataMode::FESL_PNOW),
            "acct" => Ok(DataMode::FESL_ACCT),
            "recp" => Ok(DataMode::FESL_RECP),
            "asso" => Ok(DataMode::FESL_ASSO),
            "pres" => Ok(DataMode::FESL_PRES),
            "rank" => Ok(DataMode::FESL_RANK),
            "xmsg" => Ok(DataMode::FESL_XMSG),
            "mtrx" => Ok(DataMode::FESL_MTRX),

            "CONN" => Ok(DataMode::THEATER_CONN),
            "USER" => Ok(DataMode::THEATER_USER),
            "ECNL" => Ok(DataMode::THEATER_ECNL),
            "EGAM" => Ok(DataMode::THEATER_EGAM),
            "GDAT" => Ok(DataMode::THEATER_GDAT),
            "LLST" => Ok(DataMode::THEATER_LLST),
            "LDAT" => Ok(DataMode::THEATER_LDAT), // Only response
            "GLST" => Ok(DataMode::THEATER_GLST),
            "CGAM" => Ok(DataMode::THEATER_CGAM),
            "PENT" => Ok(DataMode::THEATER_PENT),
            "EGRQ" => Ok(DataMode::THEATER_EGRQ),
            "QENT" => Ok(DataMode::THEATER_QENT),
            "EGRS" => Ok(DataMode::THEATER_EGRS),
            "EGEG" => Ok(DataMode::THEATER_EGEG),
            "UBRA" => Ok(DataMode::THEATER_UBRA),
            "UGAM" => Ok(DataMode::THEATER_UGAM),
            "RGAM" => Ok(DataMode::THEATER_RGAM),
            "PLVT" => Ok(DataMode::THEATER_PLVT),
            "UGDE" => Ok(DataMode::THEATER_UGDE),
            "PING" => Ok(DataMode::THEATER_PING),

            "ECHO" => Ok(DataMode::THEATER_ECHO), // UDP
            _ => Err("Invalid data mode"),
        }
    }
}
impl std::fmt::Debug for DataMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DataMode::FESL_FSYS => write!(f, "FSYS"),
            DataMode::FESL_PNOW => write!(f, "PNOW"),
            DataMode::FESL_ACCT => write!(f, "ACCT"),
            DataMode::FESL_RECP => write!(f, "RECP"),
            DataMode::FESL_ASSO => write!(f, "ASSO"),
            DataMode::FESL_PRES => write!(f, "PRES"),
            DataMode::FESL_RANK => write!(f, "RANK"),
            DataMode::FESL_XMSG => write!(f, "XMSG"),
            DataMode::FESL_MTRX => write!(f, "MTRX"),

            DataMode::THEATER_CONN => write!(f, "CONN"),
            DataMode::THEATER_USER => write!(f, "USER"),
            DataMode::THEATER_ECNL => write!(f, "ECNL"),
            DataMode::THEATER_EGAM => write!(f, "EGAM"),
            DataMode::THEATER_GDAT => write!(f, "GDAT"),
            DataMode::THEATER_LLST => write!(f, "LLST"),
            DataMode::THEATER_LDAT => write!(f, "LDAT"), // Only response
            DataMode::THEATER_GLST => write!(f, "GLST"),
            DataMode::THEATER_CGAM => write!(f, "CGAM"),
            DataMode::THEATER_PENT => write!(f, "PENT"),
            DataMode::THEATER_EGRQ => write!(f, "EGRQ"),
            DataMode::THEATER_QENT => write!(f, "QENT"),
            DataMode::THEATER_EGRS => write!(f, "EGRS"),
            DataMode::THEATER_EGEG => write!(f, "EGEG"),
            DataMode::THEATER_UBRA => write!(f, "UBRA"),
            DataMode::THEATER_UGAM => write!(f, "UGAM"),
            DataMode::THEATER_RGAM => write!(f, "RGAM"),
            DataMode::THEATER_PLVT => write!(f, "PLVT"),
            DataMode::THEATER_UGDE => write!(f, "UGDE"),
            DataMode::THEATER_PING => write!(f, "PING"),
            DataMode::THEATER_ECHO => write!(f, "ECHO"), // UDP
        }
    }
}

impl PacketMode {
    pub fn value(&self) -> u8 {
        match *self {
            PacketMode::FeslPingOrTheaterResponse => 0x00,
            PacketMode::FeslSinglePacketResponse => 0x80,
            PacketMode::FeslMultiPacketResponse => 0xB0,
            PacketMode::FeslSinglePacketRequest => 0xC0,
            PacketMode::FeslMultiPacketRequest => 0xF0,
            PacketMode::TheaterRequest => 0x40,
        }
    }
    pub fn from_value(value: u8) -> Result<Self, &'static str> {
        match value {
            0x00 => Ok(PacketMode::FeslPingOrTheaterResponse),
            0x80 => Ok(PacketMode::FeslSinglePacketResponse),
            0xB0 => Ok(PacketMode::FeslMultiPacketResponse),
            0xC0 => Ok(PacketMode::FeslSinglePacketRequest),
            0xF0 => Ok(PacketMode::FeslMultiPacketRequest),
            0x40 => Ok(PacketMode::TheaterRequest),
            _ => Err("Invalid packet mode"),
        }
    }
}
impl std::fmt::Debug for PacketMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PacketMode::FeslPingOrTheaterResponse => write!(f, "FeslPingOrTheaterResponse"),
            PacketMode::FeslSinglePacketResponse => write!(f, "FeslSinglePacketResponse"),
            PacketMode::FeslMultiPacketResponse => write!(f, "FeslMultiPacketResponse"),
            PacketMode::FeslSinglePacketRequest => write!(f, "FeslSinglePacketRequest"),
            PacketMode::FeslMultiPacketRequest => write!(f, "FeslMultiPacketRequest"),
            PacketMode::TheaterRequest => write!(f, "TheaterRequest"),
        }
    }
}

const PACKET_DATA_ENTRY_SPLIT: u8 = '\n' as u8;
const PACKET_DATA_KV_SPLIT: u8 = '=' as u8;
const PACKET_DATA_STOP: u8 = '\0' as u8;

#[derive(Clone)]
pub struct DataPacket {
    pub mode: DataMode,
    pub packet_mode: PacketMode,
    pub packet_id: u32,
    pub data: IndexMap<String, String>,
}

impl fmt::Debug for DataPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mask = String::from("***");
        let filtered_data: IndexMap<&String, &String> = self
            .data
            .iter()
            .map(|(key, value)| {
                // Don't log the password
                if key == "password" {
                    (key, &mask)
                } else {
                    (key, value)
                }
            })
            .collect();

        f.debug_struct("DataPacket")
            .field("mode", &self.mode)
            .field("packet_mode", &self.packet_mode)
            .field("packet_id", &self.packet_id)
            .field("data", &filtered_data)
            .finish()
    }
}

impl DataPacket {
    pub fn new(
        mode: DataMode,
        packet_mode: PacketMode,
        packet_id: u32,
        data: IndexMap<String, String>,
    ) -> Self {
        DataPacket {
            mode,
            packet_mode,
            packet_id,
            data,
        }
    }

    pub fn dec_plasma_str(encoded_string: &String) -> String {
        encoded_string
            .trim_matches('"')
            .replace("%22", "\"")
            .replace("%3d", "=")
            .replace("%25", "%")
    }

    pub fn enc_plasma_str(decoded_string: &String) -> String {
        // Escape % = and "
        let encoded_string = decoded_string
            .replace("%", "%25")
            .replace('"', "%22")
            .replace('=', "%3d");
        // Add quotes if there is a space
        if encoded_string.contains(" ") {
            format!("\"{}\"", encoded_string)
        } else {
            encoded_string
        }
    }

    fn parse_payload_to_hm(payload: Vec<u8>) -> Result<IndexMap<String, String>, &'static str> {
        // Parse bytestream to hashmap
        let mut hm = IndexMap::new();

        for entry_bytes in payload.split(|es_byte| es_byte == &PACKET_DATA_ENTRY_SPLIT) {
            if entry_bytes.len() == 0
                || (entry_bytes.len() == 1 && entry_bytes[0] == PACKET_DATA_STOP)
            {
                // We found the end byte
                continue;
            }

            // Find split byte
            let Some(split_index) = entry_bytes
                .iter()
                .position(|cur_byte| cur_byte == &PACKET_DATA_KV_SPLIT)
            else {
                return Err("Failed to find split byte");
            };

            // Split key and value
            let (key, value) = entry_bytes.split_at(split_index);

            let key = String::from_utf8_lossy(&key[..]).to_string();
            let value = String::from_utf8_lossy(&value[1..]).to_string();
            // Escape data entries
            hm.insert(
                DataPacket::dec_plasma_str(&key),
                DataPacket::dec_plasma_str(&value),
            );
        }

        Ok(hm)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Option<(usize, Self)>, &'static str> {
        // Check if the packet _header_ is complete
        if bytes.len() < 12 {
            //println!("Invalid packet length");
            return Ok(None); //Err("Invalid packet length");
        }

        // First: check if the packet is complete...
        // Reported length of packet
        let reported_packet_length: usize =
            u32::from_be_bytes([bytes[8 + 0], bytes[8 + 1], bytes[8 + 2], bytes[8 + 3]]) as usize;

        // Check if packet length is correct
        let actual_bytedata_length: usize = bytes.len();
        if reported_packet_length > actual_bytedata_length {
            println!(
                "Reported packet length is bigger than the received data. Packet is incomplete!"
            );
            return Ok(None); //return Err("Reported packet length is bigger than the received data. Packet is incomplete!");
        }

        // We can now parse the header :)
        // Read first bytes of header
        // fsys etc.
        let Ok(mode_str) = str::from_utf8(&bytes[0..4]) else {
            println!("Unexpected data mode byte sequence");
            return Ok(None); //return Err("Unexpected data mode byte sequence");
        };
        // Convert to DataMode
        let Ok(mode) = DataMode::from_value(mode_str) else {
            println!("Unknown data mode: {}", mode_str);
            return Ok(None); //return Err("Unknown data mode");
        };

        // Request or Response?
        let Ok(packet_mode) = PacketMode::from_value(bytes[4]) else {
            println!("Invalid packet mode");
            return Ok(None); //return Err("Invalid packet mode");
        };

        let packet_id: u32 = u32::from_be_bytes([0x00, bytes[4 + 1], bytes[4 + 2], bytes[4 + 3]]);

        // Parse the payload
        let payload: Vec<u8> = bytes[12..reported_packet_length].to_vec();
        let Ok(data) = Self::parse_payload_to_hm(payload) else {
            println!("Failed to parse payload");
            return Ok(None); //return Err("Failed to parse payload");
        };

        Ok(Some((
            reported_packet_length,
            DataPacket {
                mode,
                packet_mode,
                packet_id,
                data,
            },
        )))
    }

    fn serialize_hm_to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        for (key, value) in self.data.iter() {
            let enc_key = DataPacket::enc_plasma_str(key);
            let enc_value = DataPacket::enc_plasma_str(value);
            payload.extend(enc_key.as_bytes());
            payload.push(PACKET_DATA_KV_SPLIT);
            payload.extend(enc_value.as_bytes());
            payload.push(PACKET_DATA_ENTRY_SPLIT);
        }

        payload.push(PACKET_DATA_STOP);

        payload
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // Add data mode
        bytes.extend(self.mode.value().as_bytes());
        // Add packet mode
        bytes.push(self.packet_mode.value());

        // Add cut-off packet ID
        let full_packet_id_bytes = self.packet_id.to_be_bytes();
        bytes.extend(
            [
                full_packet_id_bytes[1],
                full_packet_id_bytes[2],
                full_packet_id_bytes[3],
            ]
            .iter(),
        );

        // Generate payload first...
        let payload = self.serialize_hm_to_payload();

        // Add packet length (12 bytes header + payload length)
        let payload_length: u32 = payload.len().try_into().unwrap();
        let packet_length = (12 + payload_length).to_be_bytes();
        bytes.extend(packet_length.iter());

        bytes.extend(payload.iter());

        bytes
    }
}

use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

pub struct DataPacketCodec;

impl DataPacketCodec {
    pub fn new() -> Self {
        DataPacketCodec {}
    }
}

impl Decoder for DataPacketCodec {
    type Item = DataPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Result<(usize, Self), &'static str>
        let parse_attempt = DataPacket::from_bytes(src.to_vec());

        match parse_attempt {
            Ok(Some((n_read, datapacket))) => {
                src.advance(n_read);
                Ok(Some(datapacket))
            }
            Ok(None) => Ok(None),
            Err(error_data) => {
                println!("Error parsing packet: {}", error_data);
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    error_data,
                ))
            }
        }
    }
}

use tokio_util::codec::Encoder;

impl Encoder<DataPacket> for DataPacketCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: DataPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Convert the packet to bytes
        let datapacket_bytes = item.to_bytes();

        // Reserve space in the buffer.
        dst.reserve(datapacket_bytes.len());

        // Write the length and string to the buffer.
        dst.extend_from_slice(&datapacket_bytes);
        Ok(())
    }
}
