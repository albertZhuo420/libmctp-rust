//! The SMBus specific implementation.

use crate::base_packet::{MCTPMessageBody, MCTPMessageBodyHeader, MCTPTransportHeader};
use crate::control_packet::MCTPControlMessageRequestHeader;

const HDR_VERSION: u8 = 0b001;

const MCTP_CONTROL: u8 = 0;

const MCTP_SMBUS_COMMAND_CODE: u8 = 0x0F;

// The MCTP SMBus/I2C Packet Header
bitfield! {
    struct MCTPSMBusHeader([u8]);
    u8;
    dest_read_write, set_dest_read_write: 0, 0;
    dest_slave_addr, set_dest_slave_addr : 7, 1;
    command_code, set_command_code: 15, 8;
    byte_count, set_byte_count: 23, 16;
    source_slave_addr, set_source_slave_addr: 30, 25;
    source_read_write, set_source_read_write: 24, 24;
}

impl MCTPSMBusHeader<[u8; 4]> {
    pub fn new() -> Self {
        let buf = [0; 4];
        MCTPSMBusHeader(buf)
    }
}

struct MCTPSMBusPacket<'a> {
    smbus_header: MCTPSMBusHeader<[u8; 4]>,
    base_header: MCTPTransportHeader<[u8; 4]>,
    data_bytes: &'a [MCTPMessageBody<'a>],
}

impl<'a> MCTPSMBusPacket<'a> {
    pub fn new(
        smbus_header: MCTPSMBusHeader<[u8; 4]>,
        base_header: MCTPTransportHeader<[u8; 4]>,
        data_bytes: &'a [MCTPMessageBody],
    ) -> Self {
        Self {
            smbus_header,
            base_header,
            data_bytes,
        }
    }

    pub fn to_raw_bytes(&self, buf: &mut [u8]) -> usize {
        let mut size = 0;

        buf[0..4].copy_from_slice(&self.smbus_header.0);
        size += 4;

        buf[4..8].copy_from_slice(&self.base_header.0);
        size += 4;

        for data_byte in self.data_bytes {
            size += data_byte.to_raw_bytes(&mut buf[size..]);
        }

        size
    }
}

/// The global context for MCTP SMBus operations
pub struct MCTPSMBusContext {
    address: u8,
}

impl MCTPSMBusContext {
    /// Create a new SBMust context
    ///
    /// `address`: The source address of this device
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    fn generate_transport_header(&self, dest_addr: u8) -> MCTPTransportHeader<[u8; 4]> {
        let mut base_header: MCTPTransportHeader<[u8; 4]> = MCTPTransportHeader::new(HDR_VERSION);
        base_header.set_dest_endpoint_id(dest_addr);
        base_header.set_source_endpoint_id(self.address);
        base_header.set_som(true as u8);
        base_header.set_eom(true as u8);
        base_header.set_pkt_seq(0);
        base_header.set_to(true as u8);
        base_header.set_msg_tag(0);

        base_header
    }

    fn generate_smbus_header(&self, dest_addr: u8) -> MCTPSMBusHeader<[u8; 4]> {
        let mut smbus_header: MCTPSMBusHeader<[u8; 4]> = MCTPSMBusHeader::new();
        smbus_header.set_dest_read_write(0);
        smbus_header.set_dest_slave_addr(dest_addr);
        smbus_header.set_command_code(MCTP_SMBUS_COMMAND_CODE);
        smbus_header.set_source_slave_addr(self.address);
        smbus_header.set_source_read_write(1);

        smbus_header
    }

    /// Generate a packet to get the MCTP Versions supported by a device.
    pub fn get_mctp_version_support(&self, dest_addr: u8, buf: &mut [u8]) -> usize {
        let smbus_header = self.generate_smbus_header(dest_addr);
        let base_header = self.generate_transport_header(dest_addr);
        let header: MCTPMessageBodyHeader<[u8; 1]> =
            MCTPMessageBodyHeader::new(false, MCTP_CONTROL);

        let command_header = MCTPControlMessageRequestHeader::new(false, false, 0, 0);
        let message_header = Some(&(command_header.0[..]));
        let message_data: [u8; 4] = [0x11; 4];
        let payload: [u8; 4] = [0x34; 4];
        let mic = None;

        let body: [MCTPMessageBody; 1] = [MCTPMessageBody::new(
            header,
            &message_header,
            &message_data,
            &payload,
            mic,
        )];

        let packet = MCTPSMBusPacket::new(smbus_header, base_header, &body);

        packet.to_raw_bytes(buf)
    }
}

#[cfg(test)]
mod smbus_tests {
    use super::*;

    #[test]
    fn test_generate_smbus_header() {
        const DEST_ID: u8 = 0x23;
        const SOURCE_ID: u8 = 0x23;

        let ctx = MCTPSMBusContext::new(SOURCE_ID);

        let header = ctx.generate_smbus_header(DEST_ID);
        let buf = header.0;

        // Destination slave address, bit 0 is always 0.
        assert_eq!(buf[0], DEST_ID << 1);
        // Command code, is always 0x0F
        assert_eq!(buf[1], MCTP_SMBUS_COMMAND_CODE);
        // Byte count
        // assert_eq!(buf[2], 0);
        // Source slave address, bit 0 is always 1
        assert_eq!(buf[3], SOURCE_ID << 1 | 1);
    }

    #[test]
    fn test_generate_transport_header() {
        const DEST_ID: u8 = 0x23;
        const SOURCE_ID: u8 = 0x23;

        let ctx = MCTPSMBusContext::new(SOURCE_ID);

        let header = ctx.generate_transport_header(DEST_ID);
        let buf = header.0;

        // HDR version and reserved field
        assert_eq!(buf[0], HDR_VERSION);
        // Destination endpoint ID
        assert_eq!(buf[1], DEST_ID);
        // Source endpoint ID
        assert_eq!(buf[2], SOURCE_ID);
        // SOM, EOM, Pck_seq, TO and Msg_tab
        assert_eq!(buf[3], 1 << 7 | 1 << 6 | 0 << 4 | 1 << 3 | 0);
    }

    #[test]
    fn test_get_mctp_version_support() {
        const DEST_ID: u8 = 0x23;
        const SOURCE_ID: u8 = 0x23;

        let ctx = MCTPSMBusContext::new(SOURCE_ID);
        let mut buf: [u8; 21] = [0; 21];

        let len = ctx.get_mctp_version_support(DEST_ID, &mut buf);

        assert_eq!(len, 21);
    }
}
