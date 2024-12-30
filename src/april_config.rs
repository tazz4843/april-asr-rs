use crate::april_result_type::AprilResultType;
use crate::april_token::{AprilToken, AprilTokenFlags, AprilTokens};
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;

pub type AprilHandlerCallback<D> = Box<dyn FnMut(&D, AprilResultType, AprilTokens)>;

pub struct AprilConfig<D: Sized + Send + Sync> {
    ptr: april_asr_rs_sys::AprilConfig,
    internal_safe_user_data_ptr: *mut c_void,
    phantom_type: PhantomData<D>,
}

impl<D: Sized + Send + Sync> AprilConfig<D> {
    /// Take self and return the raw C representation of the config struct.
    ///
    /// # Note
    /// Doing this means you are now responsible for handling cleanup.
    /// You likely only need to worry about cleanup if you called [`Self::set_handler_fn`], as it
    /// "leaks" a Box to allow Rust access to user data safely.
    /// Cleanup can be done by calling [`Self::from_raw`] and dropping the resulting object.
    pub fn into_raw(self) -> (april_asr_rs_sys::AprilConfig, *mut c_void) {
        let Self {
            ptr,
            internal_safe_user_data_ptr,
            ..
        } = self;
        (ptr, internal_safe_user_data_ptr)
    }

    /// Convert an AprilConfig struct from its raw C variant to this safe Rust wrapper.
    ///
    /// # Safety
    /// * `ptr` must be valid in general (upheld by default, unless you know you've done otherwise)
    /// * `internal_safe_user_data_ptr` must either be null,
    ///   OR be the exact one received from [`Self::into_raw`] with this exact same associated `ptr`.
    /// * If you got this `ptr` from [`Self::into_raw`], and you called [`Self::set_handler_fn`] at *any*
    ///   point during its lifetime, you must not have mutated the function pointer or user data in any way,
    ///   unless you replaced both.
    ///   Do note that replacing both and not running proper cleanup (ie calling this function beforehand) will cause a memory leak.
    pub unsafe fn from_raw(
        ptr: april_asr_rs_sys::AprilConfig,
        internal_safe_user_data_ptr: *mut c_void,
    ) -> Self {
        Self {
            ptr,
            internal_safe_user_data_ptr,
            phantom_type: PhantomData,
        }
    }

    // setting speaker field is unimplemented as docs state 'Currently not implemented, has no effect.'

    /// Set callback handler for April to call. Unsafe variant, see [`Self::set_handler_fn`] for safe variant.
    /// Calling this function clears any prior state automatically.
    ///
    /// # Safety
    /// You must be sure that your function is
    /// * safe to call from C code (that is, no unwinding or panicking)
    /// * does not mutate internal April state
    pub unsafe fn set_handler_fn_raw(
        &mut self,
        handler: april_asr_rs_sys::AprilRecognitionResultHandler,
        user_data: *mut c_void,
    ) {
        // Run a cleanup here to ensure we clean up any leftover data from any calls of set_handler_fn
        // and have clean state
        self.clear_handler_fn();

        self.ptr.handler = handler;
        self.ptr.userdata = user_data;
    }

