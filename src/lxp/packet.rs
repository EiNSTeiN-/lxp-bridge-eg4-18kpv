use crate::prelude::*;

use enum_dispatch::*;
use nom_derive::{Nom, Parse};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

pub enum ReadInput {
    ReadInput1(ReadInput1),
    ReadInput2(ReadInput2),
    ReadInput3(ReadInput3),
}

// {{{ ReadInput1
#[derive(Debug, Serialize, Nom, InfluxDbWriteable)]
#[nom(LittleEndian)]
pub struct ReadInput1 {
    pub status: u16,
    #[nom(Ignore)]
    pub v_pv: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_pv_1: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_pv_2: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_pv_3: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_bat: f64,

    pub soc: u8,
    pub soh: u8,
    #[nom(SkipBefore(2))]
    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16,
    pub p_pv_2: u16,
    pub p_pv_3: u16,
    pub p_charge: u16,
    pub p_discharge: u16,

    #[nom(Parse = "utils::le_u16_div10")]
    pub v_ac_r: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_ac_s: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_ac_t: f64,
    #[nom(Parse = "utils::le_u16_div100")]
    pub f_ac: f64,

    pub p_inv: u16,
    pub p_rec: u16,

    #[nom(SkipBefore(2))]
    #[nom(Parse = "utils::le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "utils::le_u16_div10")]
    pub v_eps_r: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_eps_s: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_eps_t: f64,
    #[nom(Parse = "utils::le_u16_div100")]
    pub f_eps: f64,
    #[nom(SkipBefore(4))] // peps and seps
    pub p_to_grid: u16,
    pub p_to_user: u16,

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_pv_day_1: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_pv_day_2: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_pv_day_3: f64,

    #[nom(Parse = "utils::le_u16_div10")]
    pub e_inv_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_rec_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_chg_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_dischg_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_eps_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_to_grid_day: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub e_to_user_day: f64,

    #[nom(Parse = "utils::le_u16_div10")]
    pub v_bus_1: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub v_bus_2: f64,

    #[nom(Parse = "utils::current_time")]
    #[serde(serialize_with = "UnixTime::serialize")]
    pub time: UnixTime,
    #[nom(Ignore)]
    #[serde(skip)]
    #[influxdb(tag)]
    pub datalog: Serial, // serde skips this so its not in MQTT messages
} // }}}

// {{{ ReadInput2
#[derive(Debug, Serialize, Nom, InfluxDbWriteable)]
#[nom(Debug, LittleEndian)]
pub struct ReadInput2 {
    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_pv_all_1: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_pv_all_2: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_pv_all_3: f64,

    #[nom(Parse = "utils::le_u32_div10")]
    pub e_inv_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_rec_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_chg_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_dischg_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_eps_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_to_grid_all: f64,
    #[nom(Parse = "utils::le_u32_div10")]
    pub e_to_user_all: f64,

    #[nom(SkipBefore(8))] // 4 byte fault code, 4 byte warning code?
    pub t_inner: u16,
    pub t_rad_1: u16,
    pub t_rad_2: u16,
    pub t_bat: u16,

    #[nom(SkipBefore(2))] // reserved
    pub runtime: u32,
    // bunch of auto_test stuff here I'm not doing yet
    //
    #[nom(Parse = "utils::current_time")]
    #[serde(serialize_with = "UnixTime::serialize")]
    pub time: UnixTime,
    #[nom(Ignore)]
    #[serde(skip)]
    #[influxdb(tag)]
    pub datalog: Serial, // serde skips this so its not in MQTT messages
} // }}}

// {{{ ReadInput3
#[derive(Debug, Serialize, Nom, InfluxDbWriteable)]
#[nom(LittleEndian)]
pub struct ReadInput3 {
    #[nom(SkipBefore(2))] // bat_brand, bat_com_type
    #[nom(Parse = "utils::le_u16_div100")]
    pub max_chg_curr: f64,
    #[nom(Parse = "utils::le_u16_div100")]
    pub max_dischg_curr: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub charge_volt_ref: f64,
    #[nom(Parse = "utils::le_u16_div10")]
    pub dischg_cut_volt: f64,

    pub bat_status_0: u16,
    pub bat_status_1: u16,
    pub bat_status_2: u16,
    pub bat_status_3: u16,
    pub bat_status_4: u16,
    pub bat_status_5: u16,
    pub bat_status_6: u16,
    pub bat_status_7: u16,
    pub bat_status_8: u16,
    pub bat_status_9: u16,
    pub bat_status_inv: u16,

    pub bat_count: u16,
    pub bat_capacity: u16,

    // following are for influx capability only
    #[nom(Parse = "utils::current_time")]
    #[serde(serialize_with = "UnixTime::serialize")]
    pub time: UnixTime,
    #[nom(Ignore)]
    #[serde(skip)]
    #[influxdb(tag)]
    pub datalog: Serial, // serde skips this so its not in MQTT messages
} // }}}

// {{{ TcpFunction
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum TcpFunction {
    Heartbeat = 193,
    TranslatedData = 194,
    ReadParam = 195,
    WriteParam = 196,
} // }}}

