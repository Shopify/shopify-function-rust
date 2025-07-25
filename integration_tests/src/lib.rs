use anyhow::Result;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::Command,
    sync::LazyLock,
};

const FUNCTION_RUNNER_VERSION: &str = "8.0.0";
const TRAMPOLINE_VERSION: &str = "1.0.0";

fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::PathBuf::from(manifest_dir).join("..")
}

/// Builds the example to a `.wasm` file
fn build_example(name: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-wasip1",
            "-p",
            name,
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!(status);
    }
    Ok(())
}

static FUNCTION_RUNNER_PATH: LazyLock<anyhow::Result<PathBuf>> = LazyLock::new(|| {
    let path = workspace_root().join(format!("tmp/function-runner-{FUNCTION_RUNNER_VERSION}"));

    if !path.exists() {
        std::fs::create_dir_all(workspace_root().join("tmp"))?;
        download_function_runner(&path)?;
    }

    Ok(path)
});

static TRAMPOLINE_PATH: LazyLock<anyhow::Result<PathBuf>> = LazyLock::new(|| {
    let path = workspace_root().join(format!("tmp/trampoline-{TRAMPOLINE_VERSION}"));
    if !path.exists() {
        std::fs::create_dir_all(workspace_root().join("tmp"))?;
        download_trampoline(&path)?;
    }
    Ok(path)
});

fn download_function_runner(destination: &PathBuf) -> Result<()> {
    download_from_github(
        |target_arch, target_os| {
            format!(
                "https://github.com/Shopify/function-runner/releases/download/v{FUNCTION_RUNNER_VERSION}/function-runner-{target_arch}-{target_os}-v{FUNCTION_RUNNER_VERSION}.gz",
            )
        },
        destination,
    )
}

fn download_trampoline(destination: &PathBuf) -> Result<()> {
    download_from_github(
        |target_arch, target_os| {
            format!(
            "https://github.com/Shopify/shopify-function-wasm-api/releases/download/shopify_function_trampoline/v{TRAMPOLINE_VERSION}/shopify-function-trampoline-{target_arch}-{target_os}-v{TRAMPOLINE_VERSION}.gz",
        )
        },
        destination,
    )
}

/// Downloads a file from github and saves it to the given destination
///
/// The url_builder is a function that takes the target_arch and target_os and returns the url
fn download_from_github(
    url_builder: impl Fn(&str, &str) -> String,
    destination: &PathBuf,
) -> Result<()> {
    let target_os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        anyhow::bail!("Unsupported target OS");
    };

    let target_arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm"
    } else {
        anyhow::bail!("Unsupported target architecture");
    };

    let url = url_builder(target_arch, target_os);

    let response = reqwest::blocking::get(&url)?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download artifact: {}", response.status());
    }
    let bytes = response.bytes()?;
    let mut gz_decoder = flate2::read::GzDecoder::new(bytes.as_ref());
    let mut file = std::fs::File::create(destination)?;
    std::io::copy(&mut gz_decoder, &mut file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }

    Ok(())
}

pub fn prepare_example(name: &str) -> Result<PathBuf> {
    build_example(name)?;
    let wasm_path = workspace_root()
        .join("target/wasm32-wasip1/release")
        .join(format!("{name}.wasm"));
    let trampolined_path = workspace_root()
        .join("target/wasm32-wasip1/release")
        .join(format!("{name}-trampolined.wasm"));
    let trampoline_path = TRAMPOLINE_PATH
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to download trampoline: {}", e))?;
    let status = Command::new(trampoline_path)
        .args([
            "-i",
            wasm_path
                .to_str()
                .expect("Failed to convert wasm path to string"),
            "-o",
            trampolined_path
                .to_str()
                .expect("Failed to convert wasm path to string"),
        ])
        .status()?;
    assert!(status.success());
    Ok(trampolined_path)
}

pub fn run_example(
    path: PathBuf,
    export: &str,
    input: serde_json::Value,
) -> Result<serde_json::Value> {
    let function_runner_path = FUNCTION_RUNNER_PATH
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to download function runner: {}", e))?;
    let input_json = serde_json::to_string(&input)?;
    let mut child = Command::new(function_runner_path)
        .args([
            "--json",
            "--function",
            path.to_str().expect("Failed to convert path to string"),
            "--export",
            export,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;

    std::thread::spawn(move || {
        stdin
            .write_all(input_json.as_bytes())
            .expect("Failed to write to stdin");
    });

    let status = child.wait()?;

    let mut output = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;
    let mut output_bytes = Vec::new();
    output.read_to_end(&mut output_bytes)?;

    let output: serde_json::Value = serde_json::from_slice(&output_bytes)?;

    if !status.success() {
        let logs = output
            .get("logs")
            .ok_or_else(|| anyhow::anyhow!("No logs"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Logs are not a string"))?;

        anyhow::bail!(
            "Function runner returned non-zero exit code: {}, logs: {}",
            status,
            logs,
        );
    }

    let output_json_str = output
        .get("output")
        .and_then(|o| o.get("humanized").and_then(|h| h.as_str()))
        .ok_or_else(|| anyhow::anyhow!("No output"))?;
    let output_json: serde_json::Value = serde_json::from_str(output_json_str)?;
    Ok(output_json)
}
