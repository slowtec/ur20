// Copyright (c) 2017 - 2018 slowtec GmbH <markus.kohlhase@slowtec.de>

extern crate byteorder;
#[macro_use]
extern crate num_derive;
extern crate num_traits;
#[macro_use]
extern crate lazy_static;

use std::{fmt::Debug, result, str::FromStr};

mod error;

pub mod ur20_16do_p;
pub mod ur20_1com_232_485_422;
pub mod ur20_2fcnt_100;
pub mod ur20_4ai_rtd_diag;
pub mod ur20_4ai_ui_12;
pub mod ur20_4ai_ui_16_diag;
pub mod ur20_4ao_ui_16;
pub mod ur20_4ao_ui_16_diag;
pub mod ur20_4di_p;
pub mod ur20_4do_p;
pub mod ur20_4ro_co_255;
pub mod ur20_8ai_i_16_diag_hd;
pub mod ur20_fbc_mod_tcp;
pub(crate) mod util;

pub use crate::error::*;

const S5_FACTOR: u16 = 16_384;
const S7_FACTOR: u16 = 27_648;

use crate::ur20_1com_232_485_422::{ProcessInput as RsIn, ProcessOutput as RsOut};
use crate::ur20_2fcnt_100::{ProcessInput as FcntIn, ProcessOutput as FcntOut};

/// Data type used by the module channels.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelValue {
    /// A single bit (0 == false)
    Bit(bool),
    /// A 32-Bit float value.
    Decimal32(f32),
    /// Special input data used by 1COM-232-485-422
    ComRsIn(RsIn),
    /// Special output data used by 1COM-232-485-422
    ComRsOut(RsOut),
    /// Special input data used by 2FCNT-100
    FcntIn(FcntIn),
    /// Special output data used by 2FCNT-100
    FcntOut(FcntOut),
    /// Raw binary data.
    Bytes(Vec<u8>),
    /// The channel is currently disabled.
    Disabled,
    /// The channel has no data at all.
    None,
}

/// A fieldbus independend channel address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address {
    /// Module position (beginning at `0`)
    pub module: usize,
    /// Channel number (beginning at `0`)
    pub channel: usize,
}

type Result<T> = result::Result<T, Error>;

/// A generic description of modules.
pub trait Module: Debug {
    /// Get concrete i/o module type.
    fn module_type(&self) -> ModuleType;
}

/// Describes the general class of a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleCategory {
    /// Digital input modules
    DI,
    /// Digital output modules
    DO,
    /// Analog input modules
    AI,
    /// Analog output modules
    AO,
    /// Counter modules
    CNT,
    /// Pulse-width modulation modules
    PWM,
    /// Resistance temperature detector modules
    RTD,
    /// Thermo couple modules
    TC,
    /// Communication modules
    COM,
    /// Relay output modules
    RO,
    /// Power feed modules
    PF,
}

