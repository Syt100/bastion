mod types;
mod v2;
mod validation;

pub use types::*;
pub use v2::{
    AUTH_REF_WEBDAV_CREDENTIALS, AuthRefV2, JOB_SPEC_VERSION_V2, JobSpecV2, SourceEnvelopeV2,
    TargetEnvelopeV2, parse_canonical_value, translate_v1_to_v2, translate_v2_to_v1,
};
pub use validation::{parse_value, validate, validate_canonical, validate_value};

pub const JOB_SPEC_VERSION: u32 = 1;
