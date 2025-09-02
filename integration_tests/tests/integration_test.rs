use anyhow::Result;
use integration_tests::{prepare_example, run_example};
use std::{path::PathBuf, sync::LazyLock};

static EXAMPLE_WITH_TARGETS_RESULT: LazyLock<Result<PathBuf>> =
    LazyLock::new(|| prepare_example("example_with_targets"));

#[test]
fn test_example_with_targets_target_a() -> Result<()> {
    let path = EXAMPLE_WITH_TARGETS_RESULT
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to prepare example: {}", e))?;
    let input = serde_json::json!({
        "id": "gid://shopify/Order/1234567890",
        "num": 123,
        "name": "test",
        "country": "CA"
    });
    let (output, logs) = run_example(path.clone(), "target_a", input)?;
    assert_eq!(
        output,
        serde_json::json!({
            "status": 200
        })
    );
    assert_eq!(logs, "In target_a\nWith var: 42\nWith var: 42\n");
    Ok(())
}

#[test]
fn test_example_with_targets_target_b() -> Result<()> {
    let path = EXAMPLE_WITH_TARGETS_RESULT
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to prepare example: {}", e))?;
    let input = serde_json::json!({
        "id": "gid://shopify/Order/1234567890",
        "targetAResult": 200
    });
    let (output, logs) = run_example(path.clone(), "target_b", input)?;
    assert_eq!(
        output,
        serde_json::json!({
            "name": "new name: \"gid://shopify/Order/1234567890\"",
            "operations": [
                {
                    "doThis": {
                        "thisField": "this field"
                    }
                },
                {
                    "doThat": {
                        "thatField": 42
                    }
                }
            ]
        })
    );
    assert_eq!(logs, "In target_b\n");
    Ok(())
}

#[test]
fn test_example_with_panic() -> Result<()> {
    let path = EXAMPLE_WITH_TARGETS_RESULT
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to prepare example: {}", e))?;
    let input = serde_json::json!({
        "id": "gid://shopify/Order/1234567890",
        "targetAResult": "foo"
    });
    let err = run_example(path.clone(), "target_panic", input)
        .unwrap_err()
        .to_string();
    let expected_err =
        "Function runner returned non-zero exit code: exit status: 1, logs: panicked at example_with_targets/src/main.rs:44:5:\nSomething went wrong\nerror while executing at wasm backtrace:";
    assert!(
        err.contains(expected_err),
        "Expected error message to contain:\n`{expected_err}`\nbut was:\n`{err}`"
    );
    Ok(())
}
