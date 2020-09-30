use std::error::Error;
use std::fmt;
use std::io;

use winapi::shared::winerror::HRESULT;
use winapi::um::dinput::{
    DIERR_ACQUIRED, DIERR_ALREADYINITIALIZED, DIERR_BADDRIVERVER, DIERR_BETADIRECTINPUTVERSION,
    DIERR_DEVICEFULL, DIERR_DEVICENOTREG, DIERR_EFFECTPLAYING, DIERR_GENERIC, DIERR_HASEFFECTS,
    DIERR_INCOMPLETEEFFECT, DIERR_INPUTLOST, DIERR_INSUFFICIENTPRIVS, DIERR_INVALIDPARAM,
    DIERR_MAPFILEFAIL, DIERR_MOREDATA, DIERR_NOAGGREGATION, DIERR_NOINTERFACE, DIERR_NOTACQUIRED,
    DIERR_NOTBUFFERED, DIERR_NOTDOWNLOADED, DIERR_NOTEXCLUSIVEACQUIRED, DIERR_NOTINITIALIZED,
    DIERR_OBJECTNOTFOUND, DIERR_OLDDIRECTINPUTVERSION, DIERR_OUTOFMEMORY, DIERR_REPORTFULL,
    DIERR_UNPLUGGED, DIERR_UNSUPPORTED, DI_DOWNLOADSKIPPED, DI_EFFECTRESTARTED, DI_OK,
    DI_POLLEDDEVICE, DI_SETTINGSNOTSAVED, DI_TRUNCATED, DI_TRUNCATEDANDRESTARTED, DI_WRITEPROTECT,
    E_PENDING,
};
/*
use winapi::um::dinput::{
    DIERR_HANDLEEXISTS, DIERR_NOTFOUND, DIERR_NOTINITIALIZED, DIERR_OTHERAPPHASPRIO, DIERR_READONLY,
};
*/

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
    //HandleExists,
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
    //ReadOnly,
    ReportFull,
    Unplugged,
    Unsupported,
}

impl DirectInputStatus {
    pub(crate) fn from_hresult(hr: HRESULT) -> Option<Self> {
        // This match only contains status values that have a unique `HRESULT` value (e.g.
        // `DI_BUFFEROVERFLOW` and `DI_NOTATTACHED` both use `S_FALSE` so they are not matched
        // here).
        match hr {
            DI_DOWNLOADSKIPPED => Some(Self::DownloadSkipped),
            DI_EFFECTRESTARTED => Some(Self::EffectRestarted),
            DI_OK => Some(Self::Ok),
            DI_POLLEDDEVICE => Some(Self::PolledDevice),
            DI_SETTINGSNOTSAVED => Some(Self::SettingsNotSaved),
            DI_TRUNCATED => Some(Self::Truncated),
            DI_TRUNCATEDANDRESTARTED => Some(Self::TruncatedAndRestarted),
            DI_WRITEPROTECT => Some(Self::WriteProtect),
            _ => None,
        }
    }

    pub(crate) fn from_hresult_or_ok(hr: HRESULT) -> Self {
        if let Some(status) = Self::from_hresult(hr) {
            status
        } else {
            DirectInputStatus::Ok
        }
    }
}

impl DirectInputError {
    pub(crate) fn from_hresult(hr: HRESULT) -> io::Error {
        if let Some(error) = Self::hresult_matches(hr) {
            error.to_io_error()
        } else {
            io::Error::from_raw_os_error(hr)
        }
    }

    pub(crate) fn hresult_matches(hr: HRESULT) -> Option<Self> {
        match hr {
            DIERR_ACQUIRED => Some(Self::Acquired),
            DIERR_ALREADYINITIALIZED => Some(Self::AlreadyInitialized),
            DIERR_BADDRIVERVER => Some(Self::BadDriverVersion),
            DIERR_BETADIRECTINPUTVERSION => Some(Self::BetaDirectInputVersion),
            DIERR_DEVICEFULL => Some(Self::DeviceFull),
            DIERR_DEVICENOTREG => Some(Self::DeviceNotReg),
            DIERR_EFFECTPLAYING => Some(Self::EffectPlaying),
            DIERR_GENERIC => Some(Self::Generic),
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
            E_PENDING => Some(Self::Pending),
            _ => None,
        }
    }

    pub(crate) fn to_io_error(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self)
    }
}

impl fmt::Display for DirectInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for DirectInputError {}
