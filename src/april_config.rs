pub struct AprilConfig {
    ptr: april_asr_rs_sys::AprilConfig,
}

impl Default for AprilConfig {
    fn default() -> Self {
        AprilConfig {
            ptr: april_asr_rs_sys::AprilConfig {
                speaker: april_asr_rs_sys::AprilSpeakerID,
                handler: None,
                userdata: (),
                flags: 0,
            },
        }
    }
}

bitflags::bitflags! {
    pub struct AprilConfigFlags {

    }
}
