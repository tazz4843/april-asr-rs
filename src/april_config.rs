use crate::april_result_type::AprilResultType;
use crate::april_token::{AprilToken, AprilTokenFlags, AprilTokens};
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::sync::Arc;

pub type AprilHandlerCallback = Box<dyn FnMut(AprilResultType, AprilTokens)>;

pub struct AprilConfig<'a> {
    ptr: april_asr_rs_sys::AprilConfig,
    phantom: PhantomData<&'a ()>,
}

impl<'a> AprilConfig<'a> {
    pub fn into_raw(self) -> april_asr_rs_sys::AprilConfig {
        self.ptr
    }

    // setting speaker field is unimplemented as docs state 'Currently not implemented, has no effect.'

    /// Set callback handler for April to call. Unsafe variant, see [`Self::set_handler_fn`] for safe variant.
    ///
    /// # Safety
    /// You must be sure that your function is
    /// * safe to call from C code
    /// * can safely synchronize data
    pub unsafe fn set_handler_fn_raw(
        &mut self,
        handler: april_asr_rs_sys::AprilRecognitionResultHandler,
    ) {
        self.ptr.handler = handler;
    }

    /// Set user_data for the callback handler.
    ///
    /// # Safety
    /// * if you called [`Self::set_handler_fn`]  at any point before calling this,
    ///   you **must** update the function being called with [`Self::set_handler_fn_raw`] otherwise
    ///   you **will** get Undefined Behaviour as the default trampoline tries to call your
    ///   data as a function.
    pub unsafe fn set_user_data(&mut self, user_data: *mut c_void) {
        self.ptr.userdata = user_data;
    }

    /// Safe variant of handler function.
    ///
    /// Note any panics in the handler will be caught by Rust's runtime
    /// and result in an immediate abort: do not panic!
    pub fn set_handler_fn<O, F>(&mut self, handler: O)
    where
        F: FnMut(AprilResultType, AprilTokens) + 'a,
        O: Into<Option<F>>,
    {
        unsafe extern "C" fn trampoline<F>(
            user_data: *mut c_void,
            result_type: april_asr_rs_sys::AprilResultType,
            num_tokens: usize,
            tokens: *const april_asr_rs_sys::AprilToken,
        ) where
            F: FnMut(AprilResultType, AprilTokens),
        {
            if user_data.is_null() {
                unreachable!("got nullptr for AprilConfig::set_handler_fn::trampoline::user_data: this is a bug!");
            }

            // SAFETY: genuinely who the fuck knows
            let user_fn = unsafe { &mut *(user_data as *mut AprilHandlerCallback) };

            let result_type_rusty = AprilResultType::from(result_type);

            let tokens = if tokens.is_null() {
                vec![]
            } else if !tokens.is_aligned() {
                panic!("unaligned tokens array passed to AprilConfig::set_handler_fn::trampoline.tokens");
                // we've done something very wrong here,
                // and we're also in a function that cannot unwind,
                // but, to be safe, abort
                std::process::abort();
            } else {
                // SAFETY: we must trust that april gives us a valid ptr + a valid length,
                // which should always be upheld
                let token_array = unsafe { std::slice::from_raw_parts(tokens, num_tokens) };

                let mut tokens = Vec::new();
                for elm in token_array {
                    let april_asr_rs_sys::AprilToken {
                        token,
                        logprob,
                        flags,
                        time_ms,
                        ..
                    } = elm;
                    let token = unsafe { CStr::from_ptr(*token) }.to_string_lossy();
                    let flag_bits = AprilTokenFlags::from_bits_retain(*flags);

                    tokens.push(AprilToken::new(token, *logprob, flag_bits, *time_ms));
                }
                tokens
            };

            user_fn(result_type_rusty, AprilTokens(tokens));
        }

        match handler.into() {
            Some(handler) => {
                // Stable address
                let handler = Box::new(handler) as Box<dyn FnMut(AprilResultType, AprilTokens)>;
                // Thin ptr
                let handler = Box::new(handler);
                // Raw ptr
                let handler = Box::into_raw(handler);
                self.ptr.handler = Some(trampoline::<F>);
                self.ptr.userdata = handler as *mut c_void;
            }
            None => {
                self.ptr.handler = None;
                self.ptr.userdata = std::ptr::null_mut();
            }
        }
    }
}

impl Default for AprilConfig<'_> {
    fn default() -> Self {
        AprilConfig {
            ptr: april_asr_rs_sys::AprilConfig {
                speaker: april_asr_rs_sys::AprilSpeakerID { data: [0; 16] },
                handler: None,
                userdata: std::ptr::null_mut(),
                flags: 0,
            },
            phantom: PhantomData,
        }
    }
}

bitflags::bitflags! {
    pub struct AprilConfigFlags: i32 {
        /// If set, the input audio should be fed in realtime (1 second of audio per second)
        /// in small chunks.
        ///
        /// Calls to `aas_feed_pcm16` and `aas_flush`
        /// will be fast as it will delegate processing to a background thread.
        /// The handler will be called from the background thread at some point later.
        ///
        /// The accuracy may be degraded depending on the system hardware.
        /// You may get an accuracy estimate by calling `aas_realtime_get_speedup`.
        const ASYNC_RT = 0x00000001;

        /// Similar to [`AprilConfigFlags::ASYNC_RT`], but does not degrade accuracy depending on system hardware.
        /// However, if the system is not fast enough to process audio,
        /// the background thread will fall behind, results may become unusable,
        /// and the handler will be called with APRIL_RESULT_ERROR_CANT_KEEP_UP.
        const ASYNC_NO_RT = 0x00000002;
    }
}
