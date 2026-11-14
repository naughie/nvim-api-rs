const NVIM_API: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/nvim-api");
const NVIM_API_SYS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/nvim-api-sys");
const WORKSPACE_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml");

mod install;

mod api_info;
use api_info::{ApiInfo, Version};

mod gen_api;
mod gen_api_sys;

use proc_macro2::TokenStream;

use anyhow::anyhow;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

type Error = Box<dyn std::error::Error + 'static>;

async fn update_versions(version: &Version) -> Result<(), Error> {
    let version = format!("{}.{}.{}", version.major, version.minor, version.patch);
    let sys_path = Path::new(NVIM_API_SYS).join("Cargo.toml");
    let mut sys = tokio::fs::read_to_string(&sys_path)
        .await?
        .parse::<toml_edit::DocumentMut>()?;
    let mut workspace = tokio::fs::read_to_string(WORKSPACE_MANIFEST)
        .await?
        .parse::<toml_edit::DocumentMut>()?;
    sys["package"]["version"] = toml_edit::value(&version);
    workspace["workspace"]["dependencies"]["nvim-api-sys"]["version"] = toml_edit::value(&version);
    tokio::fs::write(sys_path, sys.to_string()).await?;
    tokio::fs::write(WORKSPACE_MANIFEST, workspace.to_string()).await?;
    Ok(())
}

async fn gen_to(
    api_info: &ApiInfo,
    path: &Path,
    gen_fn: impl for<'a> FnOnce(&'a ApiInfo) -> TokenStream,
) -> Result<(), Error> {
    println!("Generating {} ...", path.display());
    let tokens = gen_fn(api_info);

    let input = syn::parse2(tokens)?;
    let content = prettyplease::unparse(&input);

    tokio::fs::write(path, content).await?;
    let status = tokio::process::Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(path)
        .status()
        .await?;
    if !status.success() {
        return Err(anyhow!("rustfmt failed for {}: {status}", path.display()).into());
    }
    println!("{}: generated", path.display());
    Ok(())
}

#[derive(Debug, PartialEq)]
enum Source {
    Download(Option<Version>),
    Local(PathBuf),
}

fn parse_cli_args(args: impl IntoIterator<Item = OsString>) -> Result<Source, Error> {
    let mut args = args.into_iter();
    let Some(arg) = args.next() else {
        return Ok(Source::Download(None));
    };

    let source = if arg == "--nvim" {
        let path = args
            .next()
            .ok_or_else(|| anyhow!("--nvim requires a path"))?;
        if path.is_empty() {
            return Err(anyhow!("--nvim requires a nonempty path").into());
        }
        Source::Local(path.into())
    } else {
        let err = || anyhow!("expected a version or --nvim PATH, got {}", arg.display());
        let ver = arg.to_str().ok_or_else(err)?;
        let ver = ver.strip_prefix('v').unwrap_or(ver);
        let mut parts = ver.split('.');
        let mut next = || -> Result<i64, Error> {
            let part = parts.next().ok_or_else(err)?;
            if part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit()) {
                return Err(err().into());
            }
            Ok(part.parse::<u16>().map_err(|_| err())?.into())
        };
        let version = Version {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        if parts.next().is_some() {
            return Err(err().into());
        }
        Source::Download(Some(version))
    };

    if let Some(arg) = args.next() {
        return Err(anyhow!("unexpected CLI argument: {}", arg.display()).into());
    }
    Ok(source)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let source = parse_cli_args(std::env::args_os().skip(1))?;
    let prog = match source {
        Source::Local(path) => path,
        Source::Download(ver) => {
            if cfg!(not(all(
                target_os = "linux",
                any(target_arch = "x86_64", target_arch = "aarch64")
            ))) {
                return Err(anyhow!("Downloading Neovim requires Linux x86_64 or aarch64").into());
            }
            install::setup(ver).await?
        }
    };
    println!("Neovim executable: {}", prog.display());
    println!();

    let api_info = ApiInfo::from_prog(&prog).await?;
    println!("Read API metadata");
    {
        let Version {
            major,
            minor,
            patch,
        } = api_info.version;
        println!("    Version: {major}.{minor}.{patch}");
    }
    println!();

    let api_sys_rs = Path::new(NVIM_API_SYS).join("src/generated.rs");
    gen_to(&api_info, &api_sys_rs, gen_api_sys::gen_rs).await?;
    let api_rs = Path::new(NVIM_API).join("src/generated.rs");
    gen_to(&api_info, &api_rs, gen_api::gen_rs).await?;
    update_versions(&api_info.version).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Source, Error> {
        parse_cli_args(args.iter().map(OsString::from))
    }

    #[test]
    fn sources() {
        assert_eq!(parse(&[]).unwrap(), Source::Download(None));
        for version in ["0.12.6", "v0.12.6"] {
            assert_eq!(
                parse(&[version]).unwrap(),
                Source::Download(Some(Version {
                    major: 0,
                    minor: 12,
                    patch: 6,
                }))
            );
        }
        assert_eq!(
            parse(&["--nvim", "/path with spaces/nvim"]).unwrap(),
            Source::Local(PathBuf::from("/path with spaces/nvim"))
        );
    }

    #[test]
    fn invalid_args() {
        for args in [
            &["--nvim"][..],
            &["--nvim", ""],
            &["--nvim", "nvim", "extra"],
            &["0.12.6", "extra"],
            &["0.12"],
            &["0.12.6.1"],
            &["0.12."],
            &["-1.12.6"],
            &["+0.12.6"],
            &["0.12.65536"],
            &["--unknown"],
        ] {
            assert!(parse(args).is_err(), "accepted {args:?}");
        }
    }
}
