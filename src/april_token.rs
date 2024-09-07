use std::borrow::Cow;

#[non_exhaustive] // exclusively to forbid public construction
pub struct AprilToken<'a> {
    pub token: Cow<'a, str>,
    pub logprob: f32,
    pub flag_bits: AprilTokenFlags,
    pub time_ms: usize,
}

impl<'a> AprilToken<'a> {
    pub(crate) fn new(
        token: Cow<'a, str>,
        logprob: f32,
        flag_bits: AprilTokenFlags,
        time_ms: usize,
    ) -> AprilToken<'a> {
        Self {
            token,
            logprob,
            flag_bits,
            time_ms,
        }
    }
}

bitflags::bitflags! {
    pub struct AprilTokenFlags: u32 {
        const EMPTY = 0x0;

        /// If set, this token marks the start of a new word.
        /// In English, this is equivalent to `token[0] == ' '`
        const WORD_BOUNDARY = 0x00000001;

        /// If set, this token marks the end of a sentence, meaning the token is equal to ".", "!", or "?".
        /// Some models may not have this token.
        const SENTENCE_END = 0x00000002;
    }
}
