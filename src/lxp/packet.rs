use crate::prelude::*;

use enum_dispatch::*;
use nom_derive::{Nom, Parse};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

pub enum ReadInput {
    ReadInputAll(Box<ReadInputAll>),
    ReadInputAll2(Box<ReadInputAll2>),
    ReadInput1(ReadInput1),
    ReadInput2(ReadInput2),
    ReadInput3(ReadInput3),
}

// {{{ ReadInputAll
#[derive(PartialEq, Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInputAll {
    pub status: u16,  // operating mode
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_1: f64, // PV1 voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_2: f64, // PV2 voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_3: f64, // PV3 voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bat: f64, // battery voltage

    pub soc: i8,  // battery capacity (state of charge)
    pub soh: i8, // state of health

    pub internal_fault: u16,

    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16, // PV1 power
    pub p_pv_2: u16, // PV2 power
    pub p_pv_3: u16, // PV3 power
    #[nom(Ignore)]
    pub p_battery: i32,
    pub p_charge: u16, // charging power (incoming battery power)
    pub p_discharge: u16, // discharge power (outflow battery power)

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_r: f64, // R-phase mains voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_s: f64, // S-phase mains voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_t: f64, // T-phase mains voltage
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_ac: f64, // Mains frequency

    pub p_inv: u16, // Inverter output power (grid port)
    pub p_rec: u16, // AC charging rectified power

    #[nom(SkipBefore(2))] // IinvRMS
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_r: f64, // R-phase off-grid output voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_s: f64, // S-phase off-grid output voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_t: f64, // T-phase off-grid output voltage
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_eps: f64, // Off-grid output frequency
    pub p_eps: u16, // Off-grid inverter power
    pub s_eps: u16, // Off-grid apparent power
    #[nom(Ignore)]
    pub p_grid: i32,
    pub p_to_grid: u16, // export power to grid
    pub p_to_user: u16, // import power from grid

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_1: f64, // PV1 power generation today
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_2: f64, // PV2 power generation today
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_3: f64, // PV3 power generation today

    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_inv_day: f64, // Today's grid-connected inverter output energy
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_rec_day: f64, // Today's AC charging rectified energy
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_chg_day: f64, // Charge energy today
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_dischg_day: f64, // Discharge energy today
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_eps_day: f64, // Off-grid output energy today
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_grid_day: f64, // Today's export energy to grid
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_user_day: f64, // Today's import energy from grid

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_1: f64, // Bus 1 voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_2: f64, // Bus 2 voltage

    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_1: f64, // PV1 cumulative power generation
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_2: f64, // PV2 cumulative power generation
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_3: f64, // PV3 cumulative power generation

    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_inv_all: f64, // Inverter accumulative output energy
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_rec_all: f64, // AC charging accumulative rectified energy
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_chg_all: f64, // Cumulative charge energy level
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_dischg_all: f64, // Cumulative discharge energy level
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_eps_all: f64, // Cumulative off-grid inverter power
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_grid_all: f64, // Cumulative export energy to grid
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_user_all: f64, // Cumulative import energy from grid

    pub fault_code: u32,
    pub warning_code: u32,

    pub t_inner: i16, // Internal ring temperature
    pub t_rad_1: i16, // Radiator 1 temperature
    pub t_rad_2: i16, // Radiator 2 temperature
    pub t_bat: i16, // Battery temperature
    #[nom(SkipBefore(2))] // reserved - radiator 3?
    pub runtime: u32,
    pub register_71: u16, // auto test result bits
    #[nom(SkipBefore(10))] // 72-76 auto_test stuff, TODO..
    pub register_77: u16, // AC couple status
    #[nom(SkipBefore(4))] // 78-79 unspecified
    #[nom(SkipBefore(2))] // bat_brand, bat_com_type
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_chg_curr: f64, // BMS limited maximum charging current
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_dischg_curr: f64, // BMS limited maximum discharge current
    #[nom(Parse = "Utils::le_u16_div10")]
    pub charge_volt_ref: f64, // BMS recommended charging voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub dischg_cut_volt: f64, // BMS recommended discharge cut-off voltage

    pub bat_status_0: u16, // BMS status information
    pub bat_status_1: u16, // BMS status information
    pub bat_status_2: u16, // BMS status information
    pub bat_status_3: u16, // BMS status information
    pub bat_status_4: u16, // BMS status information
    pub bat_status_5: u16, // BMS status information
    pub bat_status_6: u16, // BMS status information
    pub bat_status_7: u16, // BMS status information
    pub bat_status_8: u16, // BMS status information
    pub bat_status_9: u16, // BMS status information
    pub bat_status_inv: u16, // Inverter summarizes lithium battery status information

    pub bat_count: u16, // Number of batteries in parallel
    pub bat_capacity: u16, // Battery capacity (Ah)

    #[nom(Parse = "Utils::le_u16_div100")]
    pub bat_current: f64, // Battery current

    pub bms_event_1: u16, // FaultCode_BMS
    pub bms_event_2: u16, // WarningCode_BMS

    // TODO: probably floats but need non-zero sample data to check. just guessing at the div100.
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub max_cell_voltage: f64, // Maximum cell voltage
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub min_cell_voltage: f64, // Minimum cell voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_cell_temp: f64, // Maximum monomer temperature
    #[nom(Parse = "Utils::le_u16_div10")]
    pub min_cell_temp: f64, // Minimum monomer temperature

    pub bms_fw_update_state: u16, // 1 for upgrating, 2 for successful, 3 for failed

    pub cycle_count: u16, // Number of charge and discharge cycles

    #[nom(Parse = "Utils::le_u16_div10")]
    pub vbat_inv: f64, // Inverter battery voltage sampling

    // temp sensors
    #[nom(Parse = "Utils::le_u16_div10")]
    pub t1_temp: f64, // 12K BT temperature
    #[nom(SkipBefore(8))] // 109-112 reserved T2-T5 sensors

    pub register_113: u16, // phase config bits
    pub p_on_grid_load: u16, // Load power of the inverter when it is not off-grid

    #[nom(SkipBefore(10))] // 115-119 serial number

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_half_bus: f64, // Half bus voltage
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_gen: f64, // generator voltage
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_gen: f64, // generator frequency
    pub p_gen: u16, // generator power
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_gen_day: f64, // daily generator energy
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_gen_all: f64, // cumulative generator energy

    // following are for influx capability only
    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInputAll2
    #[derive(PartialEq, Clone, Debug, Serialize, Nom)]
    #[nom(LittleEndian)]
    pub struct ReadInputAll2 {
        #[nom(Parse = "Utils::le_u16_div10")]
        pub v_eps_l1: f64, // Voltage of EPS L1N
        #[nom(Parse = "Utils::le_u16_div10")]
        pub v_eps_l2: f64, // Voltage of EPS L2N
    
        pub p_eps_l1: u16, // Active power of EPS L1N
        pub p_eps_l2: u16, // Active power of EPS L2N
        pub s_eps_l1: u16, // Apparent power of EPS L1N
        pub s_eps_l2: u16, // Apparent power of EPS L2N
        #[nom(Parse = "Utils::le_u16_div10")]
        pub e_eps_l1_day: f64, // Daily energy of EPS L1N
        #[nom(Parse = "Utils::le_u16_div10")]
        pub e_eps_l2_day: f64, // Daily energy of EPS L2N
        #[nom(Parse = "Utils::le_u32_div10")]
        pub e_eps_l1_all: f64, // Total EPS L1N energy
        #[nom(Parse = "Utils::le_u32_div10")]
        pub e_eps_l2_all: f64, // Total EPS L2N energy

        #[nom(SkipBefore(2))] // 139 Qinv

        pub afci_ch1_current: u16, // AFCI current (mA)
        pub afci_ch2_current: u16, // AFCI current (mA)
        pub afci_ch3_current: u16, // AFCI current (mA)
        pub afci_ch4_current: u16, // AFCI current (mA)

        pub register_144: u16, // AFCI flag

        pub afci_arc_ch1: u16, // Real time arc of channel 1
        pub afci_arc_ch2: u16, // Real time arc of channel 2
        pub afci_arc_ch3: u16, // Real time arc of channel 3
        pub afci_arc_ch4: u16, // Real time arc of channel 4

        pub afci_max_arc_ch1: u16, // Max arc of channel 1
        pub afci_max_arc_ch2: u16, // Max arc of channel 2
        pub afci_max_arc_ch3: u16, // Max arc of channel 3
        pub afci_max_arc_ch4: u16, // Max arc of channel 4

        pub p_ac_couple: u16, // AC Coupled inverter power

        #[nom(SkipBefore(16))] // 154 to 161 Auto Test Trip Value 0-7
        #[nom(SkipBefore(16))] // 162 to 169 Auto Test Trip Time 0-7

        pub p_load: u16, // Load power for on-grid mode
        #[nom(Parse = "Utils::le_u16_div10")]
        pub e_load_day: f64, // Daily energy of loads
        #[nom(Parse = "Utils::le_u32_div10")]
        pub e_load_all: f64, // Cumulative energy of loads

        #[nom(SkipBefore(2))] // 174 Safety Switch State
    
        pub eps_overload_ctrl_time: u16, // Connect in xx S after triggering the EPS overload issue

        #[nom(SkipBefore(8))] // 176-179

        pub p_inv_s: u16, // On grid inverter power of three phase: S phase
        pub p_inv_t: u16, // On grid inverter power of three phase: T phase
        pub p_rec_s: u16, // Charging rectification power of three phase: S phase
        pub p_rec_t: u16, // Charging rectification power of three phase: T phase
        pub p_to_grid_s: u16, // User on-grid power of three phase: S phase
        pub p_to_grid_t: u16, // User on-grid power of three phase: T phase
        pub p_to_user_s: u16, // Grid supply power of three phase: S phase
        pub p_to_user_t: u16, // Grid supply power of three phase: T phase
        pub p_gen_s: u16, // Power of generator for three phase: S phase
        pub p_gen_t: u16, // Power of generator for three phase: T phase
        #[nom(Parse = "Utils::le_u16_div100")]
        pub inv_rms_curr_s: f64, // Effective value of three phase inverter current: S phase
        #[nom(Parse = "Utils::le_u16_div100")]
        pub inv_rms_curr_t: f64, // Effective value of three phase inverter current: T phase

        #[nom(Parse = "Utils::le_i16_div1000")]
        pub pf_s: f64, // Power factor of phase S in three-phase inverter (signed)
        #[nom(Parse = "Utils::le_i16_div10")]
        pub v_grid_l1: f64, // Voltage of Grid L1N (signed)
        #[nom(Parse = "Utils::le_i16_div10")]
        pub v_grid_l2: f64, // Voltage of Grid L2N (signed)
        #[nom(Parse = "Utils::le_i16_div10")]
        pub v_gen_l1: f64, // Voltage of Gen L1N (signed)
        #[nom(Parse = "Utils::le_i16_div10")]
        pub v_gen_l2: f64, // Voltage of Gen L2N (signed)
        pub p_inv_l1: i16, // Inverting power of phase L1N (signed)
        pub p_inv_l2: i16, // Inverting power of phase L2N (signed)
        pub p_rec_l1: i16, // Rectifying power of phase L1N (signed)
        pub p_rec_l2: i16, // Rectifying power of phase L2N (signed)
        pub p_to_grid_l1: u16, // Grid export power of phase L1N
        pub p_to_grid_l2: u16, // Grid export power of phase L2N
        pub p_to_user_l1: u16, // Grid import power of phase L1N
        pub p_to_user_l2: u16, // Grid import power of phase L2N
        #[nom(Parse = "Utils::le_i16_div1000")]
        pub pf_t: f64, // Power factor of phase T in three-phase inverter

        #[nom(Parse = "Utils::current_time_for_nom")]
        pub time: UnixTime,
        #[nom(Ignore)]
        pub datalog: Serial,
    } // }}}
    

