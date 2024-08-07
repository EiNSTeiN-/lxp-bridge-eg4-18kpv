use crate::prelude::*;
use lxp::packet::Register;

use serde::{Serialize, Serializer};

// ValueTemplate {{{
#[derive(Clone, Debug, PartialEq)]
pub enum ValueTemplate {
    None,
    Default, // "{{ value_json }}"
    FromKey, // "{{ value_json.$key }}"
    String(String),
}
impl ValueTemplate {
    pub fn from_default() -> Self {
        Self::String("{{ value_json }}".to_string())
    }
    pub fn from_key(key: &str) -> Self {
        Self::String(format!("{{{{ value_json.{} }}}}", key))
    }
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }
    pub fn is_from_key(&self) -> bool {
        *self == Self::FromKey
    }
}
impl Serialize for ValueTemplate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueTemplate::String(str) => serializer.serialize_str(str),
            _ => unreachable!(),
        }
    }
} // }}}

// StateTopic {{{
#[derive(Clone, Debug, PartialEq)]
pub enum StateTopic {
    Default, // "{namespace}/{datalog}/input/{key}/parsed"
    String(String),
}
impl StateTopic {
    pub fn from_default(namespace: &str, datalog: Serial, key: &str) -> Self {
        Self::String(format!("{}/{}/input/{}/parsed", namespace, datalog, key))
    }
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }
}
impl Serialize for StateTopic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StateTopic::String(str) => serializer.serialize_str(str),
            _ => unreachable!(),
        }
    }
} // }}}

#[derive(Clone, Debug, Serialize)]
pub struct Availability {
    topic: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Device {
    manufacturer: String,
    name: String,
    identifiers: [String; 1],
    // model: String, // TODO: provide inverter model
}

pub struct Config {
    inverter: config::Inverter,
    mqtt_config: config::Mqtt,
}

// https://www.home-assistant.io/integrations/sensor.mqtt/
#[derive(Clone, Debug, Serialize)]
pub struct Entity<'a> {
    // this is not serialised into the JSON output, just used as a transient store to
    // work out what unique_id and topic should be
    #[serde(skip)]
    key: &'a str, // for example, soc
    #[serde(skip)]
    is_binary_sensor: bool, // for example, soc

    unique_id: &'a str, // lxp_XXXX_soc
    name: &'a str,      // really more of a label? for example, "State of Charge"

    state_topic: StateTopic,

    // these are all skipped in the output if None. this lets us use the same struct for
    // different types of entities, just our responsibility to make sure a sane set of attributes
    // are populated. Could make subtypes to enforce the various attributes being set for different
    // HA entity types but I think its not worth the extra complexity.
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<&'a str>,
    #[serde(skip_serializing_if = "ValueTemplate::is_none")]
    value_template: ValueTemplate,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'a str>,

    device: Device,
    availability: Availability,
}

// https://www.home-assistant.io/integrations/switch.mqtt/
#[derive(Debug, Serialize)]
pub struct Switch {
    name: String,
    state_topic: String,
    command_topic: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
}

// https://www.home-assistant.io/integrations/number.mqtt/
#[derive(Debug, Serialize)]
pub struct Number {
    name: String,
    state_topic: String,
    command_topic: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
    min: f64,
    max: f64,
    step: f64,
    unit_of_measurement: String,
    mode: String,
}

// https://www.home-assistant.io/integrations/text.mqtt/
#[derive(Debug, Serialize)]
pub struct Text {
    name: String,
    state_topic: String,
    command_topic: String,
    command_template: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
    pattern: String,
}

impl Config {
    pub fn new(inverter: &config::Inverter, mqtt_config: &config::Mqtt) -> Self {
        Self {
            inverter: inverter.clone(),
            mqtt_config: mqtt_config.clone(),
        }
    }

