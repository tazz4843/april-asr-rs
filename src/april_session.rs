use crate::april_model::AprilModel;
use crate::error::{Error, Result};
use std::ffi::c_void;
use std::marker::PhantomData;

pub struct AprilSession<'a, D: Sized + Send + Sync> {
    ptr: april_asr_rs_sys::AprilASRSession,
    user_data_ptr: *mut c_void,
    phantom_model: PhantomData<&'a AprilModel>,
    phantom_type: PhantomData<D>,
}

impl<'a, D: Sized + Send + Sync> AprilSession<'a, D> {
    pub(crate) fn new(
        ptr: april_asr_rs_sys::AprilASRSession,
        user_data_ptr: *mut c_void,
    ) -> Result<AprilSession<'a, D>> {
        if ptr.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(Self {
                ptr,
                user_data_ptr,
                phantom_model: PhantomData,
                phantom_type: PhantomData,
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

impl<D: Sized + Send + Sync> Drop for AprilSession<'_, D> {
    fn drop(&mut self) {
        // Run april cleanup
        unsafe { april_asr_rs_sys::aas_free(self.ptr) }

        // After we've destroyed everything coming from April, we can safely destroy our data now
        // SAFETY: this ptr was passed in from a Box::<T>::into_raw call, or it was originally a nullptr,
        // both from AprilConfig, and it remained untouched through the whole life of self.
        unsafe {
            crate::april_config::clean_up_user_data::<D>(self.user_data_ptr);
        }
    }
}