    /// Safe variant of handler function.
    ///
    /// Note any panics in the handler will be caught by Rust's runtime
    /// and result in an immediate abort: do not panic!
    pub fn set_handler_fn<F>(&mut self, handler: F, data: D)
    where
        F: FnMut(&D, AprilResultType, AprilTokens) + 'static,
    {
        unsafe extern "C" fn trampoline<D>(
            user_data: *mut c_void,
            result_type: april_asr_rs_sys::AprilResultType,
            num_tokens: usize,
            tokens: *const april_asr_rs_sys::AprilToken,
        ) where
            D: Sized + Send + Sync,
        {
            // Assert some invariants that should always be upheld
            if user_data.is_null() {
                unreachable!("got nullptr for AprilConfig::set_handler_fn::trampoline::user_data: this is a bug!");
            } else if !user_data.is_aligned() {
                unreachable!("got unaligned pointer for AprilConfig::set_handler_fn::trampoline::user_data: this is a bug!");
            }

            // SAFETY: casting a pointer obtained from Box::<T>::into_raw to a &mut T is safe as long as
            // this function is not called concurrently. Looking into April code, this invariant holds as of commit
            // 3308e68442664552de593957cad0fa443ea183dd.
            let user_data_ptr = user_data as *mut AprilInnerCallbackData<D>;
            let rusty_user_data = unsafe { &mut *user_data_ptr };

            // Convert the result type
            let result_type_rusty = AprilResultType::from(result_type);

            // Turn the tokens ptr we got into a tokens vec
            let tokens = if tokens.is_null() {
                vec![]
            } else if !tokens.is_aligned() {
                panic!("unaligned tokens array passed to AprilConfig::set_handler_fn::trampoline.tokens");
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

            (rusty_user_data.callback)(
                &rusty_user_data.data,
                result_type_rusty,
                AprilTokens(tokens),
            );
        }

        // Box the user's fn handler
        let fn_handler = Box::new(handler) as Box<dyn FnMut(&D, AprilResultType, AprilTokens)>;
        // Plop both the boxed fn and user data into a data struct and box it too
        let boxed_data_struct = Box::new(AprilInnerCallbackData {
            callback: fn_handler,
            data,
        });
        // Convert that boxed data into a raw *mut c_void ptr
        let raw_data_ptr = Box::into_raw(boxed_data_struct) as *mut c_void;
        // SAFETY: trampoline is safe to call over C FFI boundaries, but only with our raw_data_ptr
        unsafe { self.set_handler_fn_raw(Some(trampoline::<D>), raw_data_ptr) }
        // We *must* set this only after calling the above function, as it clears this field on its own
        self.internal_safe_user_data_ptr = raw_data_ptr;
    }

    /// Clear any handler function previously set with [`Self::set_handler_fn`] or its unsafe variants.
    ///
    /// Calling this before calling [`Self::set_handler_fn`] again will avoid memory leaks from
    /// the internal Box being left unused forever.
    pub fn clear_handler_fn(&mut self) {
        // Use std::mem::replace that way we never leave a possibly bad pointer behind
        let inner_ptr =
            std::mem::replace(&mut self.internal_safe_user_data_ptr, std::ptr::null_mut());
        // SAFETY: this pointer is obtained from a place where the only time it is stored is via Box::<T>::into_raw,
        // or a nullptr is stored.
        unsafe { clean_up_user_data::<D>(inner_ptr) }

        self.ptr.handler = None;
        self.ptr.userdata = std::ptr::null_mut();
    }
}

impl<D: Sized + Send + Sync> Default for AprilConfig<D> {
    fn default() -> Self {
        AprilConfig {
            ptr: april_asr_rs_sys::AprilConfig {
                speaker: april_asr_rs_sys::AprilSpeakerID { data: [0; 16] },
                handler: None,
                userdata: std::ptr::null_mut(),
                flags: 0,
            },
            internal_safe_user_data_ptr: std::ptr::null_mut(),
            phantom_type: PhantomData,
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

/// Given a pointer obtained from `Box::<T>::into_raw`, where `T` was a [`AprilInnerCallbackData`] struct,
/// safely clean it up.
/// This function can also be safely called with a nullptr. Nothing will be done in that case.
///
/// # Safety
/// Either
/// * `user_data` must be null
///
/// OR all of the following must hold:
/// * `user_data` must have been obtained from `Box::<T>::into_raw`
/// * there must be no remaining usages of `user_data`, as any pointers to it will be pointing
///   to garbage after this call.
pub(crate) unsafe fn clean_up_user_data<D: Sized + Send + Sync>(user_data: *mut c_void) {
    if user_data.is_null() {
        // nothing to do to clean up our side as we never actually modified
        // the inner data
    } else {
        // SAFETY: we made this internal ptr, and we know it points to a Box so it is safe to
        // reconstruct as one.
        let inner_data = unsafe { Box::from_raw(user_data as *mut AprilInnerCallbackData<D>) };
        // Drop it to clean up the left behind resources
        drop(inner_data);
    }
}

struct AprilInnerCallbackData<D: Sized + Send + Sync> {
    callback: AprilHandlerCallback<D>,
    data: D,
}