// {{{ ReadInput1
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput1 {
    pub status: u16,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_3: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bat: f64,

    pub soc: i8,
    pub soh: i8,

    pub internal_fault: u16,

    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16,
    pub p_pv_2: u16,
    pub p_pv_3: u16,
    #[nom(Ignore)]
    pub p_battery: i32,
    pub p_charge: u16,
    pub p_discharge: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_ac: f64,

    pub p_inv: u16,
    pub p_rec: u16,

    #[nom(SkipBefore(2))] // IinvRMS
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_eps: f64,
    pub p_eps: u16,
    pub s_eps: u16,
    #[nom(Ignore)]
    pub p_grid: i32,
    pub p_to_grid: u16,
    pub p_to_user: u16,

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_3: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_inv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_rec_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_chg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_dischg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_eps_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_grid_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_user_day: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_2: f64,

    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInput2
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(Debug, LittleEndian)]
pub struct ReadInput2 {
    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_1: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_2: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_3: f64,

    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_inv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_rec_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_chg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_dischg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_eps_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_grid_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_user_all: f64,

    pub fault_code: u32,
    pub warning_code: u32,

    pub t_inner: i16,
    pub t_rad_1: i16,
    pub t_rad_2: i16,
    pub t_bat: i16,

    #[nom(SkipBefore(2))] // reserved
    pub runtime: u32,
    pub register_71: u16, // auto test result bits
    #[nom(SkipBefore(8))] // 72-75 auto_test stuff, TODO..
    pub register_77: u16, // AC couple status

    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInput3
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput3 {
    #[nom(SkipBefore(2))] // bat_brand, bat_com_type
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_chg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_dischg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub charge_volt_ref: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
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

    #[nom(Parse = "Utils::le_u16_div100")]
    pub bat_current: f64,

    pub bms_event_1: u16,
    pub bms_event_2: u16,

    // TODO: probably floats but need non-zero sample data to check. just guessing at the div100.
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub max_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub min_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub max_cell_temp: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub min_cell_temp: f64,

    pub bms_fw_update_state: u16,

    pub cycle_count: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub vbat_inv: f64,

    // temp sensors
    #[nom(Parse = "Utils::le_u16_div10")]
    pub t1_temp: f64, // 12K BT temperature
    #[nom(SkipBefore(8))] // 109-112 reserved T2-T5 sensors

    pub register_113: u16, // phase config bits
    pub p_on_grid_load: u16, // Load power of the inverter when it is not off-grid

    #[nom(SkipBefore(10))] // 115-119 serial number

    // following are for influx capability only
    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInputs
#[derive(Default, Clone, Debug)]
pub struct ReadInputs {
    read_input_1: Option<ReadInput1>,
    read_input_2: Option<ReadInput2>,
    read_input_3: Option<ReadInput3>,
}

impl ReadInputs {
    pub fn set_read_input_1(&mut self, i: ReadInput1) {
        self.read_input_1 = Some(i);
    }
    pub fn set_read_input_2(&mut self, i: ReadInput2) {
        self.read_input_2 = Some(i);
    }
    pub fn set_read_input_3(&mut self, i: ReadInput3) {
        self.read_input_3 = Some(i);
    }

    pub fn to_input_all(&self) -> Option<ReadInputAll> {
        match (
            self.read_input_1.as_ref(),
            self.read_input_2.as_ref(),
            self.read_input_3.as_ref(),
        ) {
            (Some(ri1), Some(ri2), Some(ri3)) => Some(ReadInputAll {
                status: ri1.status,
                v_pv_1: ri1.v_pv_1,
                v_pv_2: ri1.v_pv_2,
                v_pv_3: ri1.v_pv_3,
                v_bat: ri1.v_bat,
                soc: ri1.soc,
                soh: ri1.soh,
                internal_fault: ri1.internal_fault,
                p_pv: ri1.p_pv,
                p_pv_1: ri1.p_pv_1,
                p_pv_2: ri1.p_pv_2,
                p_pv_3: ri1.p_pv_3,
                p_battery: ri1.p_battery,
                p_charge: ri1.p_charge,
                p_discharge: ri1.p_discharge,
                v_ac_r: ri1.v_ac_r,
                v_ac_s: ri1.v_ac_s,
                v_ac_t: ri1.v_ac_t,
                f_ac: ri1.f_ac,
                p_inv: ri1.p_inv,
                p_rec: ri1.p_rec,
                pf: ri1.pf,
                v_eps_r: ri1.v_eps_r,
                v_eps_s: ri1.v_eps_s,
                v_eps_t: ri1.v_eps_t,
                f_eps: ri1.f_eps,
                p_eps: ri1.p_eps,
                s_eps: ri1.s_eps,
                p_grid: ri1.p_grid,
                p_to_grid: ri1.p_to_grid,
                p_to_user: ri1.p_to_user,
                e_pv_day: ri1.e_pv_day,
                e_pv_day_1: ri1.e_pv_day_1,
                e_pv_day_2: ri1.e_pv_day_2,
                e_pv_day_3: ri1.e_pv_day_3,
                e_inv_day: ri1.e_inv_day,
                e_rec_day: ri1.e_rec_day,
                e_chg_day: ri1.e_chg_day,
                e_dischg_day: ri1.e_dischg_day,
                e_eps_day: ri1.e_eps_day,
                e_to_grid_day: ri1.e_to_grid_day,
                e_to_user_day: ri1.e_to_user_day,
                v_bus_1: ri1.v_bus_1,
                v_bus_2: ri1.v_bus_2,
                e_pv_all: ri2.e_pv_all,
                e_pv_all_1: ri2.e_pv_all_1,
                e_pv_all_2: ri2.e_pv_all_2,
                e_pv_all_3: ri2.e_pv_all_3,
                e_inv_all: ri2.e_inv_all,
                e_rec_all: ri2.e_rec_all,
                e_chg_all: ri2.e_chg_all,
                e_dischg_all: ri2.e_dischg_all,
                e_eps_all: ri2.e_eps_all,
                e_to_grid_all: ri2.e_to_grid_all,
                e_to_user_all: ri2.e_to_user_all,
                fault_code: ri2.fault_code,
                warning_code: ri2.warning_code,
                t_inner: ri2.t_inner,
                t_rad_1: ri2.t_rad_1,
                t_rad_2: ri2.t_rad_2,
                t_bat: ri2.t_bat,
                runtime: ri2.runtime,
                register_71: ri2.register_71,
                register_77: ri2.register_77,
                max_chg_curr: ri3.max_chg_curr,
                max_dischg_curr: ri3.max_dischg_curr,
                charge_volt_ref: ri3.charge_volt_ref,
                dischg_cut_volt: ri3.dischg_cut_volt,
                bat_status_0: ri3.bat_status_0,
                bat_status_1: ri3.bat_status_1,
                bat_status_2: ri3.bat_status_2,
                bat_status_3: ri3.bat_status_3,
                bat_status_4: ri3.bat_status_4,
                bat_status_5: ri3.bat_status_5,
                bat_status_6: ri3.bat_status_6,
                bat_status_7: ri3.bat_status_7,
                bat_status_8: ri3.bat_status_8,
                bat_status_9: ri3.bat_status_9,
                bat_status_inv: ri3.bat_status_inv,
                bat_count: ri3.bat_count,
                bat_capacity: ri3.bat_capacity,
                bat_current: ri3.bat_current,
                bms_event_1: ri3.bms_event_1,
                bms_event_2: ri3.bms_event_2,
                max_cell_voltage: ri3.max_cell_voltage,
                min_cell_voltage: ri3.min_cell_voltage,
                max_cell_temp: ri3.max_cell_temp,
                min_cell_temp: ri3.min_cell_temp,
                bms_fw_update_state: ri3.bms_fw_update_state,
                cycle_count: ri3.cycle_count,
                vbat_inv: ri3.vbat_inv,
                t1_temp: ri3.t1_temp,
                register_113: ri3.register_113,
                p_on_grid_load: ri3.p_on_grid_load,
                v_half_bus: 0.0,
                v_gen: 0.0,
                f_gen: 0.0,
                p_gen: 0,
                e_gen_day: 0.0,
                e_gen_all: 0.0,
                datalog: ri1.datalog,
                time: ri1.time.clone(),
            }),
            _ => None,
        }
    }
} // }}}

// {{{ TcpFunction
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum TcpFunction {
    Heartbeat = 193,
    TranslatedData = 194,
    ReadParam = 195,
    WriteParam = 196,
} // }}}

// {{{ DeviceFunction
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceFunction {
    ReadHold = 3,
    ReadInput = 4,
    WriteSingle = 6,
    WriteMulti = 16,
    // UpdatePrepare = 33
    // UpdateSendData = 34
    // UpdateReset = 35
    // ReadHoldError = 131
    // ReadInputError = 132
    // WriteSingleError = 134
    // WriteMultiError = 144
} // }}}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register {
    FwCodeLo = 7,                   // software version definition 
    FwCodeHi = 8,                   //
    Version1 = 9,                   //
    Version2 = 10,                  //
    ResetSetting = 11,              // reboot command
    InverterTime1 = 12,             // year, month
    InverterTime2 = 13,             // date, hour
    InverterTime3 = 14,             // minute, second
    ComAddress = 15,                // Modbus address
    Language = 16,                  // 0-English 1-German
    DeviceType = 19,                // 0-Default 3-XOLTA (for high speed comm)
    PvInputModel = 20,              //
    Register21 = 21,                // "FuncEn" function enable register
    StartPvVolt = 22,               // PV start-up voltage (0.1V)
    ConnectTime = 23,               // Waiting time of On-grid (s)
    ReconnectTime = 24,             // Waiting time of Reconnect On-grid (s)
    GridVoltConnLow = 25,           // The lower limit of the allowed on-grid voltage (0.1V)
    GridVoltConnHigh = 26,          // The upper limit of the allowed on-grid voltage (0.1V)
    GridFreqConnLow = 27,           // The lower limit of the allowable on-grid frequency (0.01Hz)
    GridFreqConnHigh = 28,          // The upper limit of the allowable on-grid frequency (0.01Hz)
    GridVoltLimit1Low = 29,         // Grid voltage level 1 undervoltage protection point (0.1V)
    GridVoltLimit1High = 30,        // Grid voltage level 1 overvoltage protection point (0.1V)
    GridVoltLimit1LowTime = 31,     // Grid voltage level 1 undervoltage protection time
    GridVoltLimit1HighTime = 32,    // Grid voltage level 1 overvoltage protection time
    GridVoltLimit2Low = 33,         // Grid voltage level 2 undervoltage protection point (0.1V)
    GridVoltLimit2High = 34,        // Grid voltage level 2 overvoltage protection point (0.1V)
    GridVoltLimit2LowTime = 35,     // Grid voltage level 2 undervoltage protection time
    GridVoltLimit2HighTime = 36,    // Grid voltage level 2 overvoltage protection time
    GridVoltLimit3Low = 37,         // Grid voltage level 3 undervoltage protection point (0.1V)
    GridVoltLimit3High = 38,        // Grid voltage level 3 overvoltage protection point (0.1V)
    GridVoltLimit3LowTime = 39,     // Grid voltage level 3 undervoltage protection time
    GridVoltLimit3HighTime = 40,    // Grid voltage level 3 overvoltage protection time
    GridVoltMovAvgHigh = 41,        // Grid voltage sliding average overvoltage protection point (0.1V)
    GridFreqLimit1Low = 42,         // Grid frequency level 1 underfrequency protection point (0.01Hz)
    GridFreqLimit1High = 43,        // Grid frequency level 1 overfrequency protection point (0.01Hz)
    GridFreqLimit1LowTime = 44,     // Grid frequency level 1 underfrequency protection time
    GridFreqLimit1HighTime = 45,    // Grid frequency level 1 overfrequency protection time
    GridFreqLimit2Low = 46,         // Grid frequency level 2 underfrequency protection point (0.01Hz)
    GridFreqLimit2High = 47,        // Grid frequency level 2 overfrequency protection point (0.01Hz)
    GridFreqLimit2LowTime = 48,     // Grid frequency level 2 underfrequency protection time
    GridFreqLimit2HighTime = 49,    // Grid frequency level 2 overfrequency protection time
    GridFreqLimit3Low = 50,         // Grid frequency level 3 underfrequency protection point (0.01Hz)
    GridFreqLimit3High = 51,        // Grid frequency level 3 overfrequency protection point (0.01Hz)
    GridFreqLimit3LowTime = 52,     // Grid frequency level 3 underfrequency protection time
    GridFreqLimit3HighTime = 53,    // Grid frequency level 3 overfrequency protection time
    MaxQPercentForQV = 54,          // The maximum percentage of reactive power for the Q(V) curve (%)
    V1L = 55,                       // Q(V) curve undervoltage 1 (0.1V)
    V2L = 56,                       // Q(V) curve undervoltage 2 (0.1V)
    V1H = 57,                       // Q(V) curve overvoltage 1 (0.1V)
    V2H = 58,                       // Q(V) curve overvoltage 2 (0.1V)
    ReactivePowerCmdType = 59,      // Reactive power command type
    ActivePowerPercentCmd = 60,     // Active power percentage set value (%)
    ReactivePowerPercentCmd = 61,   // Reactive power percentage set value (%)
    PfCmd = 62,                     // PF set value, 750-1000(under), 1750-2000(over) (0.001)
    PowerSoftStartSlope = 63,       // Loading rate, the percentage of power increase per minute. (%o/min)
    ChargePowerPercentCmd = 64,     // Charging power percentage setting (%)
    DischgPowerPercentCmd = 65,     // Discharging power percentage setting (%)
    AcChargePowerCmd = 66,          // Grid Charge Power Rate (%)
    AcChargeSocLimit = 67,          // AC Charge SOC Limit (%)
    ChargePriorityPowerCmd = 74,    // Charge Priority Charge Rate (%)
    ChargePrioritySocLimit = 75,    // Charge Priority SOC Limit (%)
    ForcedDischgPowerCmd = 82,      // Forced discharge percentage setting (%)
    ForcedDischgSocLimit = 83,      // Forced Discarge SOC Limit (%)
    EpsVoltageSet = 90,             // Off-grid output voltage level setting (1V)
    EpsFrequencySet = 91,           // Off-grid output frequency system setting (1Hz)
    LockInGridVForPFCurve = 92,     // cosphi(P) lock in voltage (0.1V)
    LockOutGridVForPFCurve = 93,    // cosphi(P) lock out voltage (0.1V)
    LockInPowerForQVCurve = 94,     // Q(V) lock in power (%)
    LockOutPowerForQVCurve = 95,    // Q(V) lock out power (%)
    DelayTimeForQVCurve = 96,       // Q(V) delay
    DelayTimeForOverFCurve = 97,    // Overfrequency load reduction delay
    ChargeVoltRef = 99,             // Lead-acid battery charging specified voltage (0.1V)
    CutVoltForDischg = 100,         // Lead-acid battery discharge cut-off voltage (0.1V)
    ChargeCurr = 101,               // Charging current (A)
    DischgCurr = 102,               // Discharging current (A)
    MaxBackFlow = 103,              // Feed-in grid power setting (%)
    DischgCutOffSocEod = 105,       // Discharge cut-off SOC (%)
    TemprLowerLimitDischg = 106,    // Lead-acid Temperature low limit for discharging (0.1℃)
    TemprUpperLimitDischg = 107,    // Lead-acid Temperature high limit for discharging (0.1℃)
    TemprLowerLimitChg = 108,       // Lead-acid Temperature low limit for charging (0.1℃)
    TemprUpperLimitChg = 109,       // Lead-acid Temperature high limit for charging (0.1℃)
    FunctionEnable1 = 110,          // Function Enable 1 bits
    SetSystemType = 112,            // Set the single/parallel type
    SetComposedPhase = 113,         // Set composed phases bits
    ClearFunction = 114,            // Clear alarm function
    OVFDerateStartPoint = 115,      // Over-frequency load reduction start frequency point (0.01Hz)
    PtoUserStartDischg = 116,       // Device starts discharging when Ptouser higher than this value (1W)
    PtoUserStartCharge = 117,       // Device starts charging when Ptouser less than this value (1W)
    VbatStartDerating = 118,        // For lead-acid battery, according to given curve decrease discharging power when voltage lower than this value (0.1V)
    WCTPowerOffset = 119,           // CT Power compensation, import is positive (1W)
    StSysEnable = 120,              // Sys Enable bits
    OVFDerateEndPoint = 124,        // Overfrequency load reduction ends at the frequency point (0.01Hz)
    EpsDischgCutoffSocEod = 125,    // EPS Discharge cut-off SOC (%)
    OptimalChgDischg1 = 126,        // Optimal Charge Discharge bits
    OptimalChgDischg2 = 127,        // Optimal Charge Discharge bits
    OptimalChgDischg3 = 128,        // Optimal Charge Discharge bits
    OptimalChgDischg4 = 129,        // Optimal Charge Discharge bits
    OptimalChgDischg5 = 130,        // Optimal Charge Discharge bits
    OptimalChgDischg6 = 131,        // Optimal Charge Discharge bits
    BatCellVoltageLimit = 132,      // Battery cell voltage lower and upper limit. (0.1V)
    BatCellConfig = 133,            // Number of battery cells in parallel and series
    UVFDerateStartPoint = 134,      // Underfrequency load reduction starting point (0.01Hz)
    UVFDerateEndPoint = 135,        // The end point of underfrequency load reduction (0.01Hz)
    OVFDerateRatio = 136,           // Underfrequency load ramp rate (%Pm/Hz)
    SpecLoadCompensate = 137,       // The maximum amount of compensation for a specific load (1W)
    ChargePowerPercentCmd2 = 138,    // Charging power percentage setting (0.1%)
    DischgPowerPercentCmd2 = 139,    // Discharging power percentage setting (0.1%)
    AcChargePowerCmd2 = 140,         // AC charge percentage setting (0.1%)
    ChargePriorityPowerCmd2 = 141,   // Charging priority percentage setting (0.1%)
    ForcedDischgPowerCmd2 = 142,     // Forced discharge percentage setting (0.1%)
    ActivePowerPercentCmd2 = 143,    // Inverse active percentage setting (0.1%)
    FloatChargeVolt = 144,          // Float charge voltage (0.1V)
    OutputPrioConfig = 145,         // 
    LineMode = 146,                 // 
    BatteryCapacity = 147,          // Battery capacity, for unmatched batteries (Ah)
    BatteryNominalVolt = 148,       // Battery rating voltage, for unmatched batteries (0.1V)
    EqualizationVolt = 149,         // Battery equalization voltage
    EqualizationInterval = 150,     // Balancing interval (days)
    EqualizationTime = 151,         // Balancing duration (hours)
    AcChargeStartVolt = 158,        // Battery voltage of AC charging start, which will be valid after selecting ACChg according to voltage. (0.1V)
    AcChargeEndVolt = 159,          // Battery voltage of AC charging cut-off, effective after selecting ACChg according to voltage. (0.1V)
    AcChargeStartSocLimit = 160,    // SOC at which AC charging will begin (%)
    AcChargeEndSocLimit = 161,      // SOC at which AC charging will end (%)
    BatLowVoltage = 162,            // Battery under-voltage alarm point, which will be valid after selecting DisChgCtrl according to voltage or both voltage and time (0.1V)
    BatLowBackVoltage = 163,        // Battery under-voltage alarm recovery point, which will be valid after selecting DisChgCtrl according to voltage or both voltage and time (0.1V)
    BatLowSoc = 164,                // Battery under-voltage alarm point, which will be valid after selecting DisChgCtrl according to SOC or both SOC and time (%)
    BatLowBackSoc = 165,            // Battery under-voltage alarm recovery point, which will be valid after selecting DisChgCtrl according to SOC or both SOC and time (%)
    BatLowToUtilityVoltage = 166,   // Voltage point for battery undervoltage to grid transfer, which will be valid after selecting DisChgCtrl according to voltage or both. (0.1V)
    BatLowtoUtilitySoc = 167,       // SOC for battery under-voltage to grid transfer, which will be valid after selecting DisChgCtrl according to SOC or both. (%)
    AcChargeBatCurrent = 168,       // Charge Current from AC (A)
    OnGridEndDischrgVoltage = 169,  // On-grid end of dischage voltage (0.1V)
    SocCurveBatVolt1 = 171,         // Voltage point 1 for SOC calibration (0.1V)
    SocCurveBatVolt2 = 172,         // Voltage point 2 for SOC calibration (0.1V)
    SocCurveSoc1 = 173,             // SOC reading based on Voltage point 1 (%)
    SocCurveSoc2 = 174,             // SOC reading based on Voltage point 2 (%)
    SocCurveInnerResistance = 175,  // Inner resistance of the battery (mΩ)
    MaxGridInputPower = 176,        // Max. Grid import power limitation (W)
    GenRatePower = 177,             // The rated power of generator input (0.1kW)
    FunctionEnable2 = 179,          // Function Enable 2 bits
    AFCIArcThreshold = 180,         // 
    VoltWattV1 = 181,               // 1.05Vn-1.09Vn, default=1.06Vn (0.1V)
    VoltWattV2 = 182,               // (V1+0.01Vn)-1.10Vn, default=1.1Vn (0.1V)
    VoltWattDelayTime = 183,        // Default 10000ms
    VoltWattP2 = 184,               // (%)
    VrefQV = 185,                   // (0.1V)
    VrefFilterTime = 186,           // (s)
    Q3Qv = 187,                     // (%)
    Q4Qv = 188,                     // (%)
    P1Qp = 189,                     // (%)
    P2Qp = 190,                     // (%)
    P3Qp = 191,                     // (%)
    P4Qp = 192,                     // (%)
    UVFIncreaseRatio = 193,         // Underfrequency load ramp rate (%Pm/Hz)
    GenChargeStartVolt = 194,       // Intitial voltage for generator charging the battery, which will be valid after selecting GenChg according to voltage. (0.1V)
    GenChargeEndVolt = 195,         // Battery voltage at the end of generator charging, which will be valid after selecting GenChg according to voltage. (0.1V)
    GenChargeStartSoc = 196,        // SOC limit for generator charging the battery, which will be valid after selecting charge according to SOC (%)
    GenChargeEndSoc = 197,          // SOC limit to end the generator charging, which will be valid after selecting charge according to SOC (%)
    MaxGenChargeBatCurr = 198,      // Max. Charge current from generator (A)
    OverTempDeratePoint = 199,      // Overtemperature load reduction point (0.1℃)
    ChargePriorityEndVolt = 201,    // Charging priority voltage limit (0.1V)
    ForceDichgEndVolt = 202,        // Forced discharge voltage limit (0.1V)
    GridRegulation = 203,           // Grid regulation settings
    LeadCapacity = 204,             // Capacity of the lead acid battery (Ah)
    GridType = 205,                 // 
    GridPeakShavingPower = 206,     // (0.1kW)
    GridPeakShavingSoc = 207,       // (%)
    GridPeakShavingVolt = 208,      // (0.1V)
    SmartLoadOnVolt = 213,          // (0.1V)
    SmartLoadOffVolt = 214,         // (0.1V)
    SmartLoadOnSoc = 215,           // (%)
    SmartLoadOffSoc = 216,          // (%)
    StartPVpower = 217,             // (0.1kW)
    GridPeakShavingSoc1 = 218,      // (%)
    GridPeakShavingVolt1 = 219,     // (0.1V)
    ACCoupleStartSoc = 220,         // (%)
    ACCoupleEndSoc = 221,           // (%)
    ACCoupleStartVolt = 222,        // (0.1V)
    ACCoupleEndVolt = 223,          // (0.1V)
    LCDConfig = 224,                // LCD version, screen type, machine model code
    LCDPassword = 225,              // Password for LCD Advanced page
    BatStopChargeSoc = 227,         // When battery SOC reaches set value, inverter will stop charging the battery, and when the battery SOC < = (Set value -5), inverter will return charging the battery (%)
    BatStopChargeVolt = 228,        // When battery Voltage reaches set value, inverter will stop charging the battery, and when the battery Volt <= (Set value - 20), inverter will return charging the battery (0.1V)
    MeterConfig = 230,              // 
    ResetRecord = 231,              // 
    GridPeakShavingPower1 = 232,    // (0.1kW)
    FunctionEnable4 = 233,          // Function Enable 4 bits
    QuickChargeTime = 234,          // 
    NoFullChargeDay = 235,          // Counters relative to full battery charge
    FloatChargeThreshold = 236,     // When charge current in CV getting lower than this setting, switch to float charge (0.01C)
    GenCoolDownTime = 237,          // Gen cool down time when dry contactor is off (0.1min)
    AllowService = 241,             // 0=disable, non 0=enable
}

#[derive(Clone)]
pub struct RegisterConfig<'a> {
    pub register: Register,
    pub scale: f64, // raw register value is multiplied by this number. For example, value=171, scale=0.1, readable output=17.1
    pub unit_of_measurement: &'a str,
}

