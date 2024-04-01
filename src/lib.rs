#![feature(macro_metavar_expr)]

pub mod mpsse;
use anyhow::Result;

pub enum Error {}

pub struct MpsseInterface {}
pub struct UartInterface {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum FlowControl {
    None,
    RtsCts,
    DtrDsr,
    XonXoff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Bitmode {
    Reset = 0x00,
    Bitbang = 0x01,
    Mpsse = 0x02,
    Syncbb = 0x04,
    Mcu = 0x08,
    Opto = 0x10,
    Cbus = 0x20,
    Syncff = 0x40,
    Ft1284 = 0x80,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlRequest {
    Reset = 0x00,
    SetModemControl = 0x01,
    SetFlowControl = 0x02,
    SetBaudrate = 0x03,
    SetData = 0x04,
    GetStatus = 0x05,
    SetEventChar = 0x06,
    SetErrorChar = 0x07,
    SetLatencyTimer = 0x09,
    GetLatencyTimer = 0x0a,
    SetBitmode = 0x0b,
    ReadPins = 0x0c,
    ReadEeprom = 0x90,
    WriteEeprom = 0x91,
    EraseEeprom = 0x92,
}

#[derive(Clone, Copy, Debug)]
pub enum InterfaceType {
    Mpsse,
    Uart,
}

#[derive(Clone, Debug)]
pub struct InterfaceInfo {
    pub dev: nusb::DeviceInfo,
    pub device_type: DeviceType,
    pub num: u8,
    pub kind: InterfaceType,
}

#[derive(Clone)]
pub struct Interface {
    pub dev: nusb::DeviceInfo,
    pub device_type: DeviceType,
    pub num: u8,
    interface: nusb::Interface,
}

impl core::fmt::Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface")
            .field("dev", &self.dev)
            .field("num", &self.num)
            .finish()
    }
}

use nusb::transfer::*;

impl Interface {
    pub fn with_serial_number(sn: &str, port: u8) -> Result<Self> {
        let mut int = list_interfaces()?
            .find(|i| i.dev.serial_number().map_or(false, |_sn| _sn == sn))
            .ok_or_else(|| anyhow::Error::msg("device not found"))?;

        int.open()
    }

    pub fn set_flow_control(&self, flow_control: FlowControl) -> Result<()> {
        Ok(())
    }

    pub fn set_baudrate(&self, baudrate: u32) -> Result<()> {
        Ok(())
    }

    pub async fn latency_timer(&self) -> Result<core::time::Duration> {
        let pkt = ControlIn {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::GetLatencyTimer as u8,
            value: 0,
            index: self.num as u16,
            length: 1,
        };

        let res = self.interface.control_in(pkt).await.into_result()?;
        let res = core::time::Duration::from_millis(res[0] as u64);

        Ok(res)
    }

    pub async fn set_latency_timer(&self, timer: core::time::Duration) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::SetLatencyTimer as u8,
            value: timer.as_millis() as u16,
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn reset(&self) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::Reset as u8,
            value: 0,
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn purge_rx(&self) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::Reset as u8,
            value: 1,
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn purge_tx(&self) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::Reset as u8,
            value: 2,
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn purge_all(&self) -> Result<()> {
        self.purge_rx().await?;
        self.purge_tx().await?;

        Ok(())
    }

