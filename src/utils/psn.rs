use std::num::ParseIntError;

// Helper to decode hex string to bytes
pub fn dec_hex_str(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[derive(PartialEq)]
pub enum PSNSectionType {
    Body,
    Footer,
    Unknown00,
    Unknown07,
}

impl PSNSectionType {
    pub fn value(&self) -> u8 {
        match *self {
            PSNSectionType::Body => 0x00,
            PSNSectionType::Footer => 0x02,
            PSNSectionType::Unknown00 => 0x10,
            PSNSectionType::Unknown07 => 0x11,
        }
    }

    pub fn from(value: u8) -> Option<PSNSectionType> {
        match value {
            0x00 => Some(PSNSectionType::Body),
            0x02 => Some(PSNSectionType::Footer),
            0x10 => Some(PSNSectionType::Unknown00),
            0x11 => Some(PSNSectionType::Unknown07),
            _ => None,
        }
    }
}

impl std::fmt::Debug for PSNSectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match *self {
            PSNSectionType::Body => "Body",
            PSNSectionType::Footer => "Footer",
            PSNSectionType::Unknown00 => "Unknown00",
            PSNSectionType::Unknown07 => "Unknown07",
        };
        write!(f, "{}", value)
    }
}

#[derive(PartialEq)]
pub enum PSNDataType {
    NoData,
    Unknown,
    ConsoleID,
    StringData,
    TimestampMS,
    StringData2,
}

impl PSNDataType {
    pub fn value(&self) -> u8 {
        match *self {
            PSNDataType::NoData => 0x00,
            PSNDataType::Unknown => 0x01,
            PSNDataType::ConsoleID => 0x02,
            PSNDataType::StringData => 0x04,
            PSNDataType::TimestampMS => 0x07,
            PSNDataType::StringData2 => 0x08,
        }
    }

    pub fn from(value: u8) -> Option<PSNDataType> {
        match value {
            0x00 => Some(PSNDataType::NoData),
            0x01 => Some(PSNDataType::Unknown),
            0x02 => Some(PSNDataType::ConsoleID),
            0x04 => Some(PSNDataType::StringData),
            0x07 => Some(PSNDataType::TimestampMS),
            0x08 => Some(PSNDataType::StringData2),
            _ => None,
        }
    }
}

impl std::fmt::Debug for PSNDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match *self {
            PSNDataType::NoData => "NoData",
            PSNDataType::Unknown => "Unknown",
            PSNDataType::ConsoleID => "ConsoleID",
            PSNDataType::StringData => "StringData",
            PSNDataType::TimestampMS => "TimestampMS",
            PSNDataType::StringData2 => "StringData2",
        };
        write!(f, "{}", value)
    }
}

pub struct PSNTicketHeader {
    pub version_major: u8,
    pub version_minor: u8,
    pub length: u16,
    pub header_size: u16,
}

impl PSNTicketHeader {
    pub fn from_bytes(data: &[u8]) -> Result<PSNTicketHeader, &'static str> {
        if data.len() < 8 {
            return Err("Invalid PSN ticket header length");
        }
        // Check if byte 2-6 are not 0x00
        if data[2..7].iter().any(|&x| x != 0x00) {
            return Err("Invalid PSN ticket header");
        }

        // First byte should be 0xV1, where V is the major version
        let b0_v1 = data[0] % 0x10;
        if b0_v1 != 0x01 {
            return Err("Invalid PSN ticket header major version");
        }
        let version_major = data[0] / 0x10;

        // Second byte should be 0x0V, where V is the minor version
        let b1_v0 = data[1] / 0x10;
        if b1_v0 != 0x00 {
            return Err("Invalid PSN ticket header minor version");
        }
        let version_minor = data[1] % 0x10;

        // Get length
        let mut len = data[7] as u16;
        let mut header_size = 8 as u16;
        if len == 0x00 {
            if data.len() < 10 {
                return Err("Invalid PSN ticket header length");
            }
            len = u16::from_be_bytes([data[7], data[8]]);
            header_size = 10;
        }

        Ok(PSNTicketHeader {
            version_major,
            version_minor,
            length: len,
            header_size,
        })
    }
}

impl std::fmt::Debug for PSNTicketHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "PSNTicketHeader {{ VersionMajor: {}, VersionMinor: {} }}",
            self.version_major, self.version_minor
        )
    }
}

pub struct PSNTicketSectionHeader {
    pub section_type: PSNSectionType,
    pub length: u16,
    pub header_size: u16,
}

impl PSNTicketSectionHeader {
    pub fn from_bytes(data: &[u8]) -> Result<PSNTicketSectionHeader, &'static str> {
        if data.len() < 4 {
            return Err("Invalid PSN ticket section header length");
        }

