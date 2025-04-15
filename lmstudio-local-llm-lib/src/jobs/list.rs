use super::*;

/// Tangle Job handler to list all downloaded models using `lms ls --json`.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = LIST_MODELS_JOB_ID))]
pub async fn list_models(
    Context(ctx): Context<LlmContext>,
    // No arguments needed for this job
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<String>> {
    info!("Received list_models job");

    let args = ["ls", "--json"];
    let output = ctx.run_lms_command(&args).await?;

    // Basic validation: Check if it's valid JSON
    let _json: Value = serde_json::from_str(&output).map_err(|e| Error::LmsOutputParse(e))?;

    info!("Successfully listed downloaded models.");
    Ok(TangleResult(output)) // Return raw JSON string
}

/// Tangle Job handler to list all currently loaded models using `lms ps --json`.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = LIST_LOADED_MODELS_JOB_ID))]
pub async fn list_loaded_models(
    Context(ctx): Context<LlmContext>,
    // No arguments needed
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<String>> {
    info!("Received list_loaded_models job");

    let args = ["ps", "--json"];
    let output = ctx.run_lms_command(&args).await?;

    // Basic validation: Check if it's valid JSON
    let _json: Value = serde_json::from_str(&output).map_err(|e| Error::LmsOutputParse(e))?;

    info!("Successfully listed loaded models.");
    Ok(TangleResult(output)) // Return raw JSON string
}
