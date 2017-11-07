//! Digital frequency counter module UR20-2FCNT-100

use super::*;

#[derive(Debug, Clone)]
pub struct Parameters {
    pub input_filter: InputFilter,
}