    pub async fn set_bitmode(&self, bitmask: u8, bitmode: Bitmode) -> Result<()> {
        let value: u16 = bitmask as u16 | ((bitmode as u16) << 8);

        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::SetBitmode as u8,
            value,
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn status(&self) -> Result<()> {
        let pkt = ControlIn {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::GetStatus as u8,
            value: 0,
            index: self.num as u16,
            length: 2,
        };

        let res = self.interface.control_in(pkt).await.into_result()?;
        println!("MODEM STATUS {:?}", res);

        Ok(())
    }

    pub async fn set_dtr(&self) -> Result<()> {
        todo!();
        Ok(())
    }

    pub async fn clear_dtr(&self) -> Result<()> {
        todo!();
        Ok(())
    }

    pub async fn set_rts(&self) -> Result<()> {
        todo!();
        Ok(())
    }

    pub async fn clear_rts(&self) -> Result<()> {
        todo!();
        Ok(())
    }

    pub async fn set_event_char(&self, value: char, enable: bool) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::SetEventChar as u8,
            value: u16::from_le_bytes([value as u8, enable as u8]),
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn set_error_char(&self, value: char, enable: bool) -> Result<()> {
        let pkt = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: ControlRequest::SetErrorChar as u8,
            value: u16::from_le_bytes([value as u8, enable as u8]),
            index: self.num as u16,
            data: &[],
        };

        self.interface.control_out(pkt).await.status?;

        Ok(())
    }

    pub async fn read_all(&self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            let rb = RequestBuffer::new((buf.len() + 2).min(512));

            let res = self
                .interface
                .bulk_in(self.in_endpoint(), rb)
                .await
                .into_result()?;

            if res.len() > 2 {
                let status = [res[0], res[1]];
                //println!("STATUS {:02x?}", status);

                buf[..res.len() - 2].clone_from_slice(&res[2..]);
                buf = &mut buf[res.len() - 2..];
            }
        }

        Ok(())
    }

    pub async fn write_all(&self, buf: Vec<u8>) -> Result<()> {
        let res = self
            .interface
            .bulk_out(self.out_endpoint(), buf)
            .await
            .into_result()?;

        Ok(())
    }

    fn in_endpoint(&self) -> u8 {
        (((self.num + 1) * 2) - 1) | 0x80
    }

    fn out_endpoint(&self) -> u8 {
        (self.num + 1) * 2
    }
}

impl InterfaceInfo {
    pub fn open(&mut self) -> Result<Interface> {
        let dev = self.dev.open()?;
        let interface = dev.claim_interface(self.num)?;
        let interface = Interface {
            dev: self.dev.clone(),
            device_type: self.device_type,
            interface,
            num: self.num,
        };

        Ok(interface)
    }
    //
}

#[derive(Clone, Copy, Debug)]
pub enum DeviceType {
    FT4232H,
    FT2232C,
    FT2232H,
    FT232H,
    // FT232H = 0x6014
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub dev: nusb::DeviceInfo,
    pub device_type: DeviceType,
    pub interfaces: Vec<InterfaceInfo>,
}

pub fn list_devices() -> Result<impl Iterator<Item = DeviceInfo>> {
    let devs = nusb::list_devices()?;
    let devs = devs.filter(|dev| dev.vendor_id() == 0x0403);

    let devs = devs.map(|dev| {
        let version = dev.device_version();

        match version {
            0x800 => DeviceInfo {
                dev: dev.clone(),
                device_type: DeviceType::FT4232H,
                interfaces: dev
                    .interfaces()
                    .enumerate()
                    .map(|(i, info)| match i {
                        0..=1 => InterfaceInfo {
                            num: i as u8,
                            dev: dev.clone(),
                            device_type: DeviceType::FT4232H,
                            kind: InterfaceType::Mpsse,
                        },
                        2..=3 => InterfaceInfo {
                            num: i as u8,
                            dev: dev.clone(),
                            device_type: DeviceType::FT4232H,
                            kind: InterfaceType::Uart,
                        },
                        _ => panic!("unknown interface"),
                    })
                    .collect(),
            },
            0x900 => DeviceInfo {
                dev: dev.clone(),
                device_type: DeviceType::FT232H,
                interfaces: vec![InterfaceInfo {
                    num: 0,
                    dev: dev.clone(),
                    device_type: DeviceType::FT232H,
                    kind: InterfaceType::Mpsse,
                }],
            },
            n => panic!("unknown device version {}", n),
        }
    });

    Ok(devs)
}

pub fn list_interfaces() -> Result<impl Iterator<Item = InterfaceInfo>> {
    let devs = list_devices()?;
    let devs = devs.flat_map(|dev| dev.interfaces);
    Ok(devs)
}
