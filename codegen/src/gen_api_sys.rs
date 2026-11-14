use super::api_info::*;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use std::fmt::Display;

fn to_pascal_case(name: &str) -> String {
    use convert_case::{Case, Casing as _};
    name.to_case(Case::Pascal)
}

impl Function {
    pub(super) fn to_id(&self) -> Ident {
        let id = to_pascal_case(&self.name);
        format_ident!("{id}")
    }

    pub(super) fn doc_since(&self) -> String {
        let since = self.since;
        if let Some(deprecated) = self.deprecated_since {
            format!(" Since Neovim API level {since}; deprecated since {deprecated}.")
        } else {
            format!(" Since Neovim API level {since}.")
        }
    }

    pub(super) fn docs(&self, summary: &str) -> [String; 3] {
        let name = &self.name;
        [
            format!(" {summary} [`{name}()`](https://neovim.io/doc/user/api/#{name}())."),
            String::new(),
            self.doc_since(),
        ]
    }

    pub(super) fn deprecation(&self) -> Option<TokenStream> {
        self.deprecated_since.map(|level| {
            let note = format!("Deprecated since Neovim API level {level}");
            quote! { #[deprecated(note = #note)] }
        })
    }
}
impl UiEvent {
    pub(super) fn to_id(&self) -> Ident {
        let id = to_pascal_case(&self.name);
        format_ident!("{id}")
    }

    pub(super) fn doc_since(&self) -> String {
        let since = self.since;
        format!(" Since Neovim API level {since}.")
    }
}

fn assert_types_implemented<'a>(
    types: impl Iterator<Item = &'a Param>,
    resolver: impl for<'t> Fn(&'t ParamType) -> Option<TokenStream>,
    message: impl Display,
) {
    for param in types {
        if resolver(&param.param_type).is_none() {
            unimplemented!(
                "ERROR: {message}: unknown type {}({:?})",
                param.ident,
                param.param_type
            );
        }
    }
}

fn parse_ident(ident: &str) -> Ident {
    syn::parse_str::<Ident>(ident).unwrap_or_else(|_| format_ident!("r#{ident}"))
}

fn find_ext<'a>(ext_types: &'a ExtTypes, name: &str) -> Option<&'a ExtType> {
    ext_types.0.get(&to_pascal_case(name))
}