        // Validate constants
        if data[0] != 0x30 {
            return Err("Invalid PSN ticket section header");
        }
        if data[2] != 0x00 {
            return Err("Invalid PSN ticket section header");
        }

        let Some(section_type) = PSNSectionType::from(data[1]) else {
            return Err("Unknown PSN ticket section header type");
        };

        // Get length
        let mut length = data[3] as u16;
        let mut header_size = 4 as u16;
        if length == 0x00 {
            if data.len() < 6 {
                return Err("Invalid PSN ticket section header length");
            }
            length = u16::from_be_bytes([data[4], data[5]]);
            header_size = 6;
        }

        Ok(PSNTicketSectionHeader {
            section_type,
            length,
            header_size,
        })
    }
}

impl std::fmt::Debug for PSNTicketSectionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "PSNTicketSectionHeader {{ SectionType: {:?}, Length: {} }}",
            self.section_type, self.length
        )
    }
}

pub struct PSNTicketData {
    pub data_type: PSNDataType,
    pub payload: Vec<u8>,
}

impl PSNTicketData {
    pub fn from_bytes(data: &[u8]) -> Result<PSNTicketData, &'static str> {
        // Check if the data has at least 4 bytes (size of the data header)
        if data.len() < 4 {
            return Err("Invalid PSN ticket data length");
        }

        // Validate constants
        if data[0] != 0x00 {
            return Err("Invalid PSN ticket data");
        }
        if data[2] != 0x00 {
            return Err("Invalid PSN ticket data");
        }

        let Some(data_type) = PSNDataType::from(data[1]) else {
            return Err("Unknown PSN ticket data type");
        };

        // Get length
        let mut data_length = data[3] as u16;
        let mut data_header_size = 4 as u16;
        if data_length == 0x00 && data_type != PSNDataType::NoData {
            if data.len() < 6 {
                return Err("Invalid PSN ticket data length");
            }
            data_length = u16::from_be_bytes([data[4], data[5]]);
            data_header_size = 6;
        }

        // Try to extract payload
        if data.len() < data_header_size as usize + data_length as usize {
            return Err("Invalid PSN ticket data length");
        }

        let payload = data
            [data_header_size as usize..data_header_size as usize + data_length as usize]
            .to_vec();

        Ok(PSNTicketData { data_type, payload })
    }
}

impl std::fmt::Debug for PSNTicketData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.data_type == PSNDataType::StringData || self.data_type == PSNDataType::StringData2 {
            write!(
                f,
                "PSNTicketData {{ DataType: {:?}: {:?} }}",
                self.data_type,
                String::from_utf8_lossy(&self.payload)
            )
        } else {
            write!(
                f,
                "PSNTicketData {{ DataType: {:?}, Payload: <bytes> }}",
                self.data_type
            )
        }
    }
}

#[derive(Debug)]
pub struct PSNTicketSection {
    pub header: PSNTicketSectionHeader,
    pub data_entries: Vec<PSNTicketData>,
}

impl PSNTicketSection {
    pub fn from_bytes(data: &[u8]) -> Result<PSNTicketSection, &'static str> {
        if data.len() < 8 {
            return Err("Invalid PSN ticket section length");
        }

        let Ok(header) = PSNTicketSectionHeader::from_bytes(&data) else {
            return Err("Invalid PSN ticket section header");
        };

        let payload_offset_end = header.header_size as usize + header.length as usize;
        if payload_offset_end > data.len() {
            return Err("Invalid PSN ticket section data length");
        }

        let mut data_entries = Vec::new();
        let mut offset = header.header_size as usize;
        while offset < payload_offset_end as usize {
            // Parse Ticket Data (including its 4 bytes header)
            let Ok(data) = PSNTicketData::from_bytes(&data[offset..]) else {
                return Err("Invalid PSN ticket section data");
            };
            offset += data.payload.len() + 4;
            data_entries.push(data);
        }

        Ok(PSNTicketSection {
            header,
            data_entries,
        })
    }
}

#[derive(Debug)]
pub struct PSNTicket {
    pub header: PSNTicketHeader,
    pub sections: Vec<PSNTicketSection>,
}

impl PSNTicket {
    pub fn from_bytes(data: &[u8]) -> Result<PSNTicket, &'static str> {
        let Ok(header) = PSNTicketHeader::from_bytes(&data) else {
            return Err("Invalid PSN ticket header");
        };

        let mut sections = Vec::new();
        let mut offset = header.header_size as usize;
        while offset < data.len() {
            // Parse Section
            let Ok(section) = PSNTicketSection::from_bytes(&data[offset..]) else {
                return Err("Invalid PSN ticket section");
            };

            offset += section.header.header_size as usize + section.header.length as usize;
            sections.push(section);
        }

        Ok(PSNTicket { header, sections })
    }
}
