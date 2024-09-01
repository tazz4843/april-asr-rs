use std::ffi::c_int;
use std::sync::OnceLock;

mod april_config;
mod april_model;
mod april_session;
mod error;

static ASSERT_INIT: OnceLock<()> = OnceLock::new();

/// Initialize April once and exactly once. Safe to call multiple times,
/// subsequent calls have no effect.
fn do_init() {
    if ASSERT_INIT.set(()).is_ok() {
        unsafe {
            april_asr_rs_sys::aam_api_init(april_asr_rs_sys::APRIL_VERSION as c_int);
        }
    }
}
