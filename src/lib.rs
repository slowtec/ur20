// Copyright (c) 2017 slowtec GmbH <markus.kohlhase@slowtec.de>

use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownModule,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnknownModule => write!(f, "unknown module type"),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnknownModule => "unknown module type",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleCategory {
    DI,
    DO,
    AI,
    AO,
    CNT,
    PWM,
    RTD,
    TC,
    COM,
    RO,
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

    // Safe feed_in modules
    UR20_PF_O_1DI_SIL,
    UR20_PF_O_2DI_SIL,
    UR20_PF_O_2DI_DELAY_SIL,
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
}
