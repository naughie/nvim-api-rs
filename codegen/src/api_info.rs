use super::Error;

use serde::Deserialize;

use anyhow::anyhow;

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::ffi::OsStr;

#[derive(Debug, Deserialize, PartialEq)]
pub(super) struct Version {
    pub(super) major: i64,
    pub(super) minor: i64,
    pub(super) patch: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct ApiVersion {
    pub(super) api_level: i64,
    pub(super) api_compatible: i64,
    pub(super) api_prerelease: bool,
}

#[derive(Debug, Deserialize)]
#[serde(from = "&str")]
pub(super) enum ParamType {
    Void,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    ArrayOf(Box<ParamType>, Option<usize>),
    Object,
    Dictionary,
    Ext(String),
    /// Reference to a Lua function. `vim.api` only
    Ref,
}

impl From<&str> for ParamType {
    fn from(s: &str) -> Self {
        match s {
            "void" => Self::Void,
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Float" => Self::Float,
            "String" => Self::String,
            "Array" => Self::Array,
            "Object" => Self::Object,
            "Dictionary" | "Dict" => Self::Dictionary,
            "LuaRef" => Self::Ref,
            _ => {
                if let Some(s) = s.strip_prefix("ArrayOf(")
                    && let Some(s) = s.strip_suffix(')')
                {
                    let (typ, len) = if let Some((typ, len)) = s.split_once(", ") {
                        (Self::from(typ), len.parse().ok())
                    } else {
                        (Self::from(s), None)
                    };
                    Self::ArrayOf(Box::new(typ), len)
                } else {
                    if !matches!(s, "Window" | "Buffer" | "Tabpage") {
                        eprintln!("unknown API type: {s}");
                    }
                    Self::Ext(s.to_owned())
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Param {
    pub(super) param_type: ParamType,
    pub(super) ident: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct Params(pub Vec<Param>);

#[derive(Debug, Deserialize)]
pub(super) struct Function {
    pub(super) name: String,
    pub(super) parameters: Params,
    pub(super) since: i64,
    #[serde(default)]
    pub(super) deprecated_since: Option<i64>,
    pub(super) return_type: ParamType,
    pub(super) method: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct UiEvent {
    pub(super) name: String,
    pub(super) parameters: Params,
    pub(super) since: i64,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TypeName(pub String);

impl<T> Borrow<T> for TypeName
where
    String: Borrow<T>,
    T: ?Sized,
{
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ErrType {
    pub(super) id: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct ErrTypes(pub BTreeMap<TypeName, ErrType>);

#[derive(Debug, Deserialize)]
pub(super) struct ExtType {
    pub(super) id: i64,
    pub(super) prefix: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ExtTypes(pub BTreeMap<TypeName, ExtType>);

#[derive(Debug)]
pub(super) struct ApiInfo {
    pub(super) version: Version,
    pub(super) api: ApiVersion,
    pub(super) functions: Vec<Function>,
    pub(super) ui_events: Vec<UiEvent>,
    pub(super) ui_options: Vec<String>,
    pub(super) error_types: ErrTypes,
    pub(super) types: ExtTypes,
}

impl<'de> Deserialize<'de> for ApiInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct VersionInfo {
            #[serde(flatten)]
            version: Version,
            #[serde(flatten)]
            api: ApiVersion,
        }

        #[derive(Deserialize)]
        struct Info {
            version: VersionInfo,
            functions: Vec<Function>,
            ui_events: Vec<UiEvent>,
            ui_options: Vec<String>,
            error_types: ErrTypes,
            types: ExtTypes,
        }

        let info = Info::deserialize(deserializer)?;
        Ok(Self {
            version: info.version.version,
            api: info.version.api,
            functions: info.functions,
            ui_events: info.ui_events,
            ui_options: info.ui_options,
            error_types: info.error_types,
            types: info.types,
        })
    }
}

impl ApiInfo {
    pub(super) async fn from_prog(prog: impl AsRef<OsStr>) -> Result<Self, Error> {
        use tokio::process::Command;

        use std::process::Stdio;

        let mut cmd = Command::new(&prog);
        cmd.arg("--api-info")
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdin(Stdio::null());
        println!("{cmd:?}");

        let output = cmd.output().await?;
        if output.status.success() {
            Self::parse(&output.stdout)
        } else {
            Err(anyhow!(
                "{} --api-info failed: {}",
                prog.as_ref().display(),
                output.status
            )
            .into())
        }
    }

    fn parse(info: &[u8]) -> Result<Self, Error> {
        Ok(rmp_serde::from_slice(info)?)
    }
}
