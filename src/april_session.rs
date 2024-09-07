use crate::april_model::AprilModel;
use crate::error::{Error, Result};
use std::marker::PhantomData;

pub struct AprilSession<'a> {
    ptr: april_asr_rs_sys::AprilASRSession,
    model: PhantomData<&'a AprilModel>,
}

impl<'a> AprilSession<'a> {
    pub(crate) fn new(ptr: april_asr_rs_sys::AprilASRSession) -> Result<AprilSession<'a>> {
        if ptr.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(Self {
                ptr,
                model: PhantomData,
            })
        }
    }

    pub fn feed_pcm16(&mut self, pcm: &mut [i16]) {
        if pcm.is_empty() {
            return;
        }

        // SAFETY: self.ptr is a valid pointer to a AprilSession
        // pcm is a valid array of c_shorts with pcm.len() elements
        unsafe { april_asr_rs_sys::aas_feed_pcm16(self.ptr, pcm.as_mut_ptr(), pcm.len() as _) }
    }

    pub fn flush(&mut self) {
        unsafe { april_asr_rs_sys::aas_flush(self.ptr) }
    }

    pub fn get_realtime_speedup(&self) -> f32 {
        unsafe { april_asr_rs_sys::aas_realtime_get_speedup(self.ptr) }
    }
}

impl Drop for AprilSession<'_> {
    fn drop(&mut self) {
        unsafe { april_asr_rs_sys::aas_free(self.ptr) }
    }
}
