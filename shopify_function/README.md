# Shopify Functions Rust Crate

A crate to help developers build [Shopify Functions].

## Dependencies

* Make sure you have `graphql_client` in your dependencies

    ```
    cargo add graphql_client@0.13.0
    ```

## Usage

* The [`generate_types`] macro allows you to generate structs based on your [input query]. It will also generate output/response types for the current Function API, based on the provided schema.
    * It will automatically generate an `.output.graphql` file for code generation purposes. This file can be added to your `.gitignore`.
* The [`shopify_function`] attribute macro marks the following function as the entry point for a Shopify Function. It manages the Functions `STDIN` input parsing and `STDOUT` output serialization for you.
* The [`run_function_with_input`] function is a utility for unit testing which allows you to quickly add new tests based on a given JSON input string.

See the [example] for details on usage, or use the following guide to convert an existing Rust-based function.

## Updating an existing function to use `shopify_function`

1. `cargo add shopify_function`
1. `cargo add graphql_client@0.13.0`
1. Delete `src/api.rs`.
1. In `main.rs`:
    1. Add imports for `shopify_function`.

        ```rust
        use shopify_function::prelude::*;
        use shopify_function::Result;
        ```

    1. Remove references to `mod api`.
    1. Add type generation, right under your imports.

        ```rust
        generate_types!(query_path = "./input.graphql", schema_path = "./schema.graphql");
        ```

    1. Remove the `main` function entirely.
    1. Attribute the `function` function with the `shopify_function` macro, and change its return type.

        ```rust
        #[shopify_function]
        fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
        ```

    1. Update the types and fields utilized in the function to the new, auto-generated structs. For example:
        | Old | New |
        | --- | --- |
        | `input::Input` | `input::ResponseData` |
        | `input::Metafield` | `input::InputDiscountNodeMetafield` |
        | `input::DiscountNode` | `input::InputDiscountNode` |
        | `FunctionResult` | `output::FunctionResult` |
        | `DiscountApplicationStrategy::First` | `output::DiscountApplicationStrategy::FIRST` |

1. Add `.output.graphql` to your `.gitignore`.

---
License Apache-2.0

[Shopify Functions]: https://shopify.dev/api/functions
[`generate_types`]: https://docs.rs/shopify_function/latest/shopify_function/macro.generate_types.html
[input query]: https://shopify.dev/api/functions/input-output#input
[`shopify_function`]: https://docs.rs/shopify_function/latest/shopify_function/attr.shopify_function.html
[`run_function_with_input`]: https://docs.rs/shopify_function/latest/shopify_function/fn.run_function_with_input.html
[example]: https://github.com/Shopify/shopify-function-rust/tree/main/example