// {{{ DeviceFunction
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceFunction {
    ReadHold = 3,
    ReadInput = 4,
    WriteSingle = 6,
    WriteMulti = 16,
    // UpdatePrepare = 33
    // UpdateSendData = 34
    // UpdateReset = 35
} // }}}

#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register {
    Register21 = 21,            // not sure of a better name for this one..
    ChargePowerPercentCmd = 64, // System Charge Rate (%)
    DischgPowerPercentCmd = 65, // System Discharge Rate (%)
    AcChargePowerCmd = 66,      // Grid Charge Power Rate (%)
    AcChargeSocLimit = 67,      // AC Charge SOC Limit (%)
    DischgCutOffSocEod = 105,   // Discharge cut-off SOC (%)
}

#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum RegisterBit {
    // Register 21
    AcChargeEnable = 1 << 7,
    ForcedDischargeEnable = 1 << 10,
}

#[enum_dispatch]
pub trait PacketCommon {
    fn register(&self) -> u16 {
        unimplemented!("register() not implemented");
    }
    fn value(&self) -> u16 {
        unimplemented!("value() not implemented");
    }
}

// this contains method that TcpFrameFactory needs to make a TCP frame
// from a class.
#[enum_dispatch]
pub trait TcpFrameable {
    fn datalog(&self) -> Serial;
    fn protocol(&self) -> u16;
    fn tcp_function(&self) -> TcpFunction;
    fn bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

pub struct TcpFrameFactory;
impl TcpFrameFactory {
    pub fn build<T>(data: T) -> Vec<u8>
    where
        T: TcpFrameable,
    {
        let data_bytes = data.bytes();
        let data_length = data_bytes.len() as u8;
        let frame_length = (18 + data_length) as u16;

        // debug!("data_length={}, frame_length={}", data_length, frame_length);

        let mut r = vec![0; frame_length as usize];

        r[0] = 161;
        r[1] = 26;
        r[2..4].copy_from_slice(&data.protocol().to_le_bytes());
        r[4..6].copy_from_slice(&(frame_length - 6).to_le_bytes());
        r[6] = 1; // unsure what this is, always seems to be 1
        r[7] = data.tcp_function() as u8;

        r[8..18].copy_from_slice(&data.datalog().data());
        // WIP - trying to work out how to learn the inverter sn
        //r[8..18].copy_from_slice(&[0; 10]);

        r[18..].copy_from_slice(&data_bytes);

        r
    }
}

#[enum_dispatch(PacketCommon, TcpFrameable)]
#[derive(Debug, Clone)]
pub enum Packet {
    Heartbeat(Heartbeat),
    TranslatedData(TranslatedData),
    ReadParam(ReadParam),
}

#[derive(PartialEq)]
enum PacketSource {
    Inverter,
    Client,
}

/////////////
//
// HEARTBEATS
//
/////////////

#[derive(Clone, Debug)]
pub struct Heartbeat {
    pub datalog: Serial,
}
impl Heartbeat {
    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 19 {
            return Err(anyhow!("heartbeat packet too short"));
        }

        // assert that the final byte is 0, meaning 0 data bytes follow it
        if input[18] != 0 {
            return Err(anyhow!(
                "heartbeat with non-zero ({}) length byte?",
                input[18]
            ));
        }

        let datalog = Serial::new(&input[8..18])?;

        Ok(Self { datalog })
    }
}

impl TcpFrameable for Heartbeat {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::Heartbeat
    }
}
impl PacketCommon for Heartbeat {}

/////////////
//
// TRANSLATED DATA
//
/////////////