pub fn find_register_config(register: u16) -> Option<RegisterConfig<'static>> {
    let configs = [
        RegisterConfig {
            register: Register::GenRatePower,
            scale: 0.1,
            unit_of_measurement: "kW",
        },
        RegisterConfig {
            register: Register::MaxGenChargeBatCurr,
            scale: 1.0,
            unit_of_measurement: "A",
        },
        RegisterConfig {
            register: Register::GenCoolDownTime,
            scale: 0.1,
            unit_of_measurement: "min",
        },
    ];

    for config in configs {
        if config.register as u16 == register {
            return Some(config);
        }
    }

    None
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register21Bit {
    // Register 21
    OffGridModeEnable = 1 << 0,
    OverfrequencyLoadDerateEnable = 1 << 1,
    DRMSEnable = 1 << 2,
    LVTREnable = 1 << 3,
    AntiIslandEnable = 1 << 4,
    NeutralDetectEnable = 1 << 5,
    GridOnPowerSoftStartEnable = 1 << 6,
    AcChargeEnable = 1 << 7,
    OffGridSeamlessSwitchingEnable = 1 << 8,
    SetToStandby = 1 << 9,
    ForcedDischargeEnable = 1 << 10,
    ChargePriorityEnable = 1 << 11,
    ISOEnable = 1 << 12,
    GFCIEnable = 1 << 13,
    DCIEnable = 1 << 14,
    FeedInGridEnable = 1 << 15,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register110Bit {
    // Register 110
    PVGridOffEnable = 1 << 0,
    FastZeroExportEnable = 1 << 1,
    MicroGridEnable = 1 << 2,
    BatShared = 1 << 3,
    ChargeLastEnable = 1 << 4,
    BuzzerEnable = 1 << 7,
    TakeLoadTogether = 1 << 10,
    OnGridWorkingMode = 1 << 11,
    GreenModeEnable = 1 << 14,
    EcoModeEnable = 1 << 15,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register113Bit {
    // Register 113
    ClearDetectedPhases = 1 << 0,
    SetComposedPhaseR = 1 << 1,
    SetComposedPhaseS = 2 << 1,
    SetComposedPhaseT = 3 << 1,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register120Bit {
    // Register 120
    HalfHourAcChargeStartEnable = 1 << 0,
    //AcChargeType = 1 << 0,
    OnGridEodType = 1 << 6,
    GenChargeType = 1 << 7,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register179Bit {
    // Register 179
    AcCTDirection = 1 << 0,
    PvCTDirection = 1 << 1,
    AFCIAlarmClear = 1 << 2,
    PvSellFirst = 1 << 3,
    VoltWattEnable = 1 << 4,
    TriptimeUnit = 1 << 5,
    ActPowerCmdEnable = 1 << 6,
    GridPeakShaving = 1 << 7,
    GenPeakShaving = 1 << 8,
    BatChargeControl = 1 << 9,
    BatDischargeControl = 1 << 10,
    AcCoupling = 1 << 11,
    PvArcEnable = 1 << 12,
    SmartLoadEnable = 1 << 13,
    RsdDisable = 1 << 14,
    OnGridAlwaysOn = 1 << 15,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register233Bit {
    // Register 233
    QuickChargeStartEnable = 1 << 0,
    BattBackupEnable = 1 << 1,
    MaintenanceEnable = 1 << 2,
    WorkingMode = 1 << 3,
}

// Input71Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Input71Bits {
    pub auto_test_start: String,
    pub ub_auto_test_status: String,
    pub ub_auto_test_step: String,
}
impl Input71Bits {
    fn auto_test_start_string(value: u16) -> String {
        match value {
            0 => "Not Started".to_string(),
            1 => "Started".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn auto_test_status_string(value: u16) -> String {
        match value {
            0 => "Waiting".to_string(),
            1 => "Testing".to_string(),
            2 => "Test Fail".to_string(),
            3 => "V Test OK".to_string(),
            4 => "F Test OK".to_string(),
            5 => "Test Pass".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn auto_test_step_string(value: u16) -> String {
        match value {
            1 => "V1L Test".to_string(),
            2 => "V1H Test".to_string(),
            3 => "F1L Test".to_string(),
            4 => "F1H Test".to_string(),
            5 => "V2L Test".to_string(),
            6 => "V2H Test".to_string(),
            7 => "F2L Test".to_string(),
            8 => "F2H Test".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            auto_test_start: Self::auto_test_start_string((data >> 0) & 0b1111),
            ub_auto_test_status: Self::auto_test_status_string((data >> 4) & 0b1111),
            ub_auto_test_step: Self::auto_test_step_string((data >> 8) & 0b1111),
        }
    }
} // }}}

// Input77Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Input77Bits {
    pub ac_input_type: String,
    pub ac_couple_inverter_flow: String,
    pub ac_couple_enable: String,
}
impl Input77Bits {
    fn ac_input_type_string(value: u16) -> String {
        match value {
            0 => "Grid".to_string(),
            1 => "Generator".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            ac_input_type: Self::ac_input_type_string((data >> 0) & 0b1),
            ac_couple_inverter_flow: Self::is_bit_set(data, 1 << 1),
            ac_couple_enable: Self::is_bit_set(data, 1 << 2),
        }
    }
} // }}}

// Input113Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Input113Bits {
    pub master_or_slave: String,
    pub single_or_three_phase: String,
    pub phases_sequence: String,
    pub parallel_num: u8,
}
impl Input113Bits {
    fn master_or_slave_string(value: u16) -> String {
        match value {
            1 => "Master".to_string(),
            2 => "Slave".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn single_or_three_phase_string(value: u16) -> String {
        match value {
            1 => "R".to_string(),
            2 => "S".to_string(),
            3 => "T".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn phases_sequence_string(value: u16) -> String {
        match value {
            0 => "Positive Order".to_string(),
            1 => "Negative Order".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            master_or_slave: Self::master_or_slave_string((data >> 0) & 0b11),
            single_or_three_phase: Self::single_or_three_phase_string((data >> 2) & 0b11),
            phases_sequence: Self::phases_sequence_string((data >> 4) & 0b11),
            parallel_num: ((data >> 8) & 0b11111111) as u8,
        }
    }
} // }}}

// Input144Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Input144Bits {
    pub afci_flag_arc_alarm_ch1: String,
    pub afci_flag_arc_alarm_ch2: String,
    pub afci_flag_arc_alarm_ch3: String,
    pub afci_flag_arc_alarm_ch4: String,
    pub afci_flag_self_test_fail_ch1: String,
    pub afci_flag_self_test_fail_ch2: String,
    pub afci_flag_self_test_fail_ch3: String,
    pub afci_flag_self_test_fail_ch4: String,
}
impl Input144Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            afci_flag_arc_alarm_ch1: Self::is_bit_set(data, 1 << 0),
            afci_flag_arc_alarm_ch2: Self::is_bit_set(data, 1 << 1),
            afci_flag_arc_alarm_ch3: Self::is_bit_set(data, 1 << 2),
            afci_flag_arc_alarm_ch4: Self::is_bit_set(data, 1 << 3),
            afci_flag_self_test_fail_ch1: Self::is_bit_set(data, 1 << 4),
            afci_flag_self_test_fail_ch2: Self::is_bit_set(data, 1 << 5),
            afci_flag_self_test_fail_ch3: Self::is_bit_set(data, 1 << 6),
            afci_flag_self_test_fail_ch4: Self::is_bit_set(data, 1 << 7),
        }
    }
} // }}}

// Register21Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register21Bits {
    pub eps_en: String,
    pub ovf_load_derate_en: String,
    pub drms_en: String,
    pub lvrt_en: String,
    pub anti_island_en: String,
    pub neutral_detect_en: String,
    pub grid_on_power_ss_en: String,
    pub ac_charge_en: String,
    pub sw_seamless_en: String,
    pub set_to_standby: String,
    pub forced_discharge_en: String,
    pub charge_priority_en: String,
    pub iso_en: String,
    pub gfci_en: String,
    pub dci_en: String,
    pub feed_in_grid_en: String,
}

impl Register21Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            eps_en: Self::is_bit_set(data, 1 << 0),
            ovf_load_derate_en: Self::is_bit_set(data, 1 << 1),
            drms_en: Self::is_bit_set(data, 1 << 2),
            lvrt_en: Self::is_bit_set(data, 1 << 3),
            anti_island_en: Self::is_bit_set(data, 1 << 4),
            neutral_detect_en: Self::is_bit_set(data, 1 << 5),
            grid_on_power_ss_en: Self::is_bit_set(data, 1 << 6),
            ac_charge_en: Self::is_bit_set(data, 1 << 7),
            sw_seamless_en: Self::is_bit_set(data, 1 << 8),
            set_to_standby: Self::is_bit_set(data, 1 << 9),
            forced_discharge_en: Self::is_bit_set(data, 1 << 10),
            charge_priority_en: Self::is_bit_set(data, 1 << 11),
            iso_en: Self::is_bit_set(data, 1 << 12),
            gfci_en: Self::is_bit_set(data, 1 << 13),
            dci_en: Self::is_bit_set(data, 1 << 14),
            feed_in_grid_en: Self::is_bit_set(data, 1 << 15),
        }
    }
} // }}}

// Register110Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register110Bits {
    pub ub_pv_grid_off_en: String,
    pub ub_run_without_grid: String,
    pub ub_micro_grid_en: String,
    pub ub_bat_shared_en: String,
    pub ub_charge_last_en: String,
    pub ct_sample_ratio: String,
    pub buzzer_en: String,
    pub pv_ct_sample_type: String,
    pub take_load_together: String,
    pub on_grid_working_mode: String,
    pub pv_ct_sample_ratio: String,
    pub green_mode_en: String,
    pub eco_mode_en: String,
}
impl Register110Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            ub_pv_grid_off_en: Self::is_bit_set(data, 1 << 0),
            ub_run_without_grid: Self::is_bit_set(data, 1 << 1),
            ub_micro_grid_en: Self::is_bit_set(data, 1 << 2),
            ub_bat_shared_en: Self::is_bit_set(data, 1 << 3),
            ub_charge_last_en: Self::is_bit_set(data, 1 << 4),
            ct_sample_ratio: "Unknown".to_string(), // todo
            buzzer_en: Self::is_bit_set(data, 1 << 7),
            pv_ct_sample_type: "Unknown".to_string(), // todo
            take_load_together: Self::is_bit_set(data, 1 << 10),
            on_grid_working_mode: "Unknown".to_string(), // todo
            pv_ct_sample_ratio: "Unknown".to_string(), // todo
            green_mode_en: Self::is_bit_set(data, 1 << 14),
            eco_mode_en: Self::is_bit_set(data, 1 << 15),
        }
    }
} // }}}

