# Shopify Rust function example with targets

This is an example of how to use the [Shopify Function Rust crate][crate] with the `shopify_function` macro to write a Shopify Function with multiple API targets.

## Example API

Target | Handle (`snake_case`) | GraphQL mutation
-- | -- | --
`example.target-a` | `target_a` | `targetA(result: FunctionTargetAResult!)`
`example.target-b` | `target_b` | `targetB(result: FunctionTargetBResult!)`

### `schema.graphql`

See the contents of `schema.graphql`

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

## `shopify_function` usage

See `src/main.rs`.

### Generated code

The `typegen` macro modifies a module to add all types for the schema, as well as any queries added within the module. See the the top-level `README.md` for details.

[crate]: https://crates.io/crates/shopify-function
