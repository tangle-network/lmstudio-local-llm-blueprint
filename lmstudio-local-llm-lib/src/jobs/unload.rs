// src/jobs/unload.rs
use super::*;
use crate::error::Error;

/// Tangle Job handler to unload a model using `lms unload`.
///
/// # Arguments
/// * `model_identifier`: The identifier of the model to unload. Optional.
/// * `unload_all`: If true, unloads all models (`--all` flag). Optional.
///
/// Exactly one of `model_identifier` or `unload_all=true` must be provided.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = UNLOAD_MODEL_JOB_ID))]
pub async fn unload_model(
    Context(ctx): Context<LlmContext>,
    TangleArgs2(Optional(model_identifier), Optional(unload_all)): TangleArgs2<
        Optional<String>,
        Optional<bool>,
    >,
) -> Result<TangleResult<String>> {
    info!(
        "Received unload_model job (model: {:?}, all: {:?})",
        model_identifier,
        unload_all.unwrap_or(false)
    );

    let mut args = vec!["unload"];

    // Use if let/else if structure for clarity and lifetime management
    if let Some(id) = &model_identifier {
        // Borrow Option content
        if unload_all.unwrap_or(false) {
            return Err(Error::InvalidArgs(
                "Cannot specify both a model identifier and unload_all=true.".to_string(),
            ));
        }
        // 'id' is now &String, borrow lives long enough
        args.push(id.as_str()); // Push the borrowed slice
        info!("Unloading specific model: {}", id);
    } else if unload_all.unwrap_or(false) {
        args.push("--all");
        info!("Unloading all models");
    } else {
        // Case where neither identifier nor --all is provided
        return Err(Error::InvalidArgs(
            "Must specify either a model identifier or unload_all=true.".to_string(),
        ));
    }

    // --- Execution ---
    let output = ctx.run_lms_command(&args).await?;

    info!("Successfully executed unload command.");
    // Correct the return type wrapper
    Ok(TangleResult(output))
}
