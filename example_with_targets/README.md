# Shopify Rust function example with targets

This is an example of how to use the [Shopify Function Rust crate][crate] with the `shopify_function` macro to write a Shopify Function with multiple API targets.

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

## Usage

### `src/lib.rs`

```rust
#[typegen("schema.graphql")]
mod schema {
  #[query("a.graphql")]
  pub mod a {}

  #[query("b.graphql")]
  pub mod b {}
}

#[shopify_function]
fn target_a(
    _input: schema::a::Input,
) -> Result<schema::FunctionTargetAResult> {
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function]
fn function_b(input: schema::b::Input) -> Result<schema::FunctionTargetBResult> {
    Ok(schema::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}
```

### Generated code

The `typegen` macro adds the following to the existing module that it decorates:

- at the top level:
  - definitions for all input types, scalar types, custom scalar types, and enum types
- in the nested modules for queries
  - struct for the operation types in the query (e.g. `Input` for `query Input { ... }`)
  - nested modules for the nested query types

The `shopify_function` macro adds the following next to the existing function:
- `<existing_function_name>_export`: The function exported to the Wasm module using the Rust function name, which must match the export specified for the target in `shopify.function.extension.toml`

The generated types can be viewed using the instructions in the `shopify_function` crate `README`.

[crate]: https://crates.io/crates/shopify-function