/// Describes the concrete module type.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleType {
    // Digital input modules
    UR20_4DI_P,
    UR20_4DI_P_3W,
    UR20_8DI_P_2W,
    UR20_8DI_P_3W,
    UR20_8DI_P_3W_HD,
    UR20_16DI_P,
    UR20_16DI_P_PLC_INT,
    UR20_2DI_P_TS,
    UR20_4DI_P_TS,
    UR20_4DI_N,
    UR20_8DI_N_3W,
    UR20_16DI_N,
    UR20_16DI_N_PLC_INT,
    UR20_4DI_2W_230V_AC,

    // Digital output modules
    UR20_4DO_P,
    UR20_4DO_P_2A,
    UR20_4DO_PN_2A,
    UR20_8DO_P,
    UR20_8DO_P_2W_HD,
    UR20_16DO_P,
    UR20_16DO_P_PLC_INT,
    UR20_4DO_N,
    UR20_4DO_N_2A,
    UR20_8DO_N,
    UR20_16DO_N,
    UR20_16DO_N_PLC_INT,
    UR20_4RO_SSR_255,
    UR20_4RO_CO_255,

    // Digital pulse width modulation output modules
    UR20_2PWM_PN_0_5A,
    UR20_2PWM_PN_2A,

    // Analogue input modules
    UR20_4AI_UI_16,
    UR20_4AI_UI_16_DIAG,
    UR20_4AI_UI_DIF_16_DIAG,
    UR20_4AI_UI_16_HD,
    UR20_4AI_UI_16_DIAG_HD,
    UR20_4AI_UI_12,
    UR20_8AI_I_16_HD,
    UR20_8AI_I_16_DIAG_HD,
    UR20_8AI_I_PLC_INT,
    UR20_4AI_R_HS_16_DIAG,
    UR20_2AI_SG_24_DIAG,
    UR20_3EM_230V_AC,

    // Analogue output modul
    UR20_4AO_UI_16,
    UR20_4AO_UI_16_M,
    UR20_4AO_UI_16_DIAG,
    UR20_4AO_UI_16_M_DIAG,
    UR20_4AO_UI_16_HD,
    UR20_4AO_UI_16_DIAG_HD,

    // Digital counter modules
    UR20_1CNT_100_1DO,
    UR20_2CNT_100,
    UR20_1CNT_500,
    UR20_2FCNT_100,

    // Communication modules
    UR20_1SSI,
    UR20_1COM_232_485_422,
    UR20_1COM_SAI_PRO,
    UR20_4COM_IO_LINK,

    // Analogue input modules DIAG
    UR20_4AI_RTD_DIAG,
    UR20_4AI_TC_DIAG,

    // Power feed modules
    UR20_PF_I,
    UR20_PF_O,

    // Safe feed-in modules
    UR20_PF_O_1DI_SIL,
    UR20_PF_O_2DI_SIL,
    UR20_PF_O_2DI_DELAY_SIL,
}

/// Describes how the data should be interpreted.
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum DataFormat {
    /// Siemens S5 format
    S5 = 0,
    /// Siemens S7 format
    S7 = 1,
}

impl DataFormat {
    fn factor(&self) -> f32 {
        f32::from(match *self {
            DataFormat::S5 => S5_FACTOR,
            DataFormat::S7 => S7_FACTOR,
        })
    }
}

/// Analog input or output range (current and voltage).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum AnalogUIRange {
    /// 0mA ... 20mA
    mA0To20 = 0,
    /// 4mA ... 20mA
    mA4To20 = 1,
    /// 0V ... 10V
    V0To10 = 2,
    /// -10V ... 10V
    VMinus10To10 = 3,
    /// 0V ... 5V
    V0To5 = 4,
    /// -5V ... 5V
    VMinus5To5 = 5,
    /// 1V ... 5V
    V1To5 = 6,
    /// 2V ... 10V
    V2To10 = 7,
    /// Disabled channel.
    Disabled = 8,
}

/// Analog input or output range (current only).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum AnalogIRange {
    /// 0mA ... 20mA
    mA0To20 = 0,
    /// 4mA ... 20mA
    mA4To20 = 1,
    /// Disabled channel.
    Disabled = 2,
}

/// Resistor value range.
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum RtdRange {
    /// -200 ... 850 Degree Celsius
    PT100 = 0,
    /// -200 ... 850 Degree Celsius
    PT200 = 1,
    /// -200 ... 850 Degree Celsius
    PT500 = 2,
    /// -200 ... 850 Degree Celsius
    PT1000 = 3,
    /// -60 ... 250 Degree Celsius
    NI100 = 4,
    /// -80 ... 260 Degree Celsius
    NI120 = 5,
    /// -60 ... 250 Degree Celsius
    NI200 = 6,
    /// -60 ... 250 Degree Celsius
    NI500 = 7,
    /// -60 ... 250 Degree Celsius
    NI1000 = 8,
    /// -100 ... 260 Degree Celsius
    Cu10 = 9,
    /// Resistance 40 Ω
    R40 = 10,
    /// Resistance 80 Ω
    R80 = 11,
    /// Resistance 150 Ω
    R150 = 12,
    /// Resistance 300 Ω
    R300 = 13,
    /// Resistance 500 Ω
    R500 = 14,
    /// Resistance 1000 Ω
    R1000 = 15,
    /// Resistance 2000 Ω
    R2000 = 16,
    /// Resistance 4000 Ω
    R4000 = 17,
    /// Disabled
    Disabled = 18,
}

