use std::ffi::c_int;
use std::sync::Once;

mod april_config;
mod april_model;
mod april_result_type;
mod april_session;
mod april_token;
mod error;

pub use april_config::{AprilConfig, AprilConfigFlags, AprilHandlerCallback};
pub use april_model::AprilModel;
pub use april_result_type::AprilResultType;
pub use april_session::AprilSession;
pub use april_token::{AprilToken, AprilTokenFlags};
pub use error::{Error, Result};

static ASSERT_INIT: Once = Once::new();

/// Initialize April once and exactly once. Safe to call multiple times,
/// later calls have no effect.
fn do_init() {
    ASSERT_INIT.call_once(|| unsafe {
        april_asr_rs_sys::aam_api_init(april_asr_rs_sys::APRIL_VERSION as c_int)
    })
}

pub use april_asr_rs_sys::AprilRecognitionResultHandler;