// Register120Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register120Bits {
    pub half_hour_ac_charge_start_en: String,
    pub ac_charge_type: String,
    pub discharge_ctrl_type: String,
    pub on_grid_eod_type: String,
    pub gen_charge_type: String,
}
impl Register120Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    fn ac_charge_type_string(status: u16) -> &'static str {
        match status {
            0 => "Disable",
            1 => "According to time",
            2 => "According to voltage",
            3 => "According to state of charge",
            4 => "According to voltage and time",
            5 => "According to state of charge and time",
            _ => "Unknown",
        }
    }
    
    fn ac_charge_type_12k_hybrid_string(status: u16) -> String {
        match status {
            0 => "According to time".to_string(),
            1 => "According to state of charge and voltage".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn discharge_ctrl_type_string(status: u16) -> String {
        match status {
            0 => "According to voltage".to_string(),
            1 => "According to state of charge".to_string(),
            2 => "According to state of charge and voltage".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn on_grid_eod_type_string(status: u16) -> String {
        match status {
            0 => "According to voltage".to_string(),
            1 => "According to state of charge".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn gen_charge_type_string(status: u16) -> String {
        match status {
            0 => "According to voltage".to_string(),
            1 => "According to state of charge".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            half_hour_ac_charge_start_en: Self::is_bit_set(data, 1 << 0),
            ac_charge_type: Self::ac_charge_type_12k_hybrid_string((data >> 1) & 0b111),
            discharge_ctrl_type: Self::discharge_ctrl_type_string((data >> 4) & 0b11),
            on_grid_eod_type: Self::on_grid_eod_type_string((data >> 6) & 0b1),
            gen_charge_type: Self::gen_charge_type_string((data >> 7) & 0b1),
        }
    }
} // }}}

// Register179Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register179Bits {
    pub ac_ct_direction: String,
    pub pv_ct_direction: String,
    pub afci_alarm_clear: String,
    pub pv_sell_first: String,
    pub volt_watt_en: String,
    pub triptime_unit: String,
    pub act_power_cmd_en: String,
    pub ub_grid_peak_shaving: String,
    pub ub_gen_peak_shaving: String,
    pub ub_bat_charge_control: String,
    pub ub_bat_dischg_control: String,
    pub ub_ac_coupling: String,
    pub ub_pv_arc_en: String,
    pub ub_smart_load_en: String,
    pub ub_rsd_disable: String,
    pub on_grid_always_on: String,
}
impl Register179Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }
    fn is_bit_set_reversed(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "OFF".to_string()
        } else {
            "ON".to_string()
        }
    }

    fn direction_bit(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "Reversed".to_string()
        } else {
            "Normal".to_string()
        }
    }

    fn control_bit(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "Volt".to_string()
        } else {
            "State of Charge".to_string()
        }
    }

    fn smart_load_bit(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "Smart Load".to_string()
        } else {
            "Generator".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            ac_ct_direction: Self::direction_bit(data, 1 << 0),
            pv_ct_direction: Self::direction_bit(data, 1 << 1),
            afci_alarm_clear: "Unknown".to_string(), // todo
            pv_sell_first: Self::is_bit_set(data, 1 << 3),
            volt_watt_en: Self::is_bit_set(data, 1 << 4),
            triptime_unit: Self::is_bit_set(data, 1 << 5),
            act_power_cmd_en: Self::is_bit_set(data, 1 << 6),
            ub_grid_peak_shaving: Self::is_bit_set(data, 1 << 7),
            ub_gen_peak_shaving: Self::is_bit_set(data, 1 << 8),
            ub_bat_charge_control: Self::control_bit(data, 1 << 9),
            ub_bat_dischg_control: Self::control_bit(data, 1 << 10),
            ub_ac_coupling: Self::is_bit_set(data, 1 << 11),
            ub_pv_arc_en: Self::is_bit_set(data, 1 << 12),
            ub_smart_load_en: Self::smart_load_bit(data, 1 << 13),
            ub_rsd_disable: Self::is_bit_set_reversed(data, 1 << 14),
            on_grid_always_on: Self::is_bit_set(data, 1 << 15),
        }
    }
} // }}}