/// The unit a temperature value is represented in.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum TemperatureUnit {
    Celsius    = 0,
    Fahrenheit = 1,
    Kelvin     = 2,
}

/// Describes how the resistor is physically conneted.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ConnectionType {
    TwoWire   = 0,
    ThreeWire = 1,
    FourWire  = 2,
}

/// Time to convert a signal.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ConversionTime {
    ms240 = 0,
    ms130 = 1,
    ms80  = 2,
    ms55  = 3,
    ms43  = 4,
    ms36  = 5,
}

/// Filter signals by defining a minimal duration.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum InputFilter {
    us5    = 0,
    us11   = 1,
    us21   = 2,
    us43   = 3,
    us83   = 4,
    us167  = 5,
    us333  = 6,
    us667  = 7,
    ms1    = 8,
    ms3    = 9,
    ms5    = 10,
    ms11   = 11,
    ms22   = 12,
    ms43   = 13,
    ms91   = 14,
    ms167  = 15,
    ms333  = 16,
}

/// Time to delay a signal.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum InputDelay {
    no    = 0,
    us300 = 1, // not at PROFIBUS-DP
    ms3   = 2,
    ms10  = 3,
    ms20  = 4,
    ms40  = 5, // not at PROFIBUS-DP
}

/// Frequency suppression.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum FrequencySuppression {
    Disabled  = 0,
    Hz50      = 1,
    Hz60      = 2,
    Average16 = 3, // Average over 16 values
}