fn gen_funcs(funcs: &[Function], ext_types: &ExtTypes) -> TokenStream {
    fn resolve_type(typ: &ParamType) -> Option<TokenStream> {
        match typ {
            ParamType::Void => Some(quote! { () }),
            ParamType::Boolean => Some(quote! { types::Bool }),
            ParamType::Integer => Some(quote! { types::Int }),
            ParamType::Float => Some(quote! { types::Float }),
            ParamType::String => Some(quote! { types::String }),
            ParamType::Array => Some(quote! { types::Array }),
            ParamType::Dictionary => Some(quote! { types::Dict }),
            ParamType::Object => Some(quote! { types::Object }),
            ParamType::Ext(ext) => {
                let id = format_ident!("{ext}");
                Some(quote! { ext_types :: #id })
            }
            ParamType::ArrayOf(param, Some(len)) => {
                let typ = resolve_type(param)?;
                Some(quote! { types::FixedArrayOf < #typ, #len > })
            }
            ParamType::ArrayOf(param, None) => {
                let typ = resolve_type(param)?;
                Some(quote! { types::ArrayOf < #typ > })
            }
            ParamType::Ref => Some(quote! { types::Ref }),
        }
    }

    fn fn_to_tokens(func: &Function) -> Option<TokenStream> {
        let name = &func.name;
        let id = func.to_id();

        assert_types_implemented(
            func.parameters.0.iter(),
            resolve_type,
            "fn_to_tokens failed",
        );

        let fields = func.parameters.0.iter().map(|param| {
            let typ = resolve_type(&param.param_type).unwrap();
            let field = parse_ident(&param.ident);
            quote! { pub #field: #typ }
        });

        let params = func
            .parameters
            .0
            .iter()
            .map(|param| resolve_type(&param.param_type).unwrap());

        let Some(ret) = resolve_type(&func.return_type) else {
            unimplemented!(
                "ERROR: unknown return type in functions: {:?}",
                func.return_type
            );
        };

        let doc = func.docs("Signature of");
        let deprecated = func.deprecation();

        Some(quote! {
            #(#[doc = #doc])*
            #deprecated
            pub struct #id {
                #( #fields, )*
            }

            impl types::ApiFn for #id {
                type Params = ( #( #params, )* );
                type Return = #ret;
                const NAME: &str = #name;
            }
        })
    }

    fn method_to_tokens(func: &Function, ext_types: &ExtTypes) -> Option<TokenStream> {
        let name = &func.name;
        let id = func.to_id();

        assert_types_implemented(
            func.parameters.0.iter(),
            resolve_type,
            "method_to_tokens failed",
        );

        let (on, rest) = func
            .parameters
            .0
            .split_first()
            .expect("method should have non-empty parameters");

        let ParamType::Ext(ext_name) = &on.param_type else {
            unimplemented!("unknown method receiver type: {:?}", on.param_type);
        };
        let Some(ext_type) = find_ext(ext_types, ext_name) else {
            unimplemented!("unknown method receiver type: {ext_name}");
        };
        let on = resolve_type(&on.param_type).unwrap();

        let Some(method_name) = name.strip_prefix(&ext_type.prefix) else {
            print!(
                "Warning: method name does not start with \"{}\": {}",
                ext_type.prefix, name
            );
            if let Some(deprecated) = func.deprecated_since {
                println!(" (deprecated since {deprecated})");
            } else {
                println!();
            }
            return None;
        };

        let fields = rest.iter().map(|param| {
            let typ = resolve_type(&param.param_type).unwrap();
            let field = parse_ident(&param.ident);
            quote! { pub #field: #typ }
        });

        let params = rest
            .iter()
            .map(|param| resolve_type(&param.param_type).unwrap());
        let params_clone = params.clone();

        let Some(ret) = resolve_type(&func.return_type) else {
            unimplemented!(
                "ERROR: unknown return type in functions: {:?}",
                func.return_type
            );
        };

        let doc = func.docs("Signature of");
        let deprecated = func.deprecation();

        Some(quote! {
            #(#[doc = #doc])*
            #deprecated
            pub struct #id {
                #( #fields, )*
            }

            impl types::ApiFn for #id {
                type Params = ( #on, #( #params, )* );
                type Return = #ret;
                const NAME: &str = #name;
            }
            impl types::ApiMethod for #id {
                type On = #on;
                type Params = ( #( #params_clone, )* );
                type Return = #ret;
                const NAME: &str = #method_name;
            }
        })
    }

    let funcs = funcs.iter().filter_map(|func| {
        if func.method {
            method_to_tokens(func, ext_types).or_else(|| fn_to_tokens(func))
        } else {
            fn_to_tokens(func)
        }
    });
    quote! {
        /// Function descriptors implementing [`crate::types::ApiFn`].
        ///
        /// Descriptors for handle methods also implement [`crate::types::ApiMethod`],
        /// separating the receiver from the remaining parameters.
        pub mod functions {
            use crate::types;
            use super::ext_types;

            #( #funcs )*
        }
    }
}

fn gen_ui_events(ui_events: &[UiEvent]) -> TokenStream {
    fn resolve_type(typ: &ParamType) -> Option<TokenStream> {
        match typ {
            ParamType::Boolean => Some(quote! { types::Bool }),
            ParamType::Integer => Some(quote! { types::Int }),
            ParamType::Float => Some(quote! { types::Float }),
            ParamType::String => Some(quote! { types::String }),
            ParamType::Array => Some(quote! { types::Array }),
            ParamType::Dictionary => Some(quote! { types::Dict }),
            ParamType::Object => Some(quote! { types::Object }),
            ParamType::Ext(ext) => {
                let id = format_ident!("{ext}");
                Some(quote! { ext_types :: #id })
            }
            _ => None,
        }
    }

    let events = ui_events.iter().map(|ev| {
        let name = &ev.name;
        let id = ev.to_id();

        assert_types_implemented(ev.parameters.0.iter(), resolve_type, "ui_events failed");

        let fields = ev.parameters.0.iter().map(|param| {
            let typ = resolve_type(&param.param_type).unwrap();
            let field = parse_ident(&param.ident);
            quote! { pub #field: #typ }
        });

        let params = ev
            .parameters
            .0
            .iter()
            .map(|param| resolve_type(&param.param_type).unwrap());

        let doc = [
            format!(" Argument markers for the `{name}` UI event."),
            String::new(),
            ev.doc_since(),
        ];

        quote! {
            #(#[doc = #doc])*
            pub struct #id {
                #( #fields, )*
            }

            impl types::UiEvent for #id {
                type Params = ( #( #params, )* );
                const NAME: &str = #name;
            }
        }
    });
    quote! {
        /// UI event descriptors implementing [`crate::types::UiEvent`].
        ///
        /// `GridResize`, for example, names the `grid_resize` event and describes
        /// its `(grid, width, height)` arguments as integer type markers.
        pub mod ui_events {
            use crate::types;
            use super::ext_types;

            #( #events )*
        }
    }
}

fn gen_ui_opts(ui_opts: &[String]) -> TokenStream {
    let fields = ui_opts.iter().map(|key| {
        let name = format_ident!("{key}");
        let doc = format!(
            " Neovim's [`{key}` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option)."
        );
        quote! {
            #[doc = #doc]
            pub #name: bool
        }
    });
    let defaults = ui_opts.iter().map(|key| {
        let name = format_ident!("{key}");
        let value = key == "rgb";
        quote! { #name: #value }
    });
    quote! {
        /// UI capabilities requested when attaching to Neovim.
        ///
        /// The default enables RGB colors and leaves extension options disabled.
        /// Enable only the extensions your UI implements. For a line-grid UI:
        ///
        /// ```
        /// use nvim_api_sys::UiOptions;
        ///
        /// let options = UiOptions {
        ///     ext_linegrid: true,
        ///     ..UiOptions::default()
        /// };
        /// assert!(options.rgb);
        /// assert!(!options.ext_multigrid);
        /// ```
        ///
        /// See the [Neovim UI protocol](https://neovim.io/doc/user/api-ui-events/).
        #[derive(Debug, Clone, Copy)]
        pub struct UiOptions {
            #( #fields, )*
        }

        impl Default for UiOptions {
            fn default() -> Self {
                Self {
                    #(#defaults,)*
                }
            }
        }
    }
}

fn gen_err_types(err_types: &ErrTypes) -> TokenStream {
    let type_defs = err_types.0.iter().map(|(TypeName(name), ext)| {
        let doc = format!(" The `{name}` error category in Neovim's API metadata.");
        let name = format_ident!("{name}");
        let ext = ext.id;
        quote! {
            #[doc = #doc]
            pub struct #name;

            impl #name {
                /// The error category's numeric ID in the API metadata.
                pub const ID: i64 = #ext ;
            }
        }
    });
    quote! {
        /// Error category IDs reported by Neovim's API metadata.
        pub mod err_types {
            #( #type_defs )*
        }
    }
}

fn gen_ext_types(ext_types: &ExtTypes) -> TokenStream {
    let type_defs = ext_types.0.iter().map(|(TypeName(name), ext)| {
        let doc = format!(" MessagePack extension marker for Neovim's `{name}` handles.");
        let name = format_ident!("{name}");
        let ext = ext.id;
        quote! {
            #[doc = #doc]
            pub struct #name;

            impl #name {
                /// The MessagePack extension tag assigned by Neovim.
                pub const ID: i64 = #ext ;
            }
        }
    });
    quote! {
        /// Type markers and MessagePack extension tags for Neovim handles.
        ///
        /// For example, [`Buffer::ID`](crate::ext_types::Buffer::ID) is the tag on an encoded buffer handle.
        /// These marker types do not store runtime handle values.
        pub mod ext_types {
            #( #type_defs )*
        }
    }
}

fn gen_version(ver: &Version) -> TokenStream {
    let major = ver.major as u16;
    let minor = ver.minor as u16;
    let patch = ver.patch as u16;

    quote! {
        /// Neovim version used to generate these descriptors, as `(major, minor, patch)`.
        pub const VERSION: (u16, u16, u16) = ( #major, #minor, #patch );
    }
}

fn gen_api_version(ver: &ApiVersion) -> TokenStream {
    let level = ver.api_level;
    let compatible = ver.api_compatible;
    let prerelease = ver.api_prerelease;

    quote! {
        /// API level reported by the Neovim used for generation.
        pub const API_LEVEL: i64 = #level;
        /// Oldest API level supported by the Neovim used for generation.
        pub const API_COMPATIBLE: i64 = #compatible;
        /// Whether the generated API level was marked as a prerelease.
        pub const API_PRERELEASE: bool = #prerelease;
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
    let funcs = gen_funcs(&api_info.functions, &api_info.types);
    let ui_events = gen_ui_events(&api_info.ui_events);
    let ui_opts = gen_ui_opts(&api_info.ui_options);
    let err_types = gen_err_types(&api_info.error_types);
    let ext_types = gen_ext_types(&api_info.types);
    let ver = gen_version(&api_info.version);
    let api_ver = gen_api_version(&api_info.api);

    let comment = toplevel_comment(api_info);

    quote! {
        #comment
        #![allow(deprecated)]

        #ver
        #api_ver
        #funcs
        #ui_events
        #ui_opts
        #err_types
        #ext_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_receiver_uses_declared_type() {
        let api_info = ApiInfo {
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
            functions: ["nvim_buf_line_count", "buffer_line_count"]
                .into_iter()
                .map(|name| Function {
                    name: name.into(),
                    parameters: Params(vec![Param {
                        param_type: ParamType::Ext("Buffer".into()),
                        ident: "buf".into(),
                    }]),
                    since: 1,
                    deprecated_since: (name == "buffer_line_count").then_some(1),
                    return_type: ParamType::Integer,
                    method: true,
                })
                .collect(),
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
        };

        let tokens = gen_rs(&api_info);
        let file: syn::File = syn::parse2(tokens).unwrap();
        let generated = prettyplease::unparse(&file);
        assert!(generated.contains("impl types::ApiMethod for NvimBufLineCount"));
        assert!(generated.contains("const NAME: &str = \"nvim_buf_line_count\""));
        assert!(generated.contains("impl types::ApiFn for BufferLineCount"));
        assert!(
            generated.contains("#[deprecated(note = \"Deprecated since Neovim API level 1\")]")
        );
        assert_eq!(generated.matches("#[deprecated(").count(), 1);
        assert!(!generated.contains("impl types::ApiMethod for BufferLineCount"));
        assert!(generated.contains("pub const API_LEVEL: i64 = 14i64;"));
        assert!(generated.contains("#[derive(Debug, Clone, Copy)]"));
        assert!(generated.contains("rgb: true"));
        assert!(generated.contains("ext_linegrid: false"));
    }
}
