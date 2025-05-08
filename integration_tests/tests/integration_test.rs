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
    let output = run_example(path.clone(), "target_a", input)?;
    assert_eq!(
        output,
        serde_json::json!({
            "status": 200
        })
    );
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
    let output = run_example(path.clone(), "target_b", input)?;
    assert_eq!(
        output,
        serde_json::json!({
            "name": "new name: \"gid://shopify/Order/1234567890\""
        })
    );
    Ok(())
}