// Register224Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register224Bits {
    pub lcd_version: u8,
    pub lcd_screen_type: String,
    pub lcd_odm: String,
    pub lcd_machine_model_code: String,
}
impl Register224Bits {
    fn lcd_screen_type_string(data: u16) -> String {
        match data {
            0 => "Screen of B size".to_string(),
            1 => "Screen of S size".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    fn lcd_odm_string(data: u16) -> String {
        match data {
            0 => "Luxpower".to_string(),
            1 => "Customized".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    fn lcd_machine_model_code_string(data: u16) -> String {
        match data {
            0 => "LXP 12K".to_string(),
            1 => "All-in-one".to_string(),
            2 => "Tri-Phase 20k".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            lcd_version: (data & 0b11111111) as u8,
            lcd_screen_type: Self::lcd_screen_type_string((data >> 8) & 1),
            lcd_odm: Self::lcd_odm_string((data >> 9) & 0b11),
            lcd_machine_model_code: Self::lcd_machine_model_code_string((data >> 11) & 0b11111),
        }
    }
} // }}}

// Register230Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register230Bits {
    pub meters_num: u8,
    pub meter_measure_type: String,
    pub install_phase: String,
}
impl Register230Bits {
    fn meter_measure_type_string(data: u16) -> String {
        match data {
            0 => "Meter 1 measure AC, Meter 2 measure PV".to_string(),
            1 => "Meter 1 measure PV, Meter 2 measure AC".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    fn install_phase_string(data: u16) -> String {
        match data {
            0 => "R phase".to_string(),
            1 => "S phase".to_string(),
            2 => "T phase".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            meters_num: (data & 0b1111) as u8,
            meter_measure_type: Self::meter_measure_type_string((data >> 8) & 1),
            install_phase: Self::install_phase_string((data >> 9) & 0b11),
        }
    }
} // }}}

// Register233Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register233Bits {
    pub ub_quick_charge_start_en: String,
    pub ub_batt_backup_en: String,
    pub ub_maintenance_en: String,
    pub ub_working_mode: String,
}
impl Register233Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }
    fn ub_working_mode_bit(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "Work mode 2".to_string()
        } else {
            "Work mode 1".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            ub_quick_charge_start_en: Self::is_bit_set(data, 1 << 0),
            ub_batt_backup_en: Self::is_bit_set(data, 1 << 1),
            ub_maintenance_en: Self::is_bit_set(data, 1 << 2),
            ub_working_mode: Self::ub_working_mode_bit(data, 1 << 3),
        }
    }
} // }}}

// Register235Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register235Bits {
    pub no_full_charge_days: u8,
    pub no_full_charge_days_num_set: u8,
}
impl Register235Bits {
    pub fn new(data: u16) -> Self {
        Self {
            no_full_charge_days: (data & 0b11111111) as u8,
            no_full_charge_days_num_set: ((data & 0b11111111) >> 8) as u8,
        }
    }
} // }}}
    
