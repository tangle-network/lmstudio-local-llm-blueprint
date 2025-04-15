// src/jobs/mod.rs
use crate::context::LlmContext;
use crate::error::Result;
use serde_json::Value;
use tracing::instrument;

use crate::error::Error;
use blueprint_sdk::{
    extract::Context,
    info,
    macros::debug_job,
    tangle::extract::{Optional, TangleArg, TangleArgs2, TangleArgs4, TangleResult},
    warn,
};

// Define Job IDs
pub const LOAD_MODEL_JOB_ID: u8 = 0;
pub const UNLOAD_MODEL_JOB_ID: u8 = 1;
pub const LIST_MODELS_JOB_ID: u8 = 2;
pub const LIST_LOADED_MODELS_JOB_ID: u8 = 3;
pub const START_API_JOB_ID: u8 = 4;
pub const STOP_API_JOB_ID: u8 = 5;
pub const STATUS_JOB_ID: u8 = 6; // Added bonus: lms status

// Module for job implementations
pub mod list;
pub mod load;
pub mod server;
pub mod status;
pub mod unload;

// Re-export handlers for easy routing
pub use list::{list_loaded_models, list_models};
pub use load::load_model;
pub use server::{start_api, stop_api};
pub use status::get_status;
pub use unload::unload_model;
