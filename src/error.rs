use std::error::Error;
use std::fmt;

use winapi::um::dinput::{
    DIERR_ACQUIRED, DIERR_ALREADYINITIALIZED, DIERR_BADDRIVERVER, DIERR_BETADIRECTINPUTVERSION,
    DIERR_DEVICEFULL, DIERR_DEVICENOTREG, DIERR_EFFECTPLAYING, DIERR_HASEFFECTS,
    DIERR_INCOMPLETEEFFECT, DIERR_INPUTLOST, DIERR_INSUFFICIENTPRIVS, DIERR_INVALIDPARAM,
    DIERR_MAPFILEFAIL, DIERR_MOREDATA, DIERR_NOAGGREGATION, DIERR_NOINTERFACE, DIERR_NOTACQUIRED,
    DIERR_NOTBUFFERED, DIERR_NOTDOWNLOADED, DIERR_NOTEXCLUSIVEACQUIRED, DIERR_NOTINITIALIZED,
    DIERR_OBJECTNOTFOUND, DIERR_OLDDIRECTINPUTVERSION, DIERR_OUTOFMEMORY, DIERR_REPORTFULL,
    DIERR_UNPLUGGED, DIERR_UNSUPPORTED,
};
use windows::core::HRESULT;
use windows::Win32::Devices::HumanInterfaceDevice::{
    DI_DOWNLOADSKIPPED, DI_EFFECTRESTARTED, DI_POLLEDDEVICE, DI_SETTINGSNOTSAVED, DI_TRUNCATED,
    DI_TRUNCATEDANDRESTARTED, DI_WRITEPROTECT,
};
use windows::Win32::Foundation::{GetLastError, E_FAIL, S_OK};
use windows::Win32::System::Com::Urlmon::E_PENDING;

pub type Result<T, E = DirectInputError> = std::result::Result<T, E>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectInputStatus {
    BufferOverflow,
    DownloadSkipped,
    EffectRestarted,
    NoEffect,
    NotAttached,
    Ok,
    PolledDevice,
    PropNoEffect,
    SettingsNotSaved,
    Truncated,
    TruncatedAndRestarted,
    WriteProtect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectInputError {
    Acquired,
    AlreadyInitialized,
    BadDriverVersion,
    BetaDirectInputVersion,
    DeviceFull,
    DeviceNotReg,
    EffectPlaying,
    Generic,
    Handle,
    HandleExists,
    HasEffects,
    IncompleteEffect,
    InputLost,
    InvalidParam,
    InsufficientPrivs,
    MapFileFail,
    MoreData,
    NoAggregation,
    NoInterface,
    NotAcquired,
    NotBuffered,
    NotDownloaded,
    NotExclusiveAcquired,
    NotInitialized,
    ObjectNotFound,
    OldDirectInputVersion,
    OtherAppHasPrio,
    OutOfMemory,
    Pending,
    ReadOnly,
    ReportFull,
    Unplugged,
    Unsupported,
    Unknown(HRESULT),
}

impl DirectInputStatus {
    pub(crate) fn from_hresult(hr: HRESULT) -> Option<Self> {
        // This match only contains status values that have a unique `HRESULT` value (e.g.
        // `DI_BUFFEROVERFLOW` and `DI_NOTATTACHED` both use `S_FALSE` so they are not matched
        // here).
        match hr {
            DI_DOWNLOADSKIPPED => Some(Self::DownloadSkipped),
            DI_EFFECTRESTARTED => Some(Self::EffectRestarted),
            DI_POLLEDDEVICE => Some(Self::PolledDevice),
            DI_SETTINGSNOTSAVED => Some(Self::SettingsNotSaved),
            DI_TRUNCATED => Some(Self::Truncated),
            DI_TRUNCATEDANDRESTARTED => Some(Self::TruncatedAndRestarted),
            DI_WRITEPROTECT => Some(Self::WriteProtect),
            S_OK => Some(Self::Ok),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn from_hresult_or_ok(hr: HRESULT) -> Self {
        Self::from_hresult(hr).unwrap_or(Self::Ok)
    }
}

impl DirectInputError {
    #[inline]
    pub(crate) fn from_hresult(hr: HRESULT) -> Self {
        Self::hresult_matches(hr).unwrap_or(Self::Unknown(hr))
    }

    pub(crate) fn from_last_error() -> Self {
        let err = unsafe { GetLastError() };

        Self::from_hresult(HRESULT::from(err))
    }

    pub(crate) fn hresult_matches(hr: HRESULT) -> Option<Self> {
        match hr {
            E_FAIL => return Some(Self::Generic),
            E_PENDING => return Some(Self::Pending),
            _ => {}
        };
        match hr.0 as i32 {
            DIERR_ACQUIRED => Some(Self::Acquired),
            DIERR_ALREADYINITIALIZED => Some(Self::AlreadyInitialized),
            DIERR_BADDRIVERVER => Some(Self::BadDriverVersion),
            DIERR_BETADIRECTINPUTVERSION => Some(Self::BetaDirectInputVersion),
            DIERR_DEVICEFULL => Some(Self::DeviceFull),
            DIERR_DEVICENOTREG => Some(Self::DeviceNotReg),
            DIERR_EFFECTPLAYING => Some(Self::EffectPlaying),
            //DIERR_GENERIC => Some(Self::Generic),
            //DIERR_HANDLEEXISTS => Some(Self::HandleExists),
            DIERR_HASEFFECTS => Some(Self::HasEffects),
            DIERR_INCOMPLETEEFFECT => Some(Self::IncompleteEffect),
            DIERR_INPUTLOST => Some(Self::InputLost),
            DIERR_INVALIDPARAM => Some(Self::InvalidParam),
            DIERR_INSUFFICIENTPRIVS => Some(Self::InsufficientPrivs),
            DIERR_MAPFILEFAIL => Some(Self::MapFileFail),
            DIERR_MOREDATA => Some(Self::MoreData),
            DIERR_NOAGGREGATION => Some(Self::NoAggregation),
            DIERR_NOINTERFACE => Some(Self::NoInterface),
            DIERR_NOTACQUIRED => Some(Self::NotAcquired),
            DIERR_NOTBUFFERED => Some(Self::NotBuffered),
            DIERR_NOTDOWNLOADED => Some(Self::NotDownloaded),
            DIERR_NOTEXCLUSIVEACQUIRED => Some(Self::NotExclusiveAcquired),
            DIERR_NOTINITIALIZED => Some(Self::NotInitialized),
            DIERR_OBJECTNOTFOUND => Some(Self::ObjectNotFound),
            DIERR_OLDDIRECTINPUTVERSION => Some(Self::OldDirectInputVersion),
            //DIERR_OTHERAPPHASPRIO => Some(Self::OtherAppHasPrio),
            DIERR_OUTOFMEMORY => Some(Self::OutOfMemory),
            //DIERR_READONLY => Some(Self::ReadOnly),
            DIERR_REPORTFULL => Some(Self::ReportFull),
            DIERR_UNPLUGGED => Some(Self::Unplugged),
            DIERR_UNSUPPORTED => Some(Self::Unsupported),
            //E_PENDING => Some(Self::Pending),
            _ => None,
        }
    }
}

impl fmt::Display for DirectInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unknown(hr) => fmt::Debug::fmt(hr, f),
            _ => fmt::Debug::fmt(self, f),
        }
    }
}

impl Error for DirectInputError {}

impl From<windows::core::Error> for DirectInputError {
    #[inline]
    fn from(value: windows::core::Error) -> Self {
        Self::from_hresult(value.code())
    }
}
