use crate::error::{Error, Result};
use blueprint_sdk::{
    crypto::{sp_core::SpSr25519, tangle_pair_signer::TanglePairSigner},
    debug, error, info,
    keystore::backends::Backend,
    macros::context::{ServicesContext, TangleClientContext},
    runner::config::BlueprintEnvironment,
    trace,
};
use serde::Deserialize;
use std::{path::PathBuf, process::Stdio, sync::Arc};
use tokio::process::Command;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct LmsConfig {
    /// Path to the lms executable. If not set, attempts to find it in default locations.
    pub lms_path: Option<PathBuf>,
    /// Optional data directory for LM Studio state (if needed beyond default).
    pub data_dir: Option<PathBuf>,
    /// Whitelisted models allowed for loading (optional). If empty, all models are allowed.
    #[serde(default)]
    pub allowed_models: Vec<String>,
}

/// Context for the LM Studio Blueprint service.
#[derive(Clone, TangleClientContext, ServicesContext)]
pub struct LlmContext {
    #[config]
    pub env: BlueprintEnvironment,
    pub config: Arc<LmsConfig>,
    pub signer: TanglePairSigner<sp_core::sr25519::Pair>,
    lms_executable: PathBuf,
}

impl LlmContext {
    /// Creates a new LM Studio context.
    pub async fn new(env: BlueprintEnvironment, config: LmsConfig) -> Result<Self> {
        let lms_executable = find_lms_executable(config.lms_path.clone())?;

        // Ensure lms is executable
        let metadata = std::fs::metadata(&lms_executable)?;
        if !metadata.is_file() {
            return Err(Error::Config(format!(
                "lms path is not a file: {}",
                lms_executable.display()
            )));
        }
        // Consider adding execute permission check if needed, especially on Linux/macOS

        let signer_public = env
            .keystore()
            .first_local::<SpSr25519>()
            .map_err(|e| Error::Config(format!("Failed to get local signer key: {}", e)))?;

        let pair_secret = env
            .keystore()
            .get_secret::<SpSr25519>(&signer_public)
            .map_err(|e| Error::Config(format!("Failed to get local signer key: {}", e)))?;

        let signer = TanglePairSigner::new(pair_secret.0);

        Ok(Self {
            env,
            config: Arc::new(config),
            signer,
            lms_executable,
        })
    }

    /// Executes an `lms` command with the given arguments.
    /// Returns the combined stdout and stderr output as a String.
    pub async fn run_lms_command(&self, args: &[&str]) -> Result<String> {
        debug!(
            "Executing lms command: {} {}",
            self.lms_executable.display(),
            args.join(" ")
        );

        let output = Command::new(&self.lms_executable)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| Error::Io(e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        trace!("lms stdout: {}", stdout);
        trace!("lms stderr: {}", stderr);

        if output.status.success() {
            Ok(format!("{}{}", stdout, stderr)) // Combine both for potential info in stderr
        } else {
            let error_message = format!(
                "lms command failed with status: {}\nStdout: {}\nStderr: {}",
                output.status, stdout, stderr
            );
            error!("{}", error_message);
            Err(Error::LmsCommandFailed(error_message))
        }
    }

    /// Checks if a model identifier is allowed based on the whitelist.
    pub fn is_model_allowed(&self, model_identifier: &str) -> bool {
        if self.config.allowed_models.is_empty() {
            true // No whitelist means all models are allowed
        } else {
            self.config
                .allowed_models
                .iter()
                .any(|allowed| allowed == model_identifier)
        }
    }
}

/// Attempts to find the `lms` executable path.
fn find_lms_executable(explicit_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit_path {
        if path.exists() {
            return Ok(path);
        } else {
            return Err(Error::Config(format!(
                "Explicit lms path not found: {}",
                path.display()
            )));
        }
    }

    // Default paths based on documentation
    let default_paths = [
        // macOS / Linux
        dirs::home_dir()
            .map(|h| h.join(".lmstudio/bin/lms"))
            .unwrap_or_default(),
        // Windows
        dirs::home_dir()
            .map(|h| h.join(".lmstudio/bin/lms.exe"))
            .unwrap_or_default(),
    ];

    for path in default_paths.iter() {
        if path.exists() {
            info!("Found lms executable at: {}", path.display());
            return Ok(path.clone());
        }
    }

    // Try searching PATH environment variable as a last resort
    match which::which("lms") {
         Ok(path) => {
             info!("Found lms executable in PATH: {}", path.display());
             Ok(path)
         },
         Err(_) => Err(Error::Config(
             "lms executable not found in default locations or PATH. Please specify 'lms_path' in config.".to_string(),
         )),
     }
}
