//! Generated descriptions of Neovim API functions, UI events, and types.
//!
//! A generated function type records its API name, parameter markers, and return
//! marker. For example, [`functions::NvimGetCurrentLine`] returns the Neovim
//! String type; choosing `&str`, `String`, or bytes to hold that result belongs
//! to the caller or to the `nvim-rpc` crate.
//!
//! `nvim-rpc` reexports RPC function descriptors through `nvim_rpc::api` for
//! typed requests and notifications. This crate supplies metadata only; it does
//! not encode values or communicate with Neovim.
//!
//! # Inspect a function signature
//!
//! ```
//! use nvim_rpc_sys::{functions::NvimGetCurrentLine, types::{self, ApiFn}};
//!
//! assert_eq!(NvimGetCurrentLine::NAME, "nvim_get_current_line");
//!
//! fn string_result<F: ApiFn<Params = (), Return = types::String>>() {}
//! string_result::<NvimGetCurrentLine>();
//! ```
//!
//! [`types::String`] describes the API type. It does not contain the returned
//! text. Similarly, [`ext_types::Buffer`] describes a buffer parameter; runtime
//! buffer IDs are represented by the `Buffer` handle in `nvim-rpc`.
//!
//! # Functions and handle methods
//!
//! Buffer methods have both a full API signature and a method signature. The
//! latter separates the receiver from the remaining parameters:
//!
//! ```
//! use nvim_rpc_sys::{ext_types, functions::NvimBufGetLines};
//! use nvim_rpc_sys::types::{self, ApiFn, ApiMethod};
//!
//! assert_eq!(<NvimBufGetLines as ApiFn>::NAME, "nvim_buf_get_lines");
//! assert_eq!(<NvimBufGetLines as ApiMethod>::NAME, "get_lines");
//!
//! fn buffer_method<M>()
//! where
//!     M: ApiMethod<
//!         On = ext_types::Buffer,
//!         Params = (types::Int, types::Int, types::Bool),
//!         Return = types::ArrayOf<types::String>,
//!     >,
//! {}
//! buffer_method::<NvimBufGetLines>();
//! ```
//!
//! # UI options and generated API version
//!
//! [`UiOptions`] holds actual option values. Enable only capabilities your UI
//! implements:
//!
//! ```
//! use nvim_rpc_sys::{UiOptions, VERSION, API_LEVEL};
//!
//! let options = UiOptions {
//!     ext_linegrid: true,
//!     ..UiOptions::default()
//! };
//! assert!(options.rgb);
//! assert!(options.ext_linegrid);
//! assert!(!options.ext_multigrid);
//!
//! let (major, minor, patch) = VERSION;
//! println!("Generated from Neovim {major}.{minor}.{patch}, API level {API_LEVEL}");
//! ```
//!
//! The version constants describe the metadata used to generate this crate;
//! they do not query a running Neovim instance. Building this crate uses the
//! checked-in source and does not run Neovim or the generator.

pub mod types;

mod generated;
pub use generated::*;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_version_matches_generated_neovim_version() {
        let (major, minor, patch) = crate::VERSION;
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            format!("{major}.{minor}.{patch}")
        );
    }
}