impl ModuleType {
    pub fn try_from_u32(id: u32) -> Result<Self> {
        use crate::ModuleType::*;

        let t = match id {
            0x0009_1F84 => UR20_4DI_P,
            0x001B_1F84 => UR20_4DI_P_3W,
            0x0013_1FC1 => UR20_8DI_P_2W,
            0x000A_1FC1 => UR20_8DI_P_3W,
            0x0003_1FC1 => UR20_8DI_P_3W_HD,
            0x0004_9FC2 => UR20_16DI_P,
            0x0005_9FC2 => UR20_16DI_P_PLC_INT,
            0x0F01_4700 => UR20_2DI_P_TS,
            0x0F02_4700 => UR20_4DI_P_TS,
            0x0001_1F84 => UR20_4DI_N,
            0x0002_1FC1 => UR20_8DI_N_3W,
            0x000C_9FC2 => UR20_16DI_N,
            0x000D_9FC2 => UR20_16DI_N_PLC_INT,
            0x0016_9F84 => UR20_4DI_2W_230V_AC,

            0x0101_2FA0 => UR20_4DO_P,
            0x0105_2FA0 => UR20_4DO_P_2A,
            0x0115_2FC8 => UR20_4DO_PN_2A,
            0x0102_2FC8 => UR20_8DO_P,
            0x0119_2FC8 => UR20_8DO_P_2W_HD,
            0x0103_AFD0 => UR20_16DO_P,
            0x0104_AFD0 => UR20_16DO_P_PLC_INT,
            0x010A_2FA0 => UR20_4DO_N,
            0x010B_2FA0 => UR20_4DO_N_2A,
            0x010C_2FC8 => UR20_8DO_N,
            0x010D_AFD0 => UR20_16DO_N,
            0x010E_AFD0 => UR20_16DO_N_PLC_INT,
            0x0107_2FA0 => UR20_4RO_SSR_255,
            0x0106_2FA0 => UR20_4RO_CO_255,

            0x0908_4880 => UR20_2PWM_PN_0_5A,
            0x0909_4880 => UR20_2PWM_PN_2A,

            0x0401_15C4 => UR20_4AI_UI_16,
            0x0402_1544 => UR20_4AI_UI_16_DIAG,
            0x041E_1544 => UR20_4AI_UI_DIF_16_DIAG,
            0x0413_15C4 => UR20_4AI_UI_16_HD,
            0x0414_1544 => UR20_4AI_UI_16_DIAG_HD,
            0x0411_15C4 => UR20_4AI_UI_12,
            0x0404_15C5 => UR20_8AI_I_16_HD,
            0x0405_1545 => UR20_8AI_I_16_DIAG_HD,
            0x0409_15C5 => UR20_8AI_I_PLC_INT,
            0x041C_1544 => UR20_4AI_R_HS_16_DIAG,
            0x041B_356D => UR20_2AI_SG_24_DIAG,
            0x0418_356D => UR20_3EM_230V_AC,

            0x0502_25E0 => UR20_4AO_UI_16,
            0x0506_25E0 => UR20_4AO_UI_16_M,
            0x0501_2560 => UR20_4AO_UI_16_DIAG,
            0x0505_2560 => UR20_4AO_UI_16_M_DIAG,
            0x0504_25E0 => UR20_4AO_UI_16_HD,
            0x0503_2560 => UR20_4AO_UI_16_DIAG_HD,

            0x08C1_3800 => UR20_1CNT_100_1DO,
            0x08C3_3800 => UR20_2CNT_100,
            0x08C4_3801 => UR20_1CNT_500,
            0x0881_28EE => UR20_2FCNT_100,

            0x09C1_7880 => UR20_1SSI,
            0x0E41_3FED => UR20_1COM_232_485_422,
            0x0BC1_E800 => UR20_1COM_SAI_PRO,
            0x0E81_276D => UR20_4COM_IO_LINK,

            0x0406_1544 => UR20_4AI_RTD_DIAG,
            0x0407_1544 => UR20_4AI_TC_DIAG,

            0x1801_9F43 => UR20_PF_O_1DI_SIL,
            0x1803_9F43 => UR20_PF_O_2DI_SIL,
            0x1802_9F43 => UR20_PF_O_2DI_DELAY_SIL,

            _ => {
                return Err(Error::UnknownModule);
            }
        };
        Ok(t)
    }

    /// Returns the number of channels for a specific module type.
    #[rustfmt::skip]
    pub fn channel_count(&self) -> usize {
        use crate::ModuleType::*;

        match *self {

            UR20_PF_I               |
            UR20_PF_O               |
            UR20_PF_O_1DI_SIL       |
            UR20_PF_O_2DI_SIL       |
            UR20_PF_O_2DI_DELAY_SIL => 0,

            UR20_1CNT_100_1DO       |
            UR20_1CNT_500           |
            UR20_1COM_232_485_422   |
            UR20_1SSI               |
            UR20_1COM_SAI_PRO       => 1,

            UR20_2DI_P_TS           |
            UR20_2PWM_PN_0_5A       |
            UR20_2PWM_PN_2A         |
            UR20_2AI_SG_24_DIAG     |
            UR20_2CNT_100           |
            UR20_2FCNT_100          => 2,

            UR20_4DI_P              |
            UR20_4DI_P_3W           |
            UR20_4DI_P_TS           |
            UR20_4DI_N              |
            UR20_4DI_2W_230V_AC     |
            UR20_4DO_P              |
            UR20_4DO_P_2A           |
            UR20_4DO_PN_2A          |
            UR20_4DO_N              |
            UR20_4DO_N_2A           |
            UR20_4RO_SSR_255        |
            UR20_4RO_CO_255         |
            UR20_4AI_UI_16          |
            UR20_4AI_UI_16_DIAG     |
            UR20_4AI_UI_DIF_16_DIAG |
            UR20_4AI_UI_16_HD       |
            UR20_4AI_UI_16_DIAG_HD  |
            UR20_4AI_UI_12          |
            UR20_4AI_R_HS_16_DIAG   |
            UR20_4AO_UI_16          |
            UR20_4AO_UI_16_M        |
            UR20_4AO_UI_16_DIAG     |
            UR20_4AO_UI_16_M_DIAG   |
            UR20_4AO_UI_16_HD       |
            UR20_4AO_UI_16_DIAG_HD  |
            UR20_4COM_IO_LINK       |
            UR20_4AI_RTD_DIAG       |
            UR20_4AI_TC_DIAG        => 4,

            UR20_8DI_P_2W           |
            UR20_8DI_P_3W           |
            UR20_8DI_P_3W_HD        |
            UR20_8DI_N_3W           |
            UR20_8DO_P              |
            UR20_8DO_P_2W_HD        |
            UR20_8DO_N              |
            UR20_8AI_I_16_HD        |
            UR20_8AI_I_16_DIAG_HD   |
            UR20_8AI_I_PLC_INT      |
            UR20_3EM_230V_AC        => 8,

            UR20_16DI_P             |
            UR20_16DI_P_PLC_INT     |
            UR20_16DI_N             |
            UR20_16DI_N_PLC_INT     |
            UR20_16DO_P             |
            UR20_16DO_P_PLC_INT     |
            UR20_16DO_N             |
            UR20_16DO_N_PLC_INT     => 16,

        }
    }
}

