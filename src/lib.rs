// Copyright (c) 2017 - 2018 slowtec GmbH <markus.kohlhase@slowtec.de>

use std::str::FromStr;

mod error;

pub(crate) mod util;
pub mod ur20_fbc_mod_tcp;
pub mod ur20_1com_232_485_422;
pub mod ur20_4ao_ui_16;
pub mod ur20_4do_p;
pub mod ur20_4di_p;
pub mod ur20_8ai_i_16_diag_hd;
pub mod ur20_2fcnt_100;
pub mod ur20_4ai_rtd_diag;

pub use error::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelValue {
    Bit(bool),
    Decimal32(f32),
    Bytes(Vec<u8>),
    Disabled,
    None,
}

pub trait Module : std::fmt::Debug {
    /// Number of bytes within the process input data buffer.
    fn process_input_byte_count(&self) -> usize;
    /// Transform raw module input data into a list of channel values.
    fn process_input(&mut self, &[u16]) -> Result<Vec<ChannelValue>, Error>;
}

#[derive(Debug, Clone, PartialEq)]
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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum DataFormat {
    /// Siemens S5 format
    S5 = 0,
    /// Siemens S7 format
    S7 = 1
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnalogUIRange {
    mA0To20       = 0,
    mA4To20       = 1,
    V0To10        = 2,
    VMinus10To10  = 3,
    V0To5         = 4,
    VMinus5To5    = 5,
    V1To5         = 6,
    V2To10        = 7,
    Disabled      = 8,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnalogIRange {
    mA0To20  = 0,
    mA4To20  = 1,
    Disabled = 3,
}

#[derive(Debug, Clone, PartialEq)]
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
    Disabled = 18
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemperatureUnit {
    Celsius    = 0,
    Fahrenheit = 1,
    Kelvin     = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    TwoWire   = 0,
    ThreeWire = 1,
    FourWire  = 2,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionTime {
    ms240 = 0,
    ms130 = 1,
    ms80  = 2,
    ms55  = 3,
    ms43  = 4,
    ms36  = 5,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum InputFilter {
    us5     = 0,
    us11    = 1,
    us21    = 2,
    us43    = 3,
    us83    = 4,
    us167   = 5,
    us333   = 6,
    us667   = 7,
    ms1     = 8,
    ms3     = 9,
    ms5     = 10,
    ms11    = 11,
    ms22    = 12,
    ms43    = 13,
    ms91    = 14,
    ms167   = 15,
    ms333   = 16,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum InputDelay {
    no    = 0,
    us300 = 1, // not at PROFIBUS-DP
    ms3   = 2,
    ms10  = 3,
    ms20  = 4,
    ms40  = 5, // not at PROFIBUS-DP
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum FrequencySuppression {
    Disabled  = 0,
    Hz50      = 1,
    Hz60      = 2,
    Average16 = 3, // Average over 16 values
}

impl ModuleType {
    pub fn try_from_u32(id: u32) -> Result<Self, Error> {

        use ModuleType::*;

        let t = match id {

            0x00091F84 => UR20_4DI_P,
            0x001B1F84 => UR20_4DI_P_3W,
            0x00131FC1 => UR20_8DI_P_2W,
            0x000A1FC1 => UR20_8DI_P_3W,
            0x00031FC1 => UR20_8DI_P_3W_HD,
            0x00049FC2 => UR20_16DI_P,
            0x00059FC2 => UR20_16DI_P_PLC_INT,
            0x0F014700 => UR20_2DI_P_TS,
            0x0F024700 => UR20_4DI_P_TS,
            0x00011F84 => UR20_4DI_N,
            0x00021FC1 => UR20_8DI_N_3W,
            0x000C9FC2 => UR20_16DI_N,
            0x000D9FC2 => UR20_16DI_N_PLC_INT,
            0x00169F84 => UR20_4DI_2W_230V_AC,

            0x01012FA0 => UR20_4DO_P,
            0x01052FA0 => UR20_4DO_P_2A,
            0x01152FC8 => UR20_4DO_PN_2A,
            0x01022FC8 => UR20_8DO_P,
            0x01192FC8 => UR20_8DO_P_2W_HD,
            0x0103AFD0 => UR20_16DO_P,
            0x0104AFD0 => UR20_16DO_P_PLC_INT,
            0x010A2FA0 => UR20_4DO_N,
            0x010B2FA0 => UR20_4DO_N_2A,
            0x010C2FC8 => UR20_8DO_N,
            0x010DAFD0 => UR20_16DO_N,
            0x010EAFD0 => UR20_16DO_N_PLC_INT,
            0x01072FA0 => UR20_4RO_SSR_255,
            0x01062FA0 => UR20_4RO_CO_255,

            0x09084880 => UR20_2PWM_PN_0_5A,
            0x09094880 => UR20_2PWM_PN_2A,

            0x040115C4 => UR20_4AI_UI_16,
            0x04021544 => UR20_4AI_UI_16_DIAG,
            0x041E1544 => UR20_4AI_UI_DIF_16_DIAG,
            0x041315C4 => UR20_4AI_UI_16_HD,
            0x04141544 => UR20_4AI_UI_16_DIAG_HD,
            0x041115C4 => UR20_4AI_UI_12,
            0x040415C5 => UR20_8AI_I_16_HD,
            0x04051545 => UR20_8AI_I_16_DIAG_HD,
            0x040915C5 => UR20_8AI_I_PLC_INT,
            0x041C1544 => UR20_4AI_R_HS_16_DIAG,
            0x041B356D => UR20_2AI_SG_24_DIAG,
            0x0418356D => UR20_3EM_230V_AC,

            0x050225E0 => UR20_4AO_UI_16,
            0x050625E0 => UR20_4AO_UI_16_M,
            0x05012560 => UR20_4AO_UI_16_DIAG,
            0x05052560 => UR20_4AO_UI_16_M_DIAG,
            0x050425E0 => UR20_4AO_UI_16_HD,
            0x05032560 => UR20_4AO_UI_16_DIAG_HD,

            0x08C13800 => UR20_1CNT_100_1DO,
            0x08C33800 => UR20_2CNT_100,
            0x08C43801 => UR20_1CNT_500,
            0x088128EE => UR20_2FCNT_100,

            0x09C17880 => UR20_1SSI,
            0x0E413FED => UR20_1COM_232_485_422,
            0x0BC1E800 => UR20_1COM_SAI_PRO,
            0x0E81276D => UR20_4COM_IO_LINK,

            0x04061544 => UR20_4AI_RTD_DIAG,
            0x04071544 => UR20_4AI_TC_DIAG,

            0x18019F43 => UR20_PF_O_1DI_SIL,
            0x18039F43 => UR20_PF_O_2DI_SIL,
            0x18029F43 => UR20_PF_O_2DI_DELAY_SIL,

            _ => {
                return Err(Error::UnknownModule);
            }
        };
        Ok(t)
    }
}

impl FromStr for ModuleType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ModuleType::*;
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

impl FromStr for ModuleCategory {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ModuleCategory::*;
        let c = match &*s.to_uppercase() {
            "DI"    => DI,
            "DO"    => DO,
            "AI"    => AI,
            "AO"    => AO,
            "CNT"   => CNT,
            "PWM"   => PWM,
            "RTD"   => RTD,
            "TC"    => TC,
            "COM"   => COM,
            "RO"    => RO,
            _ => {
                return Err(Error::UnknownCategory);
            }
        };
        Ok(c)
    }
}

impl Into<ModuleCategory> for ModuleType {

    fn into(self) -> ModuleCategory {
        use ModuleType::*;
        use ModuleCategory::*;
        match self {
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

pub fn channel_count_from_module_type(t: &ModuleType) -> usize {

    use ModuleType::*;

    match *t {

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

pub fn module_list_from_registers(registers: &[u16]) -> Result<Vec<ModuleType>, Error> {
    if registers.len() == 0 || registers.len() % 2 != 0 {
        return Err(Error::RegisterCount);
    }
    let mut list = vec![];
    for i in 0..registers.len() / 2 {
        let idx = i as usize;
        let hi = registers[idx * 2] as u32;
        let lo = registers[idx * 2 + 1] as u32;
        let id = (hi << 16) + lo;
        let m = ModuleType::try_from_u32(id)?;
        list.push(m);
    }
    Ok(list)
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

    #[test]
    fn test_module_list_from_registers() {
        assert_eq!(
            module_list_from_registers(&vec![]).err().unwrap(),
            Error::RegisterCount
        );
        assert_eq!(
            module_list_from_registers(&vec![0xAB0C]).err().unwrap(),
            Error::RegisterCount
        );
        assert_eq!(
            module_list_from_registers(&vec![0x0101, 0x2FA0]).unwrap(),
            vec![ModuleType::UR20_4DO_P]
        );
    }
}
