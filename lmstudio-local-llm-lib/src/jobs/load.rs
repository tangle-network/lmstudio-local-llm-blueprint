use super::*;

/// Tangle Job handler to load a model using `lms load`.
///
/// # Arguments
/// * `model_identifier`: The identifier of the model to load (e.g., "TheBloke/phi-2-GGUF"). Required.
/// * `gpu`: GPU configuration ("max", "auto", "0.0"-"1.0"). Optional.
/// * `context_length`: Context length as an integer. Optional.
/// * `alias`: An alias to assign to the loaded model. Optional.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = LOAD_MODEL_JOB_ID))]
pub async fn load_model(
    Context(ctx): Context<LlmContext>,
    TangleArgs4(
        model_identifier,
        Optional(gpu),
        Optional(context_length),
        Optional(alias),
    ): TangleArgs4<String, Optional<String>, Optional<u32>, Optional<String>>,
) -> Result<TangleResult<String>> {
    info!(
        "Received load_model job for: {} (gpu: {:?}, ctx_len: {:?}, alias: {:?})",
        model_identifier, gpu, context_length, alias
    );

    // --- Validation ---
    if !ctx.is_model_allowed(&model_identifier) {
        warn!(
            "Model identifier '{}' is not in the allowed list.",
            model_identifier
        );
        return Err(Error::InvalidArgs(format!(
            "Model '{}' is not allowed by this service instance.",
            model_identifier
        )));
    }

    let mut args = vec!["load", model_identifier.as_str()];

    // Validate and add GPU argument
    if let Some(ref gpu_config) = gpu {
        // Basic validation - could be more strict (e.g., regex for float)
        if !["max", "auto"].contains(&gpu_config.to_lowercase().as_str()) {
            if let Ok(val) = gpu_config.parse::<f32>() {
                if !(0.0..=1.0).contains(&val) {
                    return Err(Error::InvalidArgs(format!(
                        "Invalid GPU value '{}'. Must be 'max', 'auto', or a float between 0.0 and 1.0.",
                        gpu_config
                    )));
                }
            } else {
                return Err(Error::InvalidArgs(format!(
                    "Invalid GPU value '{}'. Must be 'max', 'auto', or a float between 0.0 and 1.0.",
                    gpu_config
                )));
            }
        }
        args.push("--gpu");
        args.push(gpu_config); // Pass the original string
    }

    // Add context length argument
    let context_length_str; // Needs to live long enough
    if let Some(len) = context_length {
        context_length_str = len.to_string();
        args.push("--context-length");
        args.push(&context_length_str);
    }

    // Add alias argument
    if let Some(ref model_alias) = alias {
        args.push("--identifier"); // lms uses --identifier for alias
        args.push(model_alias);
    }

    // Add -y to automatically confirm (useful for non-interactive blueprint)
    args.push("-y");

    // --- Execution ---
    let output = ctx.run_lms_command(&args).await?;

    info!("Successfully loaded model '{}'", model_identifier);
    Ok(TangleResult(output))
}