#[rustfmt::skip]
impl FromStr for ModuleType {
    type Err = Error;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        use crate::ModuleType::*;
        let t = match &*s.to_uppercase().replace("-","_") {
            "UR20_4DI_P"               => UR20_4DI_P,
            "UR20_4DI_P_3W"            => UR20_4DI_P_3W,
            "UR20_8DI_P_2W"            => UR20_8DI_P_2W,
            "UR20_8DI_P_3W"            => UR20_8DI_P_3W,
            "UR20_8DI_P_3W_HD"         => UR20_8DI_P_3W_HD,
            "UR20_16DI_P"              => UR20_16DI_P,
            "UR20_16DI_P_PLC_INT"      => UR20_16DI_P_PLC_INT,
            "UR20_2DI_P_TS"            => UR20_2DI_P_TS,
            "UR20_4DI_P_TS"            => UR20_4DI_P_TS,
            "UR20_4DI_N"               => UR20_4DI_N,
            "UR20_8DI_N_3W"            => UR20_8DI_N_3W,
            "UR20_16DI_N"              => UR20_16DI_N,
            "UR20_16DI_N_PLC_INT"      => UR20_16DI_N_PLC_INT,
            "UR20_4DI_2W_230V_AC"      => UR20_4DI_2W_230V_AC,

            "UR20_4DO_P"               => UR20_4DO_P,
            "UR20_4DO_P_2A"            => UR20_4DO_P_2A,
            "UR20_4DO_PN_2A"           => UR20_4DO_PN_2A,
            "UR20_8DO_P"               => UR20_8DO_P,
            "UR20_8DO_P_2W_HD"         => UR20_8DO_P_2W_HD,
            "UR20_16DO_P"              => UR20_16DO_P,
            "UR20_16DO_P_PLC_INT"      => UR20_16DO_P_PLC_INT,
            "UR20_4DO_N"               => UR20_4DO_N,
            "UR20_4DO_N_2A"            => UR20_4DO_N_2A,
            "UR20_8DO_N"               => UR20_8DO_N,
            "UR20_16DO_N"              => UR20_16DO_N,
            "UR20_16DO_N_PLC_INT"      => UR20_16DO_N_PLC_INT,
            "UR20_4RO_SSR_255"         => UR20_4RO_SSR_255,
            "UR20_4RO_CO_255"          => UR20_4RO_CO_255,

            "UR20_2PWM_PN_0_5A"        => UR20_2PWM_PN_0_5A,
            "UR20_2PWM_PN_2A"          => UR20_2PWM_PN_2A,

            "UR20_4AI_UI_16"           => UR20_4AI_UI_16,
            "UR20_4AI_UI_16_DIAG"      => UR20_4AI_UI_16_DIAG,
            "UR20_4AI_UI_DIF_16_DIAG"  => UR20_4AI_UI_DIF_16_DIAG,
            "UR20_4AI_UI_16_HD"        => UR20_4AI_UI_16_HD,
            "UR20_4AI_UI_16_DIAG_HD"   => UR20_4AI_UI_16_DIAG_HD,
            "UR20_4AI_UI_12"           => UR20_4AI_UI_12,
            "UR20_8AI_I_16_HD"         => UR20_8AI_I_16_HD,
            "UR20_8AI_I_16_DIAG_HD"    => UR20_8AI_I_16_DIAG_HD,
            "UR20_8AI_I_PLC_INT"       => UR20_8AI_I_PLC_INT,
            "UR20_4AI_R_HS_16_DIAG"    => UR20_4AI_R_HS_16_DIAG,
            "UR20_2AI_SG_24_DIAG"      => UR20_2AI_SG_24_DIAG,
            "UR20_3EM_230V_AC"         => UR20_3EM_230V_AC,

            "UR20_4AO_UI_16"           => UR20_4AO_UI_16,
            "UR20_4AO_UI_16_M"         => UR20_4AO_UI_16_M,
            "UR20_4AO_UI_16_DIAG"      => UR20_4AO_UI_16_DIAG,
            "UR20_4AO_UI_16_M_DIAG"    => UR20_4AO_UI_16_M_DIAG,
            "UR20_4AO_UI_16_HD"        => UR20_4AO_UI_16_HD,
            "UR20_4AO_UI_16_DIAG_HD"   => UR20_4AO_UI_16_DIAG_HD,

            "UR20_1CNT_100_1DO"        => UR20_1CNT_100_1DO,
            "UR20_2CNT_100"            => UR20_2CNT_100,
            "UR20_1CNT_500"            => UR20_1CNT_500,
            "UR20_2FCNT_100"           => UR20_2FCNT_100,

            "UR20_1SSI"                => UR20_1SSI,
            "UR20_1COM_232_485_422"    => UR20_1COM_232_485_422,
            "UR20_1COM_SAI_PRO"        => UR20_1COM_SAI_PRO,
            "UR20_4COM_IO_LINK"        => UR20_4COM_IO_LINK,

            "UR20_4AI_RTD_DIAG"        => UR20_4AI_RTD_DIAG,
            "UR20_4AI_TC_DIAG"         => UR20_4AI_TC_DIAG,

            "UR20_PF_I"                => UR20_PF_I,
            "UR20_PF_O"                => UR20_PF_O,

            "UR20_PF_O_1DI_SIL"        => UR20_PF_O_1DI_SIL,
            "UR20_PF_O_2DI_SIL"        => UR20_PF_O_2DI_SIL,
            "UR20_PF_O_2DI_DELAY_SIL"  => UR20_PF_O_2DI_DELAY_SIL,

            _ => {
                return Err(Error::UnknownModule);
            }
        };
        Ok(t)
    }
}

