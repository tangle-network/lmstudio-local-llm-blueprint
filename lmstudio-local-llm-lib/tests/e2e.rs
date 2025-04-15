use blueprint_sdk::Job;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::serde::{from_field, to_field};
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::testing::tempfile;
use blueprint_sdk::testing::utils::harness::TestHarness;
use blueprint_sdk::testing::utils::setup_log;
use blueprint_sdk::testing::utils::tangle::TangleTestHarness;
use lmstudio_local_llm_blueprint_lib::context::LmsConfig;
use lmstudio_local_llm_blueprint_lib::{
    context::LlmContext,
    jobs::{
        LIST_LOADED_MODELS_JOB_ID,
        LIST_MODELS_JOB_ID,
        LOAD_MODEL_JOB_ID,
        START_API_JOB_ID,
        STATUS_JOB_ID,
        STOP_API_JOB_ID,
        UNLOAD_MODEL_JOB_ID,
        // Import job constants and handlers
        get_status,
        list_loaded_models,
        list_models,
        load_model,
        start_api,
        stop_api,
        unload_model,
    },
};
use std::fs::File;
use std::io::Write;

// The number of nodes to spawn in the test
const N: usize = 1;

#[tokio::test]
#[ignore = "Requires lms CLI in PATH and potentially running LM Studio app"] // Ignore by default
async fn test_lmstudio_blueprint_jobs() -> color_eyre::Result<()> {
    setup_log(); // Initialize logging

    // --- Test Setup ---
    let temp_dir = tempfile::TempDir::new()?;

    // Create a minimal config.yaml for the test harness environment
    // This allows LlmContext::new to load its config.
    // We don't specify lms_path, relying on it being in PATH.
    let config_content = r#"
lms_config:
  allowed_models: [] # Allow all models for testing simplicity
"#;
    let config_path = temp_dir.path().join("config.yaml");
    let mut file = File::create(&config_path)?;
    file.write_all(config_content.as_bytes())?;

    // Initialize Tangle test harness
    let temp_dir = tempfile::TempDir::new()?;
    let env = BlueprintEnvironment::default();
    let lms_config = LmsConfig {
        lms_path: None,
        data_dir: env.data_dir.clone(),
        allowed_models: vec![],
    };
    let context = LlmContext::new(env, lms_config).await?;
    let harness = TangleTestHarness::setup(temp_dir, context).await?;

    // Setup service with N nodes. This implicitly creates LlmContext internally.
    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    // Initialize the node(s) and register all job handlers
    test_env.initialize().await?;
    test_env.add_job(load_model.layer(TangleLayer)).await;
    test_env.add_job(unload_model.layer(TangleLayer)).await;
    test_env.add_job(list_models.layer(TangleLayer)).await;
    test_env
        .add_job(list_loaded_models.layer(TangleLayer))
        .await;
    test_env.add_job(start_api.layer(TangleLayer)).await;
    test_env.add_job(stop_api.layer(TangleLayer)).await;
    test_env.add_job(get_status.layer(TangleLayer)).await;

    // Start the test environment. It is now ready to receive job calls.
    test_env.start().await?; // Pass default shutdown signal strategy

    // --- Job Execution ---

    // 1. Get Status
    println!("Testing GET_STATUS (Job {})", STATUS_JOB_ID);
    let job_status_call = harness
        .submit_job(service_id, STATUS_JOB_ID, vec![to_field(()).unwrap()]) // No args
        .await?;
    let result_status = harness
        .wait_for_job_execution(service_id, job_status_call)
        .await?;
    println!("GET_STATUS Result: {:?}", result_status.result);

    // 2. List Downloaded Models
    println!("Testing LIST_MODELS (Job {})", LIST_MODELS_JOB_ID);
    let job_ls_call = harness
        .submit_job(service_id, LIST_MODELS_JOB_ID, vec![to_field(()).unwrap()])
        .await?;
    let result_ls = harness
        .wait_for_job_execution(service_id, job_ls_call)
        .await?;
    let results = result_ls
        .result
        .iter()
        .map(|f| from_field::<Vec<String>>(f.clone()).unwrap());
    println!("LIST_MODELS Result: {:?}", results);

    // 3. List Loaded Models (Initially likely empty)
    println!(
        "Testing LIST_LOADED_MODELS (Job {})",
        LIST_LOADED_MODELS_JOB_ID
    );
    let job_ps_call1 = harness
        .submit_job(service_id, LIST_LOADED_MODELS_JOB_ID, vec![
            to_field(()).unwrap(),
        ])
        .await?;
    let result_ps1 = harness
        .wait_for_job_execution(service_id, job_ps_call1)
        .await?;
    let results = result_ps1
        .result
        .iter()
        .map(|f| from_field::<Vec<String>>(f.clone()).unwrap());
    println!("LIST_LOADED_MODELS Result: {:?}", results);

    // 4. Load Model (Use a placeholder - this might fail if lms isn't running/configured)
    println!("Testing LOAD_MODEL (Job {})", LOAD_MODEL_JOB_ID);
    let model_to_load = "placeholder/test-model-gguf"; // Might not exist
    let load_args: (String, Option<String>, Option<u32>, Option<String>) = (
        model_to_load.to_string(),
        Some("auto".to_string()),
        None,
        None,
    );
    let job_load_call = harness
        .submit_job(service_id, LOAD_MODEL_JOB_ID, vec![
            to_field(load_args).unwrap(),
        ])
        .await?;
    let result_load = harness
        .wait_for_job_execution(service_id, job_load_call)
        .await?;

    let results = result_load
        .result
        .iter()
        .map(|f| from_field::<Vec<String>>(f.clone()).unwrap());
    println!("LOAD_MODEL Result: {:?}", results);

    // 5. List Loaded Models Again (Check if load attempt changed anything)
    println!(
        "Testing LIST_LOADED_MODELS again (Job {})",
        LIST_LOADED_MODELS_JOB_ID
    );
    let job_ps_call2 = harness
        .submit_job(service_id, LIST_LOADED_MODELS_JOB_ID, vec![
            to_field(()).unwrap(),
        ])
        .await?;
    let result_ps2 = harness
        .wait_for_job_execution(service_id, job_ps_call2)
        .await?;
    let results = result_ps2
        .result
        .iter()
        .map(|f| from_field::<Vec<String>>(f.clone()).unwrap());
    println!("LIST_LOADED_MODELS Result: {:?}", results);

    // 6. Unload Model (Attempt to unload the placeholder)
    println!("Testing UNLOAD_MODEL (Job {})", UNLOAD_MODEL_JOB_ID);
    let unload_args: (Option<String>, Option<bool>) = (Some(model_to_load.to_string()), None);
    let job_unload_call = harness
        .submit_job(service_id, UNLOAD_MODEL_JOB_ID, vec![
            to_field(unload_args).unwrap(),
        ])
        .await?;
    let result_unload = harness
        .wait_for_job_execution(service_id, job_unload_call)
        .await?;
    // Similar to load, this might succeed (if model wasn't loaded) or fail.
    println!("UNLOAD_MODEL Result: {:?}", result_unload.result);

    // 7. Start API Server
    println!("Testing START_API (Job {})", START_API_JOB_ID);
    let job_start_call = harness
        .submit_job(service_id, START_API_JOB_ID, vec![to_field(()).unwrap()])
        .await?;
    let result_start = harness
        .wait_for_job_execution(service_id, job_start_call)
        .await?;
    println!("START_API Result: {:?}", result_start.result);
    // Don't hard assert OK, depends on lms state

    // 8. Stop API Server
    println!("Testing STOP_API (Job {})", STOP_API_JOB_ID);
    let job_stop_call = harness
        .submit_job(service_id, STOP_API_JOB_ID, vec![to_field(()).unwrap()])
        .await?;
    let result_stop = harness
        .wait_for_job_execution(service_id, job_stop_call)
        .await?;
    println!("STOP_API Result: {:?}", result_stop.result);
    // Don't hard assert OK, depends on lms state

    println!("e2e test completed.");
    Ok(())
}
