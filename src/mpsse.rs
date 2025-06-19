use anyhow::Result;

use crate::DeviceType;

#[async_trait::async_trait]
pub trait MpsseInterface {
    async fn initialize_mpsse(&self) -> Result<()>;
    async fn synchronize_mpsse(&self) -> Result<()>;
    async fn set_low_data_bits(&self, value: u8, direction: u8) -> Result<()>;
    async fn set_high_data_bits(&self, value: u8, direction: u8) -> Result<()>;
    async fn enable_3phase_clocking(&self) -> Result<()>;
    async fn disable_3phase_clocking(&self) -> Result<()>;
    async fn set_frequency(&self, frequency: u32) -> Result<()>;
    async fn set_clock(&self, divisor: u16, clkdiv: Option<bool>) -> Result<()>;
    fn clock_divisor(&self, frequency: u32) -> (u16, Option<bool>);
}

#[async_trait::async_trait]
impl MpsseInterface for crate::Interface {
    fn clock_divisor(&self, frequency: u32) -> (u16, Option<bool>) {
        match self.device_type {
            DeviceType::FT2232C => ((6_000_000 / frequency - 1) as u16, None),
            DeviceType::FT2232H | DeviceType::FT4232H | DeviceType::FT232H => {
                if frequency <= 6_000_000 {
                    ((6_000_000 / frequency - 1) as u16, Some(true))
                } else {
                    ((30_000_000 / frequency - 1) as u16, Some(false))
                }
            }
            _ => panic!("Unknown device type: {:?}", self.device_type),
        }
    }

    async fn initialize_mpsse(&self) -> Result<()> {
        self.purge_all().await?;
        self.set_bitmode(0, crate::Bitmode::Reset).await?;
        self.set_bitmode(0, crate::Bitmode::Mpsse).await?;
        self.purge_all().await?;
        self.synchronize_mpsse().await?;
        self.purge_all().await?;

        Ok(())
    }

    async fn synchronize_mpsse(&self) -> Result<()> {
        self.write_all(vec![EnableLoopback::byte(), Synchronize::byte(), DisableLoopback::byte()]).await?;

        let mut buf = [0u8; 2];
        self.read_all(&mut buf).await?;

        if !(buf[0] == 0xfa && buf[1] == Synchronize::byte()) {
            return Err(anyhow::Error::msg(format!("invalid synchronization byte {:x?}", buf)));
        }

        Ok(())
    }

    async fn set_frequency(&self, frequency: u32) -> Result<()> {
        let (divisor, clkdiv) = self.clock_divisor(frequency);
        self.set_clock(divisor, clkdiv).await?;

        Ok(())
    }

    async fn set_clock(&self, divisor: u16, clkdiv: Option<bool>) -> Result<()> {
        let mut cmd = Vec::new();

        match clkdiv {
            Some(true) => cmd.push(EnableClockDivide::byte()),
            Some(false) => cmd.push(DisableClockDivide::byte()),
            None => {}
        };

        cmd.push(SetClockFrequency::byte());
        cmd.extend_from_slice(&divisor.to_le_bytes());

        self.write_all(cmd).await?;

        Ok(())
    }

    async fn enable_3phase_clocking(&self) -> Result<()> {
        self.write_all(vec![Enable3PhaseClocking::byte()]).await?;

        Ok(())
    }

    async fn disable_3phase_clocking(&self) -> Result<()> {
        self.write_all(vec![Disable3PhaseClocking::byte()]).await?;

        Ok(())
    }

    async fn set_low_data_bits(&self, value: u8, direction: u8) -> Result<()> {
        self.write_all(vec![SetDataBitsLowByte::byte(), value, direction]).await?;

        Ok(())
    }

    async fn set_high_data_bits(&self, value: u8, direction: u8) -> Result<()> {
        self.write_all(vec![SetDataBitsHighByte::byte(), value, direction]).await?;

        Ok(())
    }
}