#[enum_dispatch]
pub trait PacketCommon {
    fn datalog(&self) -> Serial;
    fn set_datalog(&mut self, datalog: Serial);
    fn inverter(&self) -> Option<Serial>;
    fn set_inverter(&mut self, serial: Serial);
    fn protocol(&self) -> u16;
    fn tcp_function(&self) -> TcpFunction;
    fn bytes(&self) -> Vec<u8>;

    fn register(&self) -> u16 {
        unimplemented!("register() not implemented");
    }
    fn value(&self) -> u16 {
        unimplemented!("value() not implemented");
    }
}

pub struct TcpFrameFactory;
impl TcpFrameFactory {
    pub fn build(data: &Packet) -> Vec<u8> {
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

#[enum_dispatch(PacketCommon)]
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Packet {
    Heartbeat(Heartbeat),
    TranslatedData(TranslatedData),
    ReadParam(ReadParam),
    WriteParam(WriteParam),
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

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Heartbeat {
    pub datalog: Serial,
}
impl Heartbeat {
    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 19 {
            bail!("heartbeat packet too short");
        }

        // assert that the final byte is 0, meaning 0 data bytes follow it
        if input[18] != 0 {
            bail!("heartbeat with non-zero ({}) length byte?", input[18]);
        }

        let datalog = Serial::new(&input[8..18])?;

        Ok(Self { datalog })
    }
}

impl PacketCommon for Heartbeat {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }
    fn inverter(&self) -> Option<Serial> {
        None
    }
    fn set_inverter(&mut self, _datalog: Serial) {}

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::Heartbeat
    }