#[rustfmt::skip]
impl FromStr for ModuleCategory {
    type Err = Error;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        use crate::ModuleCategory::*;
        let c = match &*s.to_uppercase() {
            "DI"  => DI,
            "DO"  => DO,
            "AI"  => AI,
            "AO"  => AO,
            "CNT" => CNT,
            "PWM" => PWM,
            "RTD" => RTD,
            "TC"  => TC,
            "COM" => COM,
            "RO"  => RO,
            _ => {
                return Err(Error::UnknownCategory);
            }
        };
        Ok(c)
    }
}

#[rustfmt::skip]
impl From<ModuleType> for ModuleCategory {
    fn from(val: ModuleType) -> Self {
        use crate::ModuleType::*;
        use crate::ModuleCategory::*;
        match val {
            UR20_4DI_P              |
            UR20_4DI_P_3W           |
            UR20_8DI_P_2W           |
            UR20_8DI_P_3W           |
            UR20_8DI_P_3W_HD        |
            UR20_16DI_P             |
            UR20_16DI_P_PLC_INT     |
            UR20_2DI_P_TS           |
            UR20_4DI_P_TS           |
            UR20_4DI_N              |
            UR20_8DI_N_3W           |
            UR20_16DI_N             |
            UR20_16DI_N_PLC_INT     |
            UR20_4DI_2W_230V_AC     => DI,

            UR20_4DO_P              |
            UR20_4DO_P_2A           |
            UR20_4DO_PN_2A          |
            UR20_8DO_P              |
            UR20_8DO_P_2W_HD        |
            UR20_16DO_P             |
            UR20_16DO_P_PLC_INT     |
            UR20_4DO_N              |
            UR20_4DO_N_2A           |
            UR20_8DO_N              |
            UR20_16DO_N             |
            UR20_16DO_N_PLC_INT     |
            UR20_4RO_SSR_255        |
            UR20_4RO_CO_255         => DO,

            UR20_2PWM_PN_0_5A       |
            UR20_2PWM_PN_2A         => PWM,

            UR20_4AI_UI_16          |
            UR20_4AI_UI_16_DIAG     |
            UR20_4AI_UI_DIF_16_DIAG |
            UR20_4AI_UI_16_HD       |
            UR20_4AI_UI_16_DIAG_HD  |
            UR20_4AI_UI_12          |
            UR20_8AI_I_16_HD        |
            UR20_8AI_I_16_DIAG_HD   |
            UR20_8AI_I_PLC_INT      |
            UR20_4AI_R_HS_16_DIAG   |
            UR20_2AI_SG_24_DIAG     |
            UR20_3EM_230V_AC        => AI,

            UR20_4AO_UI_16          |
            UR20_4AO_UI_16_M        |
            UR20_4AO_UI_16_DIAG     |
            UR20_4AO_UI_16_M_DIAG   |
            UR20_4AO_UI_16_HD       |
            UR20_4AO_UI_16_DIAG_HD  => AO,

            UR20_1CNT_100_1DO       |
            UR20_2CNT_100           |
            UR20_1CNT_500           |
            UR20_2FCNT_100          => CNT,

            UR20_1SSI               |
            UR20_1COM_232_485_422   |
            UR20_1COM_SAI_PRO       |
            UR20_4COM_IO_LINK       => COM,

            UR20_4AI_RTD_DIAG       => RTD,
            UR20_4AI_TC_DIAG        => TC,

            UR20_PF_I               |
            UR20_PF_O               |
            UR20_PF_O_1DI_SIL       |
            UR20_PF_O_2DI_SIL       |
            UR20_PF_O_2DI_DELAY_SIL => PF,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn module_by_u32_id() {
        assert_eq!(
            ModuleType::try_from_u32(0x05052560).unwrap(),
            ModuleType::UR20_4AO_UI_16_M_DIAG
        );
        assert_eq!(
            ModuleType::try_from_u32(0x01234567).err().unwrap(),
            Error::UnknownModule
        );
    }

    #[test]
    fn module_by_str_id() {
        assert_eq!(
            ModuleType::from_str("UR20_1COM_232_485_422").unwrap(),
            ModuleType::UR20_1COM_232_485_422
        );
        assert_eq!(
            "UR20-1COM-232-485-422".parse::<ModuleType>().unwrap(),
            ModuleType::UR20_1COM_232_485_422
        );
        assert_eq!(
            "ur20-1com-232-485-422".parse::<ModuleType>().unwrap(),
            ModuleType::UR20_1COM_232_485_422
        );
        assert_eq!(
            "not-valid".parse::<ModuleType>().err().unwrap(),
            Error::UnknownModule
        );
    }

    #[test]
    fn category_by_str_id() {
        assert_eq!(
            ModuleCategory::from_str("RTD").unwrap(),
            ModuleCategory::RTD
        );
        assert_eq!(
            ModuleCategory::from_str("rtd").unwrap(),
            ModuleCategory::RTD
        );
        assert_eq!(ModuleCategory::from_str("aO").unwrap(), ModuleCategory::AO);
    }
}