#[derive(Clone, Debug)]
pub struct TranslatedData {
    pub datalog: Serial,
    pub device_function: DeviceFunction, // ReadHold or ReadInput etc..
    pub inverter: Serial,                // inverter serial
    pub register: u16,                   // first register of values
    pub values: Vec<u8>,                 // undecoded, since can be u16 or u32s?
}
impl TranslatedData {
    pub fn value(&self) -> u16 {
        utils::u16ify(&self.values, 0)
    }

    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, utils::u16ify(value, 0)))
            .collect()
    }

    pub fn read_input(&self) -> Result<ReadInput> {
        let r = match self.register {
            0 => ReadInput::ReadInput1(self.read_input1()?),
            40 => ReadInput::ReadInput2(self.read_input2()?),
            80 => ReadInput::ReadInput3(self.read_input3()?),
            _ => return Err(anyhow!("unhandled ReadInput register={}", self.register)),
        };

        Ok(r)
    }

    fn read_input1(&self) -> Result<ReadInput1> {
        match ReadInput1::parse(&self.values) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.v_pv = r.v_pv_1 + r.v_pv_2 + r.v_pv_3;
                r.e_pv_day = r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3;
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn read_input2(&self) -> Result<ReadInput2> {
        match ReadInput2::parse(&self.values) {
            Ok((_, mut r)) => {
                r.e_pv_all = r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3;
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn read_input3(&self) -> Result<ReadInput3> {
        match ReadInput3::parse(&self.values) {
            Ok((_, mut r)) => {
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 38 {
            return Err(anyhow!("packet too short"));
        }

        let protocol = utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[20..len - 2];

        let checksum = &input[len - 2..];
        if Self::checksum(data) != checksum {
            return Err(anyhow!(
                "TranslatedData::decode checksum mismatch - got {:?}, expected {:?}",
                checksum,
                Self::checksum(data)
            ));
        }

        //let address = data[0]; // 0=client, 1=inverter?
        let device_function = DeviceFunction::try_from(data[1])?;
        let inverter = Serial::new(&data[2..12])?;
        let register = utils::u16ify(data, 12);

        let mut value_len = 2;
        let mut value_offset = 14;

        if Self::has_value_length_byte(PacketSource::Inverter, protocol, device_function) {
            value_len = data[value_offset] as usize;
            value_offset += 1;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            return Err(anyhow!(
                "TranslatedData::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            ));
        }

        Ok(Self {
            datalog,
            device_function,
            inverter,
            register,
            values,
        })
    }

    fn has_value_length_byte(
        source: PacketSource,
        protocol: u16,
        device_function: DeviceFunction,
    ) -> bool {
        use DeviceFunction::*;

        let p2 = protocol == 2;
        let psi = source == PacketSource::Inverter;

        p2 && psi && (device_function == ReadHold || device_function == ReadInput)

        /* for future support, we don't actually do WriteMulti yet anyway */
        // let b2 = p2 && !psi && device_function == WriteMulti;
    }

    fn checksum(data: &[u8]) -> [u8; 2] {
        crc16::State::<crc16::MODBUS>::calculate(data).to_le_bytes()
    }
}

impl TcpFrameable for TranslatedData {
    fn protocol(&self) -> u16 {
        1
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::TranslatedData
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = vec![0; 14];

        // data[0] is 0 when writing to inverter, 1 when reading from it?
        data[1] = self.device_function as u8;
        data[2..12].copy_from_slice(&self.inverter.data());
        // WIP - trying to work out how to learn the datalog sn
        //data[2..12].copy_from_slice(&[0xFF; 10]);
        data[12..14].copy_from_slice(&self.register.to_le_bytes());

        if Self::has_value_length_byte(PacketSource::Client, self.protocol(), self.device_function)
        {
            let len = self.values.len() as u8;
            data.extend_from_slice(&[len]);
        }

        let mut m = Vec::new();
        for i in &self.values {
            m.extend_from_slice(&i.to_le_bytes());
        }
        data.append(&mut m);

        data.extend_from_slice(&Self::checksum(&data));

        // the first two bytes now have to be the length, but not including the length bytes.
        // this could probably be a lot neater..
        let data_length = data.len();
        let mut r = Vec::with_capacity(2 + data_length);
        r.extend_from_slice(&(data_length as u16).to_le_bytes());
        r.extend(data);

        r
    }
}

impl PacketCommon for TranslatedData {
    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        utils::u16ify(&self.values, 0)
    }
}

/////////////
//
// READ PARAM
//
/////////////

#[derive(Clone, Debug)]
pub struct ReadParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or u32s?
}
impl ReadParam {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 24 {
            return Err(anyhow!("packet too short"));
        }

        let protocol = utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[18..];
        let register = utils::u16ify(data, 0);

        let mut value_len = 2;
        let mut value_offset = 2;

        if Self::has_value_length_bytes(protocol) {
            value_len = utils::u16ify(data, value_offset) as usize;
            value_offset += 2;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            return Err(anyhow!(
                "ReadParam::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            ));
        }

        Ok(Self {
            datalog,
            register,
            values,
        })
    }

    fn has_value_length_bytes(protocol: u16) -> bool {
        protocol == 2
    }
}

impl TcpFrameable for ReadParam {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::ReadParam
    }

    fn bytes(&self) -> Vec<u8> {
        let mut r = vec![0; 2];

        r[0..2].copy_from_slice(&self.register.to_le_bytes());

        r
    }
}

impl PacketCommon for ReadParam {
    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        utils::u16ify(&self.values, 0)
    }
}

pub struct Parser;
impl Parser {
    pub fn parse(input: &[u8]) -> Result<Packet> {
        let input_len = input.len() as u8;
        if input_len < 18 {
            return Err(anyhow!("packet less than 18 bytes?"));
        }

        if input[0..2] != [161, 26] {
            return Err(anyhow!("invalid packet prefix"));
        }

        if input_len < input[4] - 6 {
            return Err(anyhow!(
                "Parser::parse mismatch: input.len()={},  frame_length={}",
                input_len,
                input[4] - 6
            ));
        }

        match TcpFunction::try_from(input[7])? {
            TcpFunction::Heartbeat => Ok(Packet::Heartbeat(Heartbeat::decode(input)?)),
            TcpFunction::TranslatedData => {
                Ok(Packet::TranslatedData(TranslatedData::decode(input)?))
            }
            TcpFunction::ReadParam => Ok(Packet::ReadParam(ReadParam::decode(input)?)),
            _ => Err(anyhow!("not handled: tcp_function={}", input[7])),
        }
    }
}