    fn bytes(&self) -> Vec<u8> {
        vec![0]
    }
}

/////////////
//
// TRANSLATED DATA
//
/////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TranslatedData {
    pub datalog: Serial,
    pub device_function: DeviceFunction, // ReadHold or ReadInput etc..
    pub inverter: Serial,                // inverter serial
    pub register: u16,                   // first register of values
    pub values: Vec<u8>,                 // undecoded, since can be u16 or u32s?
}
impl TranslatedData {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    pub fn read_input(&self) -> Result<ReadInput> {
        // note len() is of Vec<u8>, so not register count
        match (self.register, self.values.len()) {
            (0, 254) => Ok(ReadInput::ReadInputAll(Box::new(self.read_input_all()?))),
            // (127, 254) has been seen but containing all zeroes, not sure what they are
            (127, 254) => Ok(ReadInput::ReadInputAll2(Box::new(self.read_input_all2()?))),
            (0, 80) => Ok(ReadInput::ReadInput1(self.read_input1()?)),
            (40, 80) => Ok(ReadInput::ReadInput2(self.read_input2()?)),
            (80, 80) => Ok(ReadInput::ReadInput3(self.read_input3()?)),
            (r1, r2) => bail!("unhandled ReadInput register={} len={}", r1, r2),
        }
    }

    fn read_input_all(&self) -> Result<ReadInputAll> {
        match ReadInputAll::parse(&self.values) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.p_grid = r.p_to_user as i32 - r.p_to_grid as i32;
                r.p_battery = r.p_charge as i32 - r.p_discharge as i32;
                r.e_pv_day = Utils::round(r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3, 1);
                r.e_pv_all = Utils::round(r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3, 1);
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("read_input_all err")),
        }
    }

    fn read_input_all2(&self) -> Result<ReadInputAll2> {
        match ReadInputAll2::parse(&self.values) {
            Ok((_, mut r)) => {
                // r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                // r.p_grid = r.p_to_user as i32 - r.p_to_grid as i32;
                // r.p_battery = r.p_charge as i32 - r.p_discharge as i32;
                // r.e_pv_day = Utils::round(r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3, 1);
                // r.e_pv_all = Utils::round(r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3, 1);
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("read_input_all2 err")),
        }
    }

    fn read_input1(&self) -> Result<ReadInput1> {
        match ReadInput1::parse(&self.values) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.p_grid = r.p_to_user as i32 - r.p_to_grid as i32;
                r.p_battery = r.p_charge as i32 - r.p_discharge as i32;
                r.e_pv_day = Utils::round(r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3, 1);
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("read_input1 err")),
        }
    }

    fn read_input2(&self) -> Result<ReadInput2> {
        match ReadInput2::parse(&self.values) {
            Ok((_, mut r)) => {
                r.e_pv_all = Utils::round(r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3, 1);
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("read_input2 err")),
        }
    }

    fn read_input3(&self) -> Result<ReadInput3> {
        match ReadInput3::parse(&self.values) {
            Ok((_, mut r)) => {
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("read_input3 err")),
        }
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 38 {
            bail!("TranslatedData::decode packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[20..len - 2];

        let checksum = &input[len - 2..];
        if Self::checksum(data) != checksum {
            bail!(
                "TranslatedData::decode checksum mismatch - got {:?}, expected {:?}",
                checksum,
                Self::checksum(data)
            );
        }

        //let address = data[0]; // 0=client, 1=inverter?
        let device_function = DeviceFunction::try_from(data[1])?;
        let inverter = Serial::new(&data[2..12])?;
        let register = Utils::u16ify(data, 12);

        let mut value_len = 2;
        let mut value_offset = 14;

        if Self::has_value_length_byte(PacketSource::Inverter, protocol, device_function) {
            value_len = data[value_offset] as usize;
            value_offset += 1;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "TranslatedData::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            );
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

        let p1 = protocol == 1;
        let psi = source == PacketSource::Inverter;
        match device_function {
            ReadHold | ReadInput => !p1 && psi,
            WriteSingle => false,
            WriteMulti => !p1 && !psi,
        }
    }

    fn checksum(data: &[u8]) -> [u8; 2] {
        crc16::State::<crc16::MODBUS>::calculate(data).to_le_bytes()
    }
}

impl PacketCommon for TranslatedData {
    fn protocol(&self) -> u16 {
        if self.device_function == DeviceFunction::WriteMulti {
            2
        } else {
            1
        }
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }

    fn inverter(&self) -> Option<Serial> {
        Some(self.inverter)
    }
    fn set_inverter(&mut self, serial: Serial) {
        self.inverter = serial;
    }

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::TranslatedData
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = vec![0; 16];

        // data[2] (address) is 0 when writing to inverter, 1 when reading from it
        data[3] = self.device_function as u8;

        // experimental: looks like maybe you don't need to fill this in..
        data[4..14].copy_from_slice(&self.inverter.data());
        //data[4..14].copy_from_slice(&[0; 10]);

        data[14..16].copy_from_slice(&self.register.to_le_bytes());

        if self.device_function == DeviceFunction::WriteMulti {
            let register_count = self.pairs().len() as u16;
            data.extend_from_slice(&register_count.to_le_bytes());
        }

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

        // the first two bytes are the data length, excluding checksum which we'll add next
        let data_length = data.len() as u16;
        data[0..2].copy_from_slice(&data_length.to_le_bytes());

        // checksum does not include the first two bytes (data length)
        data.extend_from_slice(&Self::checksum(&data[2..]));

        data
    }

    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        Utils::u16ify(&self.values, 0)
    }
}

/////////////
//
// READ PARAM
//
/////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ReadParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or i32s?
}
impl ReadParam {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 24 {
            bail!("ReadParam::decode packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[18..];
        let register = Utils::u16ify(data, 0);

        let mut value_len = 2;
        let mut value_offset = 2;

        if Self::has_value_length_bytes(protocol) {
            value_len = Utils::u16ify(data, value_offset) as usize;
            value_offset += 2;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "ReadParam::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            );
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

impl PacketCommon for ReadParam {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }
    fn inverter(&self) -> Option<Serial> {
        None
    }
    fn set_inverter(&mut self, _datalog: Serial) {}

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::ReadParam
    }

    fn bytes(&self) -> Vec<u8> {
        vec![self.register() as u8, 0]
    }

    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        Utils::u16ify(&self.values, 0)
    }
}

/////////////
//
// WRITE PARAM
//
/////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct WriteParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or i32s?
}
impl WriteParam {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 21 {
            bail!("WriteParam::decode packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[18..];
        let register = u16::from(data[0]);

        let mut value_len = 2;
        let mut value_offset = 1;

        if Self::has_value_length_bytes(protocol) {
            value_len = Utils::u16ify(data, value_offset) as usize;
            value_offset += 2;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "WriteParam::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            );
        }

        Ok(Self {
            datalog,
            register,
            values,
        })
    }

