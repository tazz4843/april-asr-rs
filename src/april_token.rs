use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub struct AprilTokens<'a>(pub Vec<AprilToken<'a>>);
impl std::fmt::Display for AprilTokens<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for token in &self.0 {
            std::fmt::Display::fmt(&token, f)?;
        }
        Ok(())
    }
}

#[non_exhaustive] // exclusively to forbid public construction
#[derive(Debug, Clone)]
pub struct AprilToken<'a> {
    pub token: Cow<'a, str>,
    pub logprob: f32,
    pub flag_bits: AprilTokenFlags,
    pub time_ms: usize,
}

impl std::fmt::Display for AprilToken<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
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
    #[derive(Copy, Clone, Debug)]
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