macro_rules! mpsse_commands {
    ($($cmd: ident { cmd: $cmd_byte:literal$(,)?$($field_name:ident: $field_type:ty),* }),*$(,)?) => {
        #[repr(u8)]
        pub enum CommandByte {
            $($cmd = $cmd_byte,)*
        }

        $(
            #[repr(C, align(8))]
            #[derive(Copy, Clone, Default, Eq, PartialEq)]
            pub struct $cmd;

            impl $cmd {
                pub fn byte() -> u8 {
                    $cmd_byte
                }
            }
        )*
    };
}

mpsse_commands! {
    SetDataBitsLowByte { cmd: 0x80, value: u8, direction: u8 },
    GetDataBitsLowByte { cmd: 0x81 },
    SetDataBitsHighByte { cmd: 0x82, value: u8, direction: u8 },
    GetDataBitsHighByte { cmd: 0x83 },
    EnableLoopback { cmd: 0x84 },
    DisableLoopback { cmd: 0x85 },
    SetClockFrequency { cmd: 0x86 },
    SendImmediate { cmd: 0x87 },
    WaitOnIOHigh { cmd: 0x88 },
    WaitOnIOLow { cmd: 0x89 },
    DisableClockDivide { cmd: 0x8A },
    EnableClockDivide { cmd: 0x8B },
    Enable3PhaseClocking { cmd: 0x8C },
    Disable3PhaseClocking { cmd: 0x8D },
    DelayBits { cmd: 0x8E },
    DelayBytes { cmd: 0x8F, },
    EnableAdaptiveClocking { cmd: 0x96 },
    DisableAdaptiveClocking { cmd: 0x97 },
    EnableDriveOnlyZero { cmd: 0x9E },

    WriteBytesPosLsb { cmd: 0x18 },
    WriteBytesNegLsb { cmd: 0x19 },
    WriteBitsPosLsb { cmd: 0x1A },
    WriteBitsNegLsb { cmd: 0x1B },
    ReadBytesPosLsb { cmd: 0x28, length: u16 },
    ReadBitsPosLsb { cmd: 0x2A, length: u16 },
    ReadBytesNegLsb { cmd: 0x2C, length: u16 },
    ReadBitsNegLsb { cmd: 0x2E, length: u16 },
    WriteBytesNegReadPosLsb { cmd: 0x39 },
    WriteBitsNegReadPosLsb { cmd: 0x3B },
    WriteBytesPosReadNegLsb { cmd: 0x3C },
    WriteBitsPosReadNegLsb { cmd: 0x3E },

    WriteBytesPosMsb { cmd: 0x10 },
    WriteBytesNegMsb { cmd: 0x11 },
    WriteBitsPosMsb { cmd: 0x12 },
    WriteBitsNegMsb { cmd: 0x13 },
    ReadBytesPosMsb { cmd: 0x20, length: u16 },
    ReadBitsPosMsb { cmd: 0x22, length: u16 },
    ReadBitsNegMsb { cmd: 0x26, length: u16 },
    ReadBytesNegMsb { cmd: 0x24, length: u16 },
    WriteBytesNegReadPosMsb { cmd: 0x31 },
    WriteBytesPosReadNegMsb { cmd: 0x34 },
    WriteBitsNegReadPosMsb { cmd: 0x33 },
    WriteBitsPosReadNegMsb { cmd: 0x36 },

    WriteTmsBitsPos { cmd: 0x4A, length: u8, byte: u8  },
    WriteTmsBitsNeg { cmd: 0x4B, length: u8, byte: u8  },
    WriteTmsBitsPosReadPos { cmd: 0x6A, length: u8, byte: u8 },
    WriteTmsBitsPosReadNeg { cmd: 0x6E, length: u8, byte: u8 },
    WriteTmsBitsNegReadPos { cmd: 0x6B, length: u8, byte: u8 },
    WriteTmsBitsNegReadNeg { cmd: 0x6F, length: u8, byte: u8 },

    Synchronize { cmd: 0xAB },
}