    fn has_value_length_bytes(_protocol: u16) -> bool {
        false
    }
}

impl PacketCommon for WriteParam {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }
    fn inverter(&self) -> Option<Serial> {
        None
    }
    fn set_inverter(&mut self, _datalog: Serial) {}

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::WriteParam
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = vec![0; 2];

        data[0..2].copy_from_slice(&self.register.to_le_bytes());

        let len = self.values.len() as u16;
        data.extend_from_slice(&len.to_le_bytes());

        let mut m = Vec::new();
        for i in &self.values {
            m.extend_from_slice(&i.to_le_bytes());
        }
        data.append(&mut m);

        data
    }

    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        Utils::u16ify(&self.values, 0)
    }
}

pub struct Parser;
impl Parser {
    pub fn parse(input: &[u8]) -> Result<Packet> {
        let input_len = input.len() as u8;
        if input_len < 18 {
            bail!("packet less than 18 bytes?");
        }

        if input[0..2] != [161, 26] {
            bail!("invalid packet prefix");
        }

        if input_len < input[4] - 6 {
            bail!(
                "Parser::parse mismatch: input.len()={},  frame_length={}",
                input_len,
                input[4] - 6
            );
        }

        let r = match TcpFunction::try_from(input[7])? {
            TcpFunction::Heartbeat => Packet::Heartbeat(Heartbeat::decode(input)?),
            TcpFunction::TranslatedData => Packet::TranslatedData(TranslatedData::decode(input)?),
            TcpFunction::ReadParam => Packet::ReadParam(ReadParam::decode(input)?),
            TcpFunction::WriteParam => Packet::WriteParam(WriteParam::decode(input)?),
            //_ => bail!("unhandled: tcp_function={} input={:?}", input[7], input),
        };

        Ok(r)
    }
}

// Register 16
pub struct LanguageString;
impl LanguageString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "English",
            1 => "German",

            _ => "Unknown",
        }
    }
}

// Register 19
pub struct DtcDeviceTypeString;
impl DtcDeviceTypeString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "Default",
            3 => "XOLTA",

            _ => "Unknown",
        }
    }
}

// Register 20
pub struct PVInputModelStandardString;
impl PVInputModelStandardString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "No PV plug in",
            1 => "PV1 plug in",
            2 => "PV2 plug in",
            3 => "Two PVs in parallel",
            4 => "Two separate PVs",

            _ => "Unknown",
        }
    }
}
pub struct PVInputModel12KHybridString;
impl PVInputModel12KHybridString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "No PV plug in",
            1 => "PV1 plug in",
            2 => "PV2 plug in",
            3 => "PV3 plug in",
            4 => "PV1&2 in",
            5 => "PV1&3 in",
            6 => "PV2&3 in",
            7 => "PV1&2&3 in",

            _ => "Unknown",
        }
    }
}
pub struct PVInputModelTriPhase6To20KString;
impl PVInputModelTriPhase6To20KString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "All MPPTs with individual PV strings",
            1 => "PV1&2 in parallel",
            2 => "PV1&3 in parallel",
            3 => "PV2&3 in parallel",
            4 => "PV1&2&3 in parallel",

            _ => "Unknown",
        }
    }
}

// Register 59
pub struct ReactivePowerCmdTypeString;
impl ReactivePowerCmdTypeString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "Unit power factor",
            1 => "Fixed power factor",
            2 => "Default PF curve (American machine: Q(P))",
            3 => "Custom PF curve",
            4 => "Capacitive reactive power percentage",
            5 => "Inductive reactive power percentage",
            6 => "QV curve",
            7 => "QV dynamic",

            _ => "Unknown",
        }
    }
}

// Register 112
pub struct SetSystemTypeString;
impl SetSystemTypeString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "Single Unit",
            1 => "Single-phase parallel (Primary)",
            2 => "Single-phase parallel (Secondary)",
            3 => "Three phase parallel (Master)",
            4 => "2*208 (Master)",
            5 => "Inductive reactive power percentage",
            6 => "QV curve",
            7 => "QV dynamic",

            _ => "Unknown",
        }
    }
}

// Register 145
pub struct OutputPrioConfigString;
impl OutputPrioConfigString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "Battery first",
            1 => "PV first",
            2 => "AC first",
            _ => "Unknown",
        }
    }
}

// Register 146
pub struct LineModeString;
impl LineModeString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "APL (90-280V 20ms)",
            1 => "UPS (170-280V 10ms)",
            2 => "GEN (90-280V 20ms)",
            _ => "Unknown",
        }
    }
}

// Register 205
pub struct GridTypeString;
impl GridTypeString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0 => "Split 240V/120V",
            1 => "Tri-phase 208V/120V",
            2 => "Single 240V",
            3 => "Single 230V",
            4 => "Split 200V/100V",
            _ => "Unknown",
        }
    }
}

pub struct StatusString;
impl StatusString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0x00 => "Standby",
            0x01 => "Fault",
            0x02 => "FW Updating",
            0x04 => "PV On-grid",
            0x08 => "PV Charge",
            0x0C => "PV Charge & On-grid",
            0x10 => "Battery On-grid",
            0x11 => "Bypass",
            0x14 => "PV & Battery On-grid",
            0x19 => "PV Charge + Bypass",
            0x20 => "AC Charge",
            0x28 => "PV & AC Charge",
            0x40 => "Battery Off-grid",
            0x60 => "Off-grid & AC-coupled battery charging",
            0x80 => "PV Off-grid",
            0xC0 => "PV & Battery Off-grid",
            0x88 => "PV Charge Off-grid",

            _ => "Unknown",
        }
    }
}

pub struct WarningCodeString;
impl WarningCodeString {
    pub fn from_value(value: u32) -> &'static str {
        if value == 0 {
            return "OK";
        }

        (0..=31)
            .find(|i| value & (1 << i) > 0)
            .map(Self::from_bit)
            .unwrap()
    }

    fn from_bit(bit: usize) -> &'static str {
        match bit {
            0 => "W000: Battery communication failure",
            1 => "W001: AFCI communication failure",
            2 => "W002: AFCI high",
            3 => "W003: Meter communication failure",
            4 => "W004: Both charge and discharge forbidden by battery",
            5 => "W005: Auto test failed",
            6 => "W006: Reserved",
            7 => "W007: LCD communication failure",
            8 => "W008: FW version mismatch",
            9 => "W009: Fan stuck",
            10 => "W010: Reserved",
            11 => "W011: Parallel number out of range",
            12 => "W012: Bat On Mos",
            13 => "W013: Overtemperature (NTC reading is too high)",
            14 => "W014: Reserved",
            15 => "W015: Battery reverse connection",
            16 => "W016: Grid power outage",
            17 => "W017: Grid voltage out of range",
            18 => "W018: Grid frequency out of range",
            19 => "W019: Reserved",
            20 => "W020: PV insulation low",
            21 => "W021: Leakage current high",
            22 => "W022: DCI high",
            23 => "W023: PV short",
            24 => "W024: Reserved",
            25 => "W025: Battery voltage high",
            26 => "W026: Battery voltage low",
            27 => "W027: Battery open circuit",
            28 => "W028: EPS overload",
            29 => "W029: EPS voltage high",
            30 => "W030: Meter reverse connection",
            31 => "W031: DCV high",

            _ => todo!("Unknown Warning"),
        }
    }
}

pub struct FaultCodeString;
impl FaultCodeString {
    pub fn from_value(value: u32) -> &'static str {
        if value == 0 {
            return "OK";
        }

        (0..=31)
            .find(|i| value & (1 << i) > 0)
            .map(Self::from_bit)
            .unwrap()
    }

    fn from_bit(bit: usize) -> &'static str {
        match bit {
            0 => "E000: Internal communication fault 1",
            1 => "E001: Model fault",
            2 => "E002: BatOnMosFail",
            3 => "E003: CT Fail",
            4 => "E004: Reserved",
            5 => "E005: Reserved",
            6 => "E006: Reserved",
            7 => "E007: Reserved",
            8 => "E008: CAN communication error in parallel system",
            9 => "E009: master lost in parallel system",
            10 => "E010: multiple master units in parallel system",
            11 => "E011: AC input inconsistent in parallel system",
            12 => "E012: UPS short",
            13 => "E013: Reverse current on UPS output",
            14 => "E014: Bus short",
            15 => "E015: Phase error in three phase system",
            16 => "E016: Relay check fault",
            17 => "E017: Internal communication fault 2",
            18 => "E018: Internal communication fault 3",
            19 => "E019: Bus voltage high",
            20 => "E020: EPS connection fault",
            21 => "E021: PV voltage high",
            22 => "E022: Over current protection",
            23 => "E023: Neutral fault",
            24 => "E024: PV short",
            25 => "E025: Radiator temperature over range",
            26 => "E026: Internal fault",
            27 => "E027: Sample inconsistent between Main CPU and redundant CPU",
            28 => "E028: Reserved",
            29 => "E029: Reserved",
            30 => "E030: Reserved",
            31 => "E031: Internal communication fault 4",
            _ => todo!("Unknown Fault"),
        }
    }
}
