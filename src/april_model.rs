use crate::april_session::AprilSession;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};

pub struct AprilModel {
    ptr: april_asr_rs_sys::AprilASRModel,
}

macro_rules! null_ptr_error {
    ($ptr: expr) => {
        if $ptr.is_null() {
            return Err(Error::NullPtr);
        }
    };
}

impl AprilModel {
    pub fn new(path: impl Into<Vec<u8>>) -> Result<Self> {
        Self::_new(CString::new(path)?)
    }

    fn _new(path: CString) -> Result<Self> {
        let res = unsafe { april_asr_rs_sys::aam_create_model(path.as_ptr()) };
        null_ptr_error!(res);

        Ok(Self { ptr: res })
    }

    pub fn get_model_name(&self) -> Result<&str> {
        let name_ptr = unsafe { april_asr_rs_sys::aam_get_name(self.ptr) };
        null_ptr_error!(name_ptr);

        Ok(unsafe { CStr::from_ptr(name_ptr) }.to_str()?)
    }

    pub fn get_model_description(&self) -> Result<&str> {
        let name_ptr = unsafe { april_asr_rs_sys::aam_get_description(self.ptr) };
        null_ptr_error!(name_ptr);

        Ok(unsafe { CStr::from_ptr(name_ptr) }.to_str()?)
    }

    pub fn get_model_language(&self) -> Result<&str> {
        let name_ptr = unsafe { april_asr_rs_sys::aam_get_language(self.ptr) };
        null_ptr_error!(name_ptr);

        Ok(unsafe { CStr::from_ptr(name_ptr) }.to_str()?)
    }

    pub fn get_sample_rate(&self) -> usize {
        unsafe { april_asr_rs_sys::aam_get_sample_rate(self.ptr) }
    }

    pub fn create_session<'this>(&'this self, config: AprilConfig) -> AprilSession<'this> {
        unsafe { april_asr_rs_sys::aas_create_session(self.ptr, config) }
    }
}

impl Drop for AprilModel {
    fn drop(&mut self) {
        unsafe { april_asr_rs_sys::aam_free(self.ptr) }
    }
}