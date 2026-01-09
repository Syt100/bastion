mod types;
mod validation;

pub use types::*;
pub use validation::{parse_value, validate, validate_value};

pub const JOB_SPEC_VERSION: u32 = 1;
