use super::api_info::{ApiInfo, Function, ParamType};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

fn parse_ident(ident: &str) -> Ident {
    syn::parse_str::<Ident>(ident).unwrap_or_else(|_| format_ident!("r#{ident}"))
}

fn resolve_type(typ: &ParamType) -> Option<TokenStream> {
    Some(match typ {
        ParamType::Void => quote! { () },
        ParamType::Boolean => quote! { sys::types::Bool },
        ParamType::Integer => quote! { sys::types::Int },
        ParamType::Float => quote! { sys::types::Float },
        ParamType::String => quote! { sys::types::String },
        ParamType::Array => quote! { sys::types::Array },
        ParamType::Dictionary => quote! { sys::types::Dict },
        ParamType::Object => quote! { sys::types::Object },
        ParamType::Ext(name) => {
            let id = format_ident!("{name}");
            quote! { sys::ext_types::#id }
        }
        ParamType::ArrayOf(typ, len) => {
            let typ = resolve_type(typ)?;
            match len {
                Some(len) => quote! { sys::types::FixedArrayOf<#typ, #len> },
                None => quote! { sys::types::ArrayOf<#typ> },
            }
        }
        ParamType::Ref => return None,
    })
}

fn rpc_function(func: &Function) -> bool {
    resolve_type(&func.return_type).is_some()
        && func
            .parameters
            .0
            .iter()
            .all(|param| resolve_type(&param.param_type).is_some())
}

fn gen_method(func: &Function, method: &str) -> TokenStream {
    let id = func.to_id();
    let method = parse_ident(method);
    let doc = func.docs("Build a call to");
    let deprecated = func.deprecation();
    let args: Vec<_> = func
        .parameters
        .0
        .iter()
        .skip(1)
        .map(|param| parse_ident(&param.ident))
        .collect();
    let generics: Vec<_> = (0..args.len()).map(|n| format_ident!("A{n}")).collect();
    let bounds = func.parameters.0.iter().skip(1).map(|param| {
        resolve_type(&param.param_type).expect("RPC parameters must have a wire type")
    });
    let (generic_params, where_clause) = if generics.is_empty() {
        (TokenStream::new(), TokenStream::new())
    } else {
        (
            quote! { <#(#generics),*> },
            quote! { where #(#generics: Like<#bounds>),* },
        )
    };

    quote! {
        #(#[doc = #doc])*
        #deprecated
        pub fn #method #generic_params (self, #(#args: #generics),*)
            -> crate::Call<sys::functions::#id, (Self, #(#generics,)*)>
        #where_clause
        {
            crate::Call::new((self, #(#args,)*))
        }
    }
}

fn gen_ui_options(options: &[String]) -> TokenStream {
    let count = options.len();
    let encoded = quote! { mp::encode::primitive::Map<[(&'static str, bool); #count]> };
    let fields = options.iter().map(|key| {
        let field = format_ident!("{key}");
        quote! { (#key, options.#field) }
    });
    let impls = [quote! { sys::types::Dict }, quote! { sys::types::Object }]
        .into_iter()
        .map(|typ| {
            quote! {
                impl Like<#typ> for sys::UiOptions {
                    type EncodeAs = #encoded;

                    fn encode(self) -> Self::EncodeAs {
                        encode_ui_options(&self)
                    }
                }

                impl Like<#typ> for &sys::UiOptions {
                    type EncodeAs = #encoded;

                    fn encode(self) -> Self::EncodeAs {
                        encode_ui_options(self)
                    }
                }
            }
        });

    quote! {
        fn encode_ui_options(options: &sys::UiOptions) -> #encoded {
            mp::encode::primitive::Map([#(#fields,)*])
        }

        #(#impls)*
    }
}

fn toplevel_comment(api_info: &ApiInfo) -> TokenStream {
    let major = api_info.version.major;
    let minor = api_info.version.minor;
    let patch = api_info.version.patch;

    let comment = format!(" Generated from Neovim {major}.{minor}.{patch} API metadata.");
    quote! { #![doc = #comment] }
}

pub(super) fn gen_rs(api_info: &ApiInfo) -> TokenStream {
    let comment = toplevel_comment(api_info);
    let functions = api_info
        .functions
        .iter()
        .filter(|func| rpc_function(func))
        .map(Function::to_id);
    let ui_options = gen_ui_options(&api_info.ui_options);

    let handles = api_info.types.0.iter().map(|(name, typ)| {
        let id = format_ident!("{}", name.0);
        let methods = api_info.functions.iter().filter_map(|func| {
            if !func.method || !rpc_function(func) {
                return None;
            }
            let receiver = func.parameters.0.first()?;
            if !matches!(&receiver.param_type, ParamType::Ext(on) if on == &name.0) {
                return None;
            }
            let method = func.name.strip_prefix(&typ.prefix)?;
            Some(gen_method(func, method))
        });
        quote! {
            impl crate::handles::#id {
                #(#methods)*
            }
        }
    });

    quote! {
        #comment
        #![allow(deprecated)]

        use crate::api_types::encode::Like;
        use crate::sys;

        /// Neovim API function descriptors for typed requests and notifications.
        ///
        /// Select a descriptor and pass its arguments as a tuple to
        /// [`crate::Neovim::request`] or [`crate::Neovim::notify`]:
        ///
        /// ```
        /// use nvim_api::{api, Buffer, Neovim};
        ///
        /// let lines = ["first", "second"];
        /// let mut nvim = Neovim::new(Vec::new());
        /// let pending = nvim.request::<api::NvimBufSetLines>((
        ///     Buffer::CURRENT, 0i64, -1i64, true, &lines[..],
        /// )).unwrap();
        /// assert_eq!(pending.id().0, 0);
        /// ```
        ///
        /// The async equivalents are [`crate::Neovim::request_async`] and
        /// [`crate::Neovim::notify_async`]. Handle methods construct [`crate::Call`]
        /// values for manual encoding. Lua-only functions requiring a `LuaRef`
        /// are omitted from this module.
        pub mod api {
            pub use crate::sys::functions::{#(#functions),*};
        }

        #(#handles)*
        #ui_options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_info::*;

    fn function(name: &str, params: &[(&str, &str)], method: bool) -> Function {
        Function {
            name: name.into(),
            parameters: Params(
                params
                    .iter()
                    .map(|(typ, ident)| Param {
                        param_type: ParamType::from(*typ),
                        ident: (*ident).into(),
                    })
                    .collect(),
            ),
            since: 1,
            deprecated_since: None,
            return_type: ParamType::Void,
            method,
        }
    }

    fn api_info(functions: Vec<Function>) -> ApiInfo {
        ApiInfo {
            version: Version {
                major: 0,
                minor: 12,
                patch: 6,
            },
            api: ApiVersion {
                api_level: 14,
                api_compatible: 0,
                api_prerelease: false,
            },
            functions,
            ui_events: Vec::new(),
            ui_options: vec!["rgb".into(), "ext_linegrid".into()],
            error_types: ErrTypes(Default::default()),
            types: ExtTypes(
                [(
                    TypeName("Buffer".into()),
                    ExtType {
                        id: 0,
                        prefix: "nvim_buf_".into(),
                    },
                )]
                .into(),
            ),
        }
    }

    #[test]
    fn typed_descriptors_and_handle_builders() {
        let info = api_info(vec![
            function("nvim_get_mode", &[], false),
            function("nvim_command", &[("String", "command")], false),
            function(
                "nvim_buf_set_lines",
                &[("Buffer", "buf"), ("ArrayOf(String)", "replacement")],
                true,
            ),
            function(
                "nvim_buf_set_mark",
                &[("Buffer", "buf"), ("ArrayOf(Integer, 2)", "pos")],
                true,
            ),
        ]);
        let file = syn::parse2::<syn::File>(gen_rs(&info)).unwrap();
        let generated = prettyplease::unparse(&file);
        assert!(generated.contains("pub mod api"));
        assert!(generated.contains("pub use crate::sys::functions::{"));
        for id in [
            "NvimGetMode",
            "NvimCommand",
            "NvimBufSetLines",
            "NvimBufSetMark",
        ] {
            assert!(generated.contains(id));
        }
        assert!(!generated.contains("pub fn nvim_"));
        assert!(generated.contains("A0: Like<sys::types::ArrayOf<sys::types::String>>"));
        assert!(generated.contains("A0: Like<sys::types::FixedArrayOf<sys::types::Int, 2usize>>"));
        assert!(generated.contains("impl crate::handles::Buffer"));
        assert!(generated.contains("pub fn set_lines<A0>("));
        assert!(generated.contains("crate::Call::new((self, replacement))"));
        assert!(generated.contains("crate::Call<sys::functions::NvimBufSetLines, (Self, A0)>"));
    }

    #[test]
    fn omit_lua_functions_keep_legacy_rpc_aliases() {
        let mut lua_return = function("lua_return", &[], false);
        lua_return.return_type = ParamType::Ref;
        let info = api_info(vec![
            function(
                "nvim_buf_call",
                &[("Buffer", "buf"), ("LuaRef", "fun")],
                true,
            ),
            function("lua_array", &[("ArrayOf(LuaRef)", "functions")], false),
            lua_return,
            function("buffer_line_count", &[("Buffer", "buf")], true),
        ]);
        let generated = gen_rs(&info).to_string();
        assert!(!generated.contains("NvimBufCall"));
        assert!(!generated.contains("LuaArray"));
        assert!(!generated.contains("LuaReturn"));
        assert!(generated.contains("functions :: { BufferLineCount }"));
        assert!(!generated.contains("fn line_count"));
    }

    #[test]
    fn ui_options_encode_owned_and_borrowed_without_allocation() {
        let info = api_info(Vec::new());
        let file = syn::parse2::<syn::File>(gen_rs(&info)).unwrap();
        let generated = prettyplease::unparse(&file);
        for typ in ["Dict", "Object"] {
            assert!(
                generated.contains(&format!("impl Like<sys::types::{typ}> for sys::UiOptions"))
            );
            assert!(
                generated.contains(&format!("impl Like<sys::types::{typ}> for &sys::UiOptions"))
            );
        }
        assert!(generated.contains("Map<[(&'static str, bool); 2usize]>"));
        assert!(generated.contains("(\"rgb\", options.rgb)"));
        assert!(generated.contains("(\"ext_linegrid\", options.ext_linegrid)"));
    }
}
