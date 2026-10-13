//! Crate to write Shopify Functions in Rust.
//!
//! This crate provides some convenience when building Shopify Functions using
//! Rust. The crate takes care of generating the required Rust structs to handle
//! the data types being passed between Shopify and the Function. The crate also
//! takes care of deserializing the input data and serializing the output data.
//!
//! ```ignore
//! use shopify_function::prelude::*
//!
//! #[typegen("./schema.graphql")]
//! mod schema {
//!     #[query("./input.graphql")]
//!     pub mod input {}
//! }
//!
//! #[shopify_function]
//! fn run(input: schema::input::Input) -> Result<schema::FunctionRunResult> {
//!     /* ... */
//! }
//! ```

#[cfg(all(target_arch = "wasm32", target_os = "wasi", target_env = "p1"))]
compile_error!("Compiling to wasm32-wasip1 is unsupported, change your target to wasm32-unknown-unknown instead");

pub use shopify_function_macro::{shopify_function, typegen, Deserialize};

pub mod scalars;

pub mod prelude {
    #[allow(deprecated)]
    pub use crate::eprintln;
    pub use crate::log;
    pub use crate::scalars::*;
    pub use shopify_function_macro::{shopify_function, typegen, Deserialize};
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Runs the given function `f` with the invocation payload, returning the
/// deserialized output. This function is provided as a helper when writing
/// tests.
#[cfg(not(target_family = "wasm"))]
pub fn run_function_with_input<F, P: wasm_api::Deserialize, O>(f: F, payload: &str) -> Result<O>
where
    F: Fn(P) -> Result<O>,
{
    let parsed_json: serde_json::Value = serde_json::from_str(payload)?;
    let context = wasm_api::Context::new_with_input(parsed_json);
    let input = wasm_api::Deserialize::deserialize(&context.input_get().unwrap()).unwrap();
    f(input)
}

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        {
            use std::fmt::Write;
            let mut buf = std::string::String::new();
            writeln!(&mut buf, $($args)*).unwrap();
            $crate::wasm_api::Context.log(&buf);
        }
    };
}

#[macro_export]
#[deprecated(note = "Use log! instead")]
macro_rules! eprintln {
    ($($tt:tt)*) => { std::eprintln!($($tt)*) }
}

pub use shopify_function_wasm_api as wasm_api;

/// Each `compile_fail` case differs in one way from the first case, which compiles. Stable rustdoc
/// doesn't check error codes, so the first case is what keeps the others from failing for an
/// unrelated reason.
///
/// ```
/// # use shopify_function::prelude::*;
/// # #[typegen("tests/fixtures/prepare_schema.graphql", enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
/// #     "TargetPrepareResult.variables" => target_run::InputVariables,
/// # })]
/// # mod schema {
/// #     #[query("tests/fixtures/prepare_run.graphql")]
/// #     pub mod target_run {}
/// # }
/// # fn main() {}
/// fn prepare() -> schema::TargetPrepareResult {
///     schema::TargetPrepareResult {
///         variables: schema::target_run::InputVariables {
///             handle: "tier-a".to_string(),
///             first: Some(10),
///         },
///     }
/// }
/// ```
///
/// ```compile_fail,E0308
/// # use shopify_function::prelude::*;
/// # use shopify_function::scalars::JsonValue;
/// # use std::collections::BTreeMap;
/// # #[typegen("tests/fixtures/prepare_schema.graphql", enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
/// #     "TargetPrepareResult.variables" => target_run::InputVariables,
/// # })]
/// # mod schema {
/// #     #[query("tests/fixtures/prepare_run.graphql")]
/// #     pub mod target_run {}
/// # }
/// # fn main() {}
/// // A hand-written `JsonValue`.
/// fn prepare() -> schema::TargetPrepareResult {
///     schema::TargetPrepareResult {
///         variables: JsonValue::Object(BTreeMap::from([(
///             "handle".to_string(),
///             JsonValue::String("tier-a".to_string()),
///         )])),
///     }
/// }
/// ```
///
/// ```compile_fail,E0063
/// # use shopify_function::prelude::*;
/// # #[typegen("tests/fixtures/prepare_schema.graphql", enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
/// #     "TargetPrepareResult.variables" => target_run::InputVariables,
/// # })]
/// # mod schema {
/// #     #[query("tests/fixtures/prepare_run.graphql")]
/// #     pub mod target_run {}
/// # }
/// # fn main() {}
/// // `$handle` is declared but not set.
/// fn prepare() -> schema::TargetPrepareResult {
///     schema::TargetPrepareResult {
///         variables: schema::target_run::InputVariables { first: Some(10) },
///     }
/// }
/// ```
///
/// ```compile_fail,E0560
/// # use shopify_function::prelude::*;
/// # #[typegen("tests/fixtures/prepare_schema.graphql", enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
/// #     "TargetPrepareResult.variables" => target_run::InputVariables,
/// # })]
/// # mod schema {
/// #     #[query("tests/fixtures/prepare_run.graphql")]
/// #     pub mod target_run {}
/// # }
/// # fn main() {}
/// // `$tierHandle` is not what the query declares.
/// fn prepare() -> schema::TargetPrepareResult {
///     schema::TargetPrepareResult {
///         variables: schema::target_run::InputVariables {
///             tier_handle: "tier-a".to_string(),
///             first: Some(10),
///         },
///     }
/// }
/// ```
///
/// ```compile_fail,E0308
/// # use shopify_function::prelude::*;
/// # #[typegen("tests/fixtures/prepare_schema.graphql", enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
/// #     "TargetPrepareResult.variables" => target_run::InputVariables,
/// # })]
/// # mod schema {
/// #     #[query("tests/fixtures/prepare_run.graphql")]
/// #     pub mod target_run {}
/// # }
/// # fn main() {}
/// // `$first` is an `Int`.
/// fn prepare() -> schema::TargetPrepareResult {
///     schema::TargetPrepareResult {
///         variables: schema::target_run::InputVariables {
///             handle: "tier-a".to_string(),
///             first: Some("10".to_string()),
///         },
///     }
/// }
/// ```
#[cfg(doctest)]
pub struct PrepareVariablesDoctests;

#[cfg(test)]
mod tests {}
