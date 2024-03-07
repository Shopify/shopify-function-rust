# Shopify Rust function example with targets

This is an example of how to use the [Shopify Function Rust crate][crate] with the `shopify_function_target` macro to write a Shopify Function with multiple API targets.

## Example API

Target | Handle (`snake_case`) | GraphQL mutation
-- | -- | --
`example.target-a` | `target_a` | `targetA(result: FunctionTargetAResult!)`
`example.target-b` | `target_b` | `targetB(result: FunctionTargetBResult!)`

### `schema.graphql`

```graphql
"""
The input object for the function.
"""
type Input {
  id: ID!
  num: Int
  name: String
  targetAResult: Int @restrictTarget(only: ["example.target-b"])
}

"""
The root mutation for the API.
"""
type MutationRoot {
  """
  The function for API target A.
  """
  targetA(
    """
    The result of calling the function for API target A.
    """
    result: FunctionTargetAResult!
  ): Void!

  """
  The function for API target B.
  """
  targetB(
    """
    The result of calling the function for API target B.
    """
    result: FunctionTargetBResult!
  ): Void!
}

"""
The result of API target A.
"""
input FunctionTargetAResult {
  status: Int
}

"""
The result of API target B.
"""
input FunctionTargetBResult {
  name: String
}
```

## `shopify.function.extension.toml`

```toml
[[targeting]]
target = "example.target-a"
input_query = "a.graphql"
export = "target_a"

[[targeting]]
target = "example.target-b"
input_query = "b.graphql"
export = "function_b"
```

- `target`: The API-specific handle for the target implemented by the Wasm function.
- `input_query`: The path to the target-specific input query file.
- `export` (optional): The name of the Wasm function export to run.
  - default: The target handle as `snake_case`.

## `shopify_function_target` usage

### Arguments

- `query_path`: A path to a GraphQL query, whose result will be used
  as the input for the function invocation. The query MUST be named "Input".
- `schema_path`: A path to Shopify's GraphQL schema definition. Use the CLI
  to download a fresh copy.
- `target` (optional): The API-specific handle for the target if the function name does not match the target handle as `snake_case`.
- `module_name` (optional): The name of the generated module.
  - default: The target handle as `snake_case`
- `extern_enums` (optional): A list of Enums for which an external type should be used.
  For those, code generation will be skipped. This is useful for large enums
  which can increase binary size, or for enums shared between multiple targets.
  Example: `extern_enums = ["LanguageCode"]`
    - default: `["LanguageCode", "CountryCode", "CurrencyCode"]`

### `src/lib.rs`

```rust
#[shopify_function_target(
    // Implicit target = "example.target-a"
    // Implicit generated module name = "target_a"
    query_path = "a.graphql",
    schema_path = "schema.graphql"
)]
fn target_a(
    _input: target_a::input::ResponseData,
) -> Result<target_a::output::FunctionTargetAResult> {
    Ok(target_a::output::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function_target(
    // Explicit target if function name does not match target handle
    target = "example.target-b",
    // Override generated module name
    module_name = "mod_b",
    query_path = "b.graphql",
    schema_path = "schema.graphql"
)]
fn function_b(input: mod_b::input::ResponseData) -> Result<mod_b::output::FunctionTargetBResult> {
    Ok(mod_b::output::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}
```

### Generated code

The `shopify_function_target` macro uses `generate_types` and `shopify_function` to generate a module (optionally using `module_name`) containing:

- `input`
  - `ResponseData` for the target-specific input query
- `output`
  - a target-specific result type, e.g. `FunctionTargetAResult`
- `export`: The function exported to the Wasm module using the Rust function name, which must match the export specified for the target in `shopify.function.extension.toml`

The generated types can be viewed using the instructions in the `shopify_function` crate `README`.

#### `*.output.graphql`

Each target will have an `.output.graphql` file prefixed with the target handle as `snake_case`. This file is used to generate the target-specific `output` types, and can be added to `.gitignore`.

Target handle (`snake_case`) | Output file | GraphQL mutation
-- | -- | --
`target_a` | `.target_a.output.graphql` | `mutation Output($result: FunctionTargetAResult!) { targetA(result: $result) }`
`target_b` | `.target_b.output.graphql` | `mutation Output($result: FunctionTargetBResult!) { targetB(result: $result) }`

If the Rust function name does not match the target handle as `snake_case`, the `target` argument must be provided to `shopify_function_target` to generate the `output` types.

[crate]: https://crates.io/crates/shopify-function