    pub fn sensors(&self) -> Vec<mqtt::Message> {
        let base = Entity {
            key: &String::default(),
            is_binary_sensor: false,
            unique_id: &String::default(),
            name: &String::default(),
            entity_category: None,
            device_class: None,
            state_class: None,
            unit_of_measurement: None,
            icon: None,
            value_template: ValueTemplate::Default, // "{{ value_json }}"
            // TODO: might change this to an enum that defaults to InputsAll but can be replaced
            // with a string for a specific topic?
            state_topic: StateTopic::Default,
            device: self.device(),
            availability: self.availability(),
        };

        let voltage = Entity {
            device_class: Some("voltage"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("V"),
            ..base.clone()
        };

        let frequency = Entity {
            device_class: Some("frequency"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("Hz"),
            ..base.clone()
        };

        let power = Entity {
            device_class: Some("power"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("W"),
            ..base.clone()
        };

        let current = Entity {
            device_class: Some("current"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("A"),
            ..base.clone()
        };

        let energy = Entity {
            device_class: Some("energy"),
            state_class: Some("total_increasing"),
            unit_of_measurement: Some("kWh"),
            ..base.clone()
        };

        let temperature = Entity {
            device_class: Some("temperature"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("°C"),
            ..base.clone()
        };

        // now each entry in here should only have to specify specific overrides for each key.
        // if we have multiple things sharing keys, consider whether to make a new variable to
        // inherit from.
        let sensors = [
            Entity {
                key: "status",
                name: "Status",
                device_class: Some("enum"),
                ..base.clone()
            },
            Entity {
                key: "soc",
                name: "State of Charge",
                device_class: Some("battery"),
                state_class: Some("measurement"),
                unit_of_measurement: Some("%"),
                ..base.clone()
            },
            Entity {
                key: "fault_code",
                name: "Fault Code",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                icon: Some("mdi:alert"),
                ..base.clone()
            },
            Entity {
                key: "warning_code",
                name: "Warning Code",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                icon: Some("mdi:alert-outline"),
                ..base.clone()
            },
            Entity {
                key: "ac_input_type",
                name: "AC Input Type",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_77"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "ac_couple_inverter_flow",
                is_binary_sensor: true,
                name: "AC Couple Inverter Flow",
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_77"),
                entity_category: Some("diagnostic"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "ac_couple_enable",
                is_binary_sensor: true,
                name: "AC Couple Enable",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_77"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "master_or_slave",
                name: "Parallel Inverter Role",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_113"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "single_or_three_phase",
                name: "Parallel Inverter Phase",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_113"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "phases_sequence",
                name: "Parallel Inverter Phases Sequence",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_113"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "parallel_num",
                name: "Parallel Inverter Count",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_113"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_ch1_current",
                name: "AFCI Channel 1 Current",
                entity_category: Some("diagnostic"),
                unit_of_measurement: Some("mA"),
                ..current.clone()
            },
            Entity {
                key: "afci_ch2_current",
                name: "AFCI Channel 2 Current",
                entity_category: Some("diagnostic"),
                unit_of_measurement: Some("mA"),
                ..current.clone()
            },
            Entity {
                key: "afci_ch3_current",
                name: "AFCI Channel 3 Current",
                entity_category: Some("diagnostic"),
                unit_of_measurement: Some("mA"),
                ..current.clone()
            },
            Entity {
                key: "afci_ch4_current",
                name: "AFCI Channel 4 Current",
                entity_category: Some("diagnostic"),
                unit_of_measurement: Some("mA"),
                ..current.clone()
            },
            Entity {
                key: "afci_flag_arc_alarm_ch1",
                is_binary_sensor: true,
                name: "AFCI ARC Alarm Channel 1",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_arc_alarm_ch2",
                is_binary_sensor: true,
                name: "AFCI ARC Alarm Channel 2",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_arc_alarm_ch3",
                is_binary_sensor: true,
                name: "AFCI ARC Alarm Channel 3",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_arc_alarm_ch4",
                is_binary_sensor: true,
                name: "AFCI ARC Alarm Channel 4",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_self_test_fail_ch1",
                name: "AFCI Self Test Fail Channel 1",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_self_test_fail_ch2",
                is_binary_sensor: true,
                name: "AFCI Self Test Fail Channel 2",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_self_test_fail_ch3",
                is_binary_sensor: true,
                name: "AFCI Self Test Fail Channel 3",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_flag_self_test_fail_ch4",
                is_binary_sensor: true,
                name: "AFCI Self Test Fail Channel 4",
                entity_category: Some("diagnostic"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_144"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "afci_arc_ch1",
                name: "Real Time Arc of Channel 1",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_arc_ch2",
                name: "Real Time Arc of Channel 2",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_arc_ch3",
                name: "Real Time Arc of Channel 3",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_arc_ch4",
                name: "Real Time Arc of Channel 4",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_max_arc_ch1",
                name: "Max Arc of Channel 1",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_max_arc_ch2",
                name: "Max Arc of Channel 2",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_max_arc_ch3",
                name: "Max Arc of Channel 3",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "afci_max_arc_ch4",
                name: "Max Arc of Channel 4",
                entity_category: Some("diagnostic"),
                ..base.clone()
            },
            Entity {
                key: "v_bat",
                name: "Battery Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_ac_r",
                name: "Grid Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_1",
                name: "PV Voltage (String 1)",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_2",
                name: "PV Voltage (String 2)",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_3",
                name: "PV Voltage (String 3)",
                ..voltage.clone()
            },
            Entity {
                key: "v_eps_r",
                name: "EPS Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_bus_1",
                name: "Bus 1 Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_bus_2",
                name: "Bus 2 Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_half_bus",
                name: "Half Bus Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_eps_l1",
                name: "EPS Voltage L1",
                ..voltage.clone()
            },
            Entity {
                key: "v_eps_l2",
                name: "EPS Voltage L2",
                ..voltage.clone()
            },
            Entity {
                key: "f_ac",
                name: "Grid Frequency",
                ..frequency.clone()
            },
            Entity {
                key: "f_eps",
                name: "EPS Frequency",
                ..frequency.clone()
            },
            Entity {
                key: "s_eps",
                name: "Apparent EPS Power",
                device_class: Some("apparent_power"),
                unit_of_measurement: Some("VA"),
                ..power.clone()
            },
            Entity {
                key: "s_eps_l1",
                name: "Apparent EPS Power L1",
                device_class: Some("apparent_power"),
                unit_of_measurement: Some("VA"),
                ..power.clone()
            },
            Entity {
                key: "s_eps_l2",
                name: "Apparent EPS Power L2",
                device_class: Some("apparent_power"),
                unit_of_measurement: Some("VA"),
                ..power.clone()
            },
            Entity {
                key: "p_pv",
                name: "PV Power (Array)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_1",
                name: "PV Power (String 1)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_2",
                name: "PV Power (String 2)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_3",
                name: "PV Power (String 3)",
                ..power.clone()
            },
            Entity {
                key: "p_battery",
                name: "Battery Power (discharge is negative)",
                ..power.clone()
            },
            Entity {
                key: "p_charge",
                name: "Battery Charge",
                ..power.clone()
            },
            Entity {
                key: "p_discharge",
                name: "Battery Discharge",
                ..power.clone()
            },
            Entity {
                key: "p_grid",
                name: "Grid Power (export is negative)",
                ..power.clone()
            },
            Entity {
                key: "p_to_user",
                name: "Power from Grid",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid",
                name: "Power to Grid",
                ..power.clone()
            },
            Entity {
                key: "p_eps",
                name: "Active EPS Power",
                ..power.clone()
            },
            Entity {
                key: "p_inv",
                name: "Inverter Power",
                ..power.clone()
            },
            Entity {
                key: "p_rec",
                name: "AC Charge Power",
                ..power.clone()
            },
            Entity {
                key: "p_eps_l1",
                name: "EPS Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_eps_l2",
                name: "EPS Power L2",
                ..power.clone()
            },
            Entity {
                key: "e_pv_all",
                name: "PV Generation (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_1",
                name: "PV Generation (All time) (String 1)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_2",
                name: "PV Generation (All time) (String 2)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_3",
                name: "PV Generation (All time) (String 3)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day",
                name: "PV Generation (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_1",
                name: "PV Generation (Today) (String 1)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_2",
                name: "PV Generation (Today) (String 2)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_3",
                name: "PV Generation (Today) (String 3)",
                ..energy.clone()
            },
            Entity {
                key: "e_chg_all",
                name: "Battery Charge (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_chg_day",
                name: "Battery Charge (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_dischg_all",
                name: "Battery Discharge (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_dischg_day",
                name: "Battery Discharge (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_user_all",
                name: "Energy from Grid (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_user_day",
                name: "Energy from Grid (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_grid_all",
                name: "Energy to Grid (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_grid_day",
                name: "Energy to Grid (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_all",
                name: "Energy from EPS (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_day",
                name: "Energy from EPS (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_rec_all",
                name: "Energy of AC Charging (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_rec_day",
                name: "Energy of AC Charging (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_inv_all",
                name: "Energy of Inverter (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_inv_day",
                name: "Energy of Inverter (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l1_all",
                name: "Energy of EPS L1 (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l1_day",
                name: "Energy of EPS L1  (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l2_all",
                name: "Energy of EPS L2 (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l2_day",
                name: "Energy of EPS L2  (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_gen_day",
                name: "Energy of Generator (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_gen_all",
                name: "Energy of Generator (All Time)",
                ..energy.clone()
            },
            Entity {
                key: "e_load_day",
                name: "Energy of Load (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_load_all",
                name: "Energy of Load (All Time)",
                ..energy.clone()
            },
            Entity {
                key: "eps_overload_ctrl_time",
                name: "EPS Overload Connect Time",
                entity_category: Some("diagnostic"),
                device_class: Some("duration"),
                unit_of_measurement: Some("s"),
                ..base.clone()
            },
            Entity {
                key: "t_inner",
                name: "Inverter Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_rad_1",
                name: "Radiator 1 Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_rad_2",
                name: "Radiator 2 Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_bat",
                name: "Battery Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t1_temp",
                name: "12K BT Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "max_chg_curr",
                name: "Max Charge Current",
                ..current.clone()
            },
            Entity {
                key: "max_dischg_curr",
                name: "Max Discharge Current",
                ..current.clone()
            },
            Entity {
                key: "min_cell_voltage",
                name: "Min Cell Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "charge_volt_ref",
                name: "Recommended Charge Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "dischg_cut_volt",
                name: "Recommended Discharge Cut-Off Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "bat_count",
                name: "Battery Count",
                state_class: Some("measurement"),
                ..base.clone()
            },
            Entity {
                key: "bat_capacity",
                name: "Battery Capacity",
                state_class: Some("measurement"),
                unit_of_measurement: Some("Ah"),
                ..base.clone()
            },
            Entity {
                key: "bat_current",
                name: "Battery Current",
                ..current.clone()
            },
            Entity {
                key: "max_cell_voltage",
                name: "Max Cell Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "min_cell_temp",
                name: "Min Cell Temperature (BMS)",
                ..temperature.clone()
            },
            Entity {
                key: "max_cell_temp",
                name: "Max Cell Temperature (BMS)",
                ..temperature.clone()
            },
            Entity {
                key: "cycle_count",
                name: "Battery Charge Discharge Cycles",
                state_class: Some("measurement"),
                ..base.clone()
            },
            Entity {
                key: "vbat_inv",
                name: "Inverter Battery Voltage Sampling",
                ..voltage.clone()
            },
            Entity {
                key: "v_gen",
                name: "Generator Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "f_gen",
                name: "Generator Frequency",
                ..frequency.clone()
            },
            Entity {
                key: "p_gen",
                name: "Generator Power",
                ..power.clone()
            },
            Entity {
                key: "p_on_grid_load",
                name: "On-grid Load Power",
                ..power.clone()
            },
            Entity {
                key: "p_ac_couple",
                name: "AC Coupled Inverter Power",
                ..power.clone()
            },
            Entity {
                key: "p_load",
                name: "Load Power",
                ..power.clone()
            },

            Entity {
                key: "p_inv_s",
                name: "On-grid Inverter Power of Three-Phase: S-phase",
                ..power.clone()
            },
            Entity {
                key: "p_inv_t",
                name: "On-grid Inverter Power of Three-Phase: T-phase",
                ..power.clone()
            },
            Entity {
                key: "p_rec_s",
                name: "Charging Rectification Power of Three-Phase: S-phase",
                ..power.clone()
            },
            Entity {
                key: "p_rec_t",
                name: "Charging Rectification Power of Three-Phase: T-phase",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid_s",
                name: "Grid Export Power of Three-Phase: S-phase",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid_t",
                name: "Grid Export Power of Three-Phase: T-phase",
                ..power.clone()
            },
            Entity {
                key: "p_to_user_s",
                name: "Grid Import Power of Three-Phase: S-phase",
                ..power.clone()
            },
            Entity {
                key: "p_to_user_t",
                name: "Grid Import Power of Three-Phase: T-phase",
                ..power.clone()
            },
            Entity {
                key: "p_gen_s",
                name: "Generator Power of Three-Phase: S-phase",
                ..power.clone()
            },
            Entity {
                key: "p_gen_t",
                name: "Generator Power of Three-Phase: T-phase",
                ..power.clone()
            },
            Entity {
                key: "inv_rms_curr_s",
                name: "Effective value of Three-Phase Inverter Current: S-phase",
                ..current.clone()
            },
            Entity {
                key: "inv_rms_curr_t",
                name: "Effective value of Three-Phase Inverter Current: T-phase",
                ..current.clone()
            },
            Entity {
                key: "v_grid_l1",
                name: "Grid Voltage L1",
                ..voltage.clone()
            },
            Entity {
                key: "v_grid_l2",
                name: "Grid Voltage L2",
                ..voltage.clone()
            },
            Entity {
                key: "v_gen_l1",
                name: "Generator Voltage L1",
                ..voltage.clone()
            },
            Entity {
                key: "v_gen_l2",
                name: "Generator Voltage L2",
                ..voltage.clone()
            },
            Entity {
                key: "p_inv_l1",
                name: "Inverting Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_inv_l2",
                name: "Inverting Power L2",
                ..power.clone()
            },
            Entity {
                key: "p_rec_l1",
                name: "Rectifying Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_rec_l2",
                name: "Rectifying Power L2",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid_l1",
                name: "Grid Export Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid_l2",
                name: "Grid Export Power L2",
                ..power.clone()
            },
            Entity {
                key: "p_to_user_l1",
                name: "Grid Import Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_to_user_l2",
                name: "Grid Import Power L2",
                ..power.clone()
            },
            Entity {
                key: "auto_test_start",
                name: "Auto Test Started",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_71"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "ub_auto_test_status",
                name: "Auto Test Status",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_71"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "ub_auto_test_step",
                name: "Auto Test Step",
                entity_category: Some("diagnostic"),
                device_class: Some("enum"),
                state_topic: StateTopic::from_default(self.mqtt_config.namespace(), self.inverter.datalog(), "register_71"),
                value_template: ValueTemplate::FromKey,
                ..base.clone()
            },
            Entity {
                key: "runtime",
                name: "Total Runtime",
                entity_category: Some("diagnostic"),
                device_class: Some("duration"),
                state_class: Some("total_increasing"),
                unit_of_measurement: Some("s"),
                ..base.clone()
            },
        ];

        sensors
            .map(|sensor| {
                // fill in unique_id and value_template (if default) which are derived from key
                let mut sensor = Entity {
                    unique_id: &self.unique_id(sensor.key),
                    ..sensor
                };
                if sensor.value_template.is_default() {
                    sensor.value_template = ValueTemplate::from_default();
                }
                if sensor.value_template.is_from_key() {
                    sensor.value_template = ValueTemplate::from_key(sensor.key);
                }
                if sensor.state_topic.is_default() {
                    sensor.state_topic = StateTopic::from_default(
                        self.mqtt_config.namespace(),
                        self.inverter.datalog(),
                        sensor.key
                    );
                }

                let topic = self.ha_discovery_topic(
                    if sensor.is_binary_sensor { "binary_sensor" } else { "sensor" }, 
                    sensor.key
                );
                let payload = serde_json::to_string(&sensor).unwrap();

                debug!("mqtt message payload for home assistant: {} = {}", topic, payload);

                mqtt::Message {
                    topic: topic,
                    retain: true,
                    payload: payload,
                }
            })
            .to_vec()
    }

    pub fn all(&self) -> Result<Vec<mqtt::Message>> {
        let mut r = vec![
            self.switch("ac_charge", "AC Charge")?,
            self.switch("charge_priority", "Charge Priority")?,
            self.switch("forced_discharge", "Forced Discharge")?,
            self.number_percent(Register::ChargePowerPercentCmd, "System Charge Rate (%)")?,
            self.number_percent(Register::DischgPowerPercentCmd, "System Discharge Rate (%)")?,
            self.number_percent(Register::AcChargePowerCmd, "AC Charge Rate (%)")?,
            self.number_percent(Register::AcChargeSocLimit, "AC Charge Limit %")?,
            self.number_percent(Register::ChargePriorityPowerCmd, "Charge Priority Rate (%)")?,
            self.number_percent(Register::ChargePrioritySocLimit, "Charge Priority Limit %")?,
            self.number_percent(Register::ForcedDischgSocLimit, "Forced Discharge Limit %")?,
            self.number_percent(Register::DischgCutOffSocEod, "Discharge Cutoff %")?,
            self.number_percent(
                Register::EpsDischgCutoffSocEod,
                "Discharge Cutoff for EPS %",
            )?,
            self.number_percent(
                Register::AcChargeStartSocLimit,
                "Charge From AC Lower Limit %",
            )?,
            self.number_percent(
                Register::AcChargeEndSocLimit,
                "Charge From AC Upper Limit %",
            )?,
            self.time_range("ac_charge/1", "AC Charge Timeslot 1")?,
            self.time_range("ac_charge/2", "AC Charge Timeslot 2")?,
            self.time_range("ac_charge/3", "AC Charge Timeslot 3")?,
            self.time_range("ac_first/1", "AC First Timeslot 1")?,
            self.time_range("ac_first/2", "AC First Timeslot 2")?,
            self.time_range("ac_first/3", "AC First Timeslot 3")?,
            self.time_range("charge_priority/1", "Charge Priority Timeslot 1")?,
            self.time_range("charge_priority/2", "Charge Priority Timeslot 2")?,
            self.time_range("charge_priority/3", "Charge Priority Timeslot 3")?,
            self.time_range("forced_discharge/1", "Forced Discharge Timeslot 1")?,
            self.time_range("forced_discharge/2", "Forced Discharge Timeslot 2")?,
            self.time_range("forced_discharge/3", "Forced Discharge Timeslot 3")?,
            self.number(Register::GenRatePower, "Generator Rated Power (kW)")?,
            self.number_percent(Register::GenChargeStartSoc, "Generator Start SOC (%)")?,
            self.number_percent(Register::GenChargeEndSoc, "Generator End SOC (%)")?,
            self.number(Register::MaxGenChargeBatCurr, "Generator Max Charge Current (A)")?,
            self.number(Register::GenCoolDownTime, "Generator Cool Down Time (min)")?,
        ];

        r.append(&mut self.sensors());

        Ok(r)
    }

    fn ha_discovery_topic(&self, kind: &str, name: &str) -> String {
        format!(
            "{}/{}/lxp_{}/{}/config",
            self.mqtt_config.homeassistant().prefix(),
            kind,
            self.inverter.datalog(),
            // The forward slash is used in some names (e.g. ac_charge/1) but
            // has semantic meaning in MQTT, so must be changed
            name.replace('/', "_"),
        )
    }

    fn switch(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        let config = Switch {
            value_template: format!("{{{{ value_json.{}_en }}}}", name),
            state_topic: format!(
                "{}/{}/hold/21/bits",
                self.mqtt_config.namespace(),
                self.inverter.datalog()
            ),
            command_topic: format!(
                "{}/cmd/{}/set/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name
            ),
            unique_id: format!("lxp_{}_{}", self.inverter.datalog(), name),
            name: label.to_string(),
            device: self.device(),
            availability: self.availability(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("switch", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn number_percent(&self, register: Register, label: &str) -> Result<mqtt::Message> {
        let reg_config = lxp::packet::find_register_config(register as u16);
        let step = if reg_config.is_none() { 1.0 } else { reg_config.unwrap().scale };

        let config = Number {
            name: label.to_string(),
            state_topic: format!(
                "{}/{}/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as u16,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as u16,
            ),
            value_template: "{{ float(value) }}".to_string(),
            unique_id: format!("lxp_{}_number_{:?}", self.inverter.datalog(), register),
            device: self.device(),
            availability: self.availability(),
            min: 0.0,
            max: 200.0, // some values return 120%, maybe related to fast charge?
            step: step,
            mode: "slider".to_string(),
            unit_of_measurement: "%".to_string(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("number", &format!("{:?}", register)),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn number(&self, register: Register, label: &str) -> Result<mqtt::Message> {
        let reg_config = lxp::packet::find_register_config(register as u16);
        let mut step = 1.0;
        let mut uom = "";

        if !reg_config.is_none() {
            let u_reg_config = reg_config.unwrap();
            step = u_reg_config.scale;
            uom = u_reg_config.unit_of_measurement;
        }

        let config = Number {
            name: label.to_string(),
            state_topic: format!(
                "{}/{}/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as u16,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as u16,
            ),
            value_template: "{{ float(value) }}".to_string(),
            unique_id: format!("lxp_{}_number_{:?}", self.inverter.datalog(), register),
            device: self.device(),
            availability: self.availability(),
            min: 0.0,
            max: 65535.0,
            step: step,
            mode: "box".to_string(),
            unit_of_measurement: uom.to_string(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("number", &format!("{:?}", register)),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    // Models a time range as an MQTT Text field taking values like: 00:00-23:59
    fn time_range(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        let config = Text {
            name: label.to_string(),
            state_topic: format!(
                "{}/{}/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name,
            ),
            command_template: r#"{% set parts = value.split("-") %}{"start":"{{ parts[0] }}", "end":"{{ parts[1] }}"}"#.to_string(),
            value_template: r#"{{ value_json["start"] }}-{{ value_json["end"] }}"#.to_string(),
            unique_id: format!("lxp_{}_text_{}", self.inverter.datalog(), name),
            device: self.device(),
            availability: self.availability(),
            pattern: r"([01]?[0-9]|2[0-3]):[0-5][0-9]-([01]?[0-9]|2[0-3]):[0-5][0-9]".to_string(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("text", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn unique_id(&self, name: &str) -> String {
        format!("lxp_{}_{}", self.inverter.datalog(), name)
    }

    fn device(&self) -> Device {
        Device {
            identifiers: [format!("lxp_{}", self.inverter.datalog())],
            manufacturer: "LuxPower".to_owned(),
            name: format!("lxp_{}", self.inverter.datalog()),
        }
    }

    fn availability(&self) -> Availability {
        Availability {
            topic: format!("{}/LWT", self.mqtt_config.namespace()),
        }
    }
}
