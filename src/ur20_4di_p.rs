//! Digital input module UR20-4DI-P

use super::*;

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub input_delay: InputDelay,
}
