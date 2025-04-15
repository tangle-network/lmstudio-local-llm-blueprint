# <h1 align="center">🤖 LM Studio Tangle Blueprint: Local LLM-as-a-Service 🧠</h1>

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

## 📚 Overview

This Tangle Blueprint provides a powerful **Local LLM-as-a-Service** layer, bridging the gap between the decentralized Tangle Network and the local capabilities of [LM Studio](https://lmstudio.ai/). It allows operators to securely expose and manage local Large Language Models (LLMs) via Tangle jobs, making advanced AI accessible through blockchain interactions.

Users can submit jobs to request specific models, manage their lifecycle (load/unload), check status, and control the LM Studio inference server running on the operator's machine—all orchestrated through the Tangle Network's robust infrastructure.

This blueprint leverages the LM Studio Command Line Interface (`lms`) for interacting with the local LM Studio instance, enabling automation and integration within the Tangle ecosystem.

For more background on Tangle Blueprints, please refer to the [official Tangle documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## ✨ Features

This blueprint exposes the core functionalities of the `lms` CLI as Tangle jobs:

- **Load Model (`LOAD_MODEL_JOB_ID = 0`):** Loads a specified model (e.g., `TheBloke/phi-2-GGUF`) into LM Studio, with options for GPU acceleration, context length, and aliasing.
- **Unload Model (`UNLOAD_MODEL_JOB_ID = 1`):** Unloads a specific model by identifier or unloads all currently loaded models.
- **List Downloaded Models (`LIST_MODELS_JOB_ID = 2`):** Returns a JSON list of all models downloaded and available in the operator's LM Studio library (`lms ls --json`).
- **List Loaded Models (`LIST_LOADED_MODELS_JOB_ID = 3`):** Returns a JSON list of models currently loaded and ready for inference (`lms ps --json`).
- **Start API Server (`START_API_JOB_ID = 4`):** Starts the LM Studio local inference server (`lms server start`).
- **Stop API Server (`STOP_API_JOB_ID = 5`):** Stops the LM Studio local inference server (`lms server stop`).
- **Get Status (`STATUS_JOB_ID = 6`):** Retrieves the current status of the LM Studio application (`lms status`).
- **Secure Execution:** Uses direct command execution, avoiding shell injection vulnerabilities.
- **Configurable:** Allows operators to whitelist specific models via configuration.

## ⚙️ How It Works

The blueprint operates as a standard Tangle service:

1.  **Tangle Producer:** Listens for finalized blocks on the Tangle network, identifying relevant job requests.
2.  **Router:** Maps incoming job requests based on their Job ID to the corresponding Rust handler function.
3.  **Job Handlers:** Parse arguments from the job request (model identifiers, flags, etc.).
4.  **`LlmContext`:** Holds configuration (like the `lms` path) and provides a method to securely execute `lms` commands as subprocesses.
5.  **`lms` CLI:** The blueprint invokes the local `lms` executable with the appropriate arguments.
6.  **Tangle Consumer:** Captures the output (stdout/stderr) or success/failure status from the `lms` command and submits the result back to the Tangle network.

## 📋 Prerequisites

Before running or developing this blueprint, ensure you have:

1.  **Rust:** Install the Rust toolchain from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **`cargo-tangle`:** The Tangle CLI for blueprint management. Install it with:
    ```bash
    cargo install cargo-tangle --git https://github.com/webb-tools/tangle --tag v0.1.0 # Use the official repo
    ```
3.  **LM Studio:** Download and install LM Studio from [lmstudio.ai](https://lmstudio.ai/).
4.  **`lms` CLI:** **Crucially**, you must run LM Studio at least once and then **bootstrap** the `lms` CLI so it's available in your system's PATH. Follow the instructions in the LM Studio app or run:
    - **macOS/Linux:** `~/.lmstudio/bin/lms bootstrap`
    - **Windows (PowerShell):** `cmd /c $env:USERPROFILE/.lmstudio/bin/lms.exe bootstrap`
    - Verify by opening a **new** terminal and running `lms`.

## ⭐ Getting Started

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/your-username/lmstudio-local-llm.git # Replace with actual repo URL
    cd lmstudio-local-llm
    ```
2.  **Configure:** Create or modify the `config.yaml` file in the project root. This file is essential for the `LlmContext`.

    ```yaml
    # config.yaml (Example)

    # Tangle network details (node URL, etc.)
    # Provided during deployment or via environment variables typically handled by cargo-tangle

    # LM Studio specific configuration
    lms_config:
      # Optional: Explicit path to the 'lms' executable.
      # If commented out or null, the blueprint searches default LM Studio paths and the system PATH.
      # lms_path: "/Users/youruser/.lmstudio/bin/lms" # Example for macOS

      # Optional: Specify a custom data directory for LM Studio state. Usually not needed.
      # data_dir: "/path/to/your/lmstudio/data"

      # Optional: Whitelist specific models operators are allowed to load via jobs.
      # If empty or omitted, *all* models discovered by `lms` are allowed.
      # It is RECOMMENDED to configure this for security.
      allowed_models:
        - "TheBloke/phi-2-GGUF"
        - "NousResearch/Hermes-2-Pro-Mistral-7B-GGUF"
        # - "Other-Allowed-Model-Identifier"
    ```

3.  **Build:** Compile the blueprint project:
    ```bash
    cargo build --release
    ```

## 🚀 Usage

1.  **Deploy:** Deploy the blueprint to a Tangle network using the Tangle CLI:
    ```bash
    cargo tangle blueprint deploy --provider <rpc_url> # Provide the Tangle node RPC URL
    # Follow the prompts for configuration and signing
    ```
2.  **Interact:** Once deployed and a service instance is running, users can submit jobs via the Tangle network. The exact mechanism depends on your Tangle client/SDK, but the core components are:

    - **`service_id`:** The ID of the running service instance.
    - **`job_id`:** The numeric ID of the job to execute (see Features section).
    - **`inputs`:** SCALE-encoded arguments for the job.

    **Example Job Submissions (Conceptual):**

    - **List Downloaded Models (Job ID 2):**

      - `job_id`: `2`
      - `inputs`: `()` (No arguments, SCALE-encoded unit type)
      - _Expected Result:_ SCALE-encoded `String` containing JSON output from `lms ls --json`.

    - **Load Phi-2 Model (Job ID 0):**

      - `job_id`: `0`
      - `inputs`: `("TheBloke/phi-2-GGUF", Some("max"), None, Some("phi2-service"))` (SCALE-encoded tuple: `(String, Option<String>, Option<u32>, Option<String>)`)
      - _Expected Result:_ SCALE-encoded `String` containing output from the `lms load` command.

    - **Unload All Models (Job ID 1):**
      - `job_id`: `1`
      - `inputs`: `(None, Some(true))` (SCALE-encoded tuple: `(Option<String>, Option<bool>)`)
      - _Expected Result:_ SCALE-encoded `String` containing output from `lms unload --all`.

## 🛠️ Development

- **Build:** `cargo build`
- **Format:** `cargo fmt`
- **Lint:** `cargo clippy`
- **Test:**
  - Run unit tests: `cargo test`
  - Run ignored E2E tests (requires `lms` in PATH and potentially a running LM Studio instance): `cargo test -- --ignored`

## 🔐 Security Considerations

- **Model Whitelisting:** It is **strongly recommended** to use the `allowed_models` configuration in `config.yaml` to restrict which models can be loaded via jobs. Allowing arbitrary model loading could expose the operator's machine to risks if malicious model files exist.
- **Command Execution:** The blueprint executes `lms` directly without using a shell, mitigating shell injection risks. Arguments are passed explicitly.
- **Resource Usage:** Loading large LLMs consumes significant RAM, VRAM, and CPU. Operators should ensure their hardware is sufficient for the models they allow.

## 📜 License

This project is dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
