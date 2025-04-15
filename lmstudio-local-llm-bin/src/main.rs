use blueprint_sdk::Job;
use blueprint_sdk::Router;
use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::info;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::consumer::TangleConsumer;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::producer::TangleProducer;
use lmstudio_local_llm_blueprint_lib::context::LlmContext;
use lmstudio_local_llm_blueprint_lib::context::LmsConfig;
use lmstudio_local_llm_blueprint_lib::jobs::*;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), blueprint_sdk::error::Error> {
    // Load environment variables and initialize logging
    dotenv::dotenv().ok();

    // Load Blueprint environment configuration
    let env = BlueprintEnvironment::load()?;

    // Initialize the custom context
    let context = LlmContext::new(env.clone(), LmsConfig::default())
        .await
        .map_err(|e| blueprint_sdk::error::Error::Other(e.to_string()))?;

    // Set up Tangle producer (listens for finalized blocks)
    let client = context.tangle_client().await?; // Get client from context
    let producer = TangleProducer::finalized_blocks(client.rpc_client.clone()).await?;

    // Set up Tangle consumer (sends results back)
    let consumer = TangleConsumer::new(client.rpc_client.clone(), context.signer.clone());

    // --- Define the Job Router ---
    let router = Router::new()
        .route(
            LOAD_MODEL_JOB_ID,
            load_model.layer(TangleLayer), // Apply TangleLayer for filtering/context
        )
        .route(UNLOAD_MODEL_JOB_ID, unload_model.layer(TangleLayer))
        .route(LIST_MODELS_JOB_ID, list_models.layer(TangleLayer))
        .route(
            LIST_LOADED_MODELS_JOB_ID,
            list_loaded_models.layer(TangleLayer),
        )
        .route(START_API_JOB_ID, start_api.layer(TangleLayer))
        .route(STOP_API_JOB_ID, stop_api.layer(TangleLayer))
        .route(STATUS_JOB_ID, get_status.layer(TangleLayer))
        .with_context(context); // Pass the context to all handlers

    info!("Starting LM Studio Tangle Blueprint...");

    // Build and run the Blueprint
    BlueprintRunner::builder(TangleConfig::default(), env)
        .router(router)
        .producer(producer)
        .consumer(consumer)
        .run()
        .await?;

    Ok(())
}

pub fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .without_time()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
