use crate::april_model::AprilModel;
use crate::error::{Error, Result};
use std::marker::PhantomData;

pub struct AprilSession<'a> {
    ptr: april_asr_rs_sys::AprilASRSession,
    model: PhantomData<&'a AprilModel>,
}

impl AprilSession<'_> {
    pub(crate) fn new<'a>(ptr: april_asr_rs_sys::AprilASRSession) -> Result<Self<'a>> {
        if ptr.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(Self {
                ptr,
                model: PhantomData,
            })
        }
    }
}
