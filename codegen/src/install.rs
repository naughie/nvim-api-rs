use super::Error;
use super::api_info::{ApiInfo, Version};

use anyhow::anyhow;

use std::path::{Path, PathBuf};

const TMP_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tmp/nvim");

fn nvim_bin_path() -> PathBuf {
    Path::new(TMP_DIR).join("nvim")
}

async fn get_version() -> Result<Option<Version>, Error> {
    use std::os::unix::fs::PermissionsExt as _;

    let path = nvim_bin_path();
    if tokio::fs::try_exists(&path).await? {
        let metadata = tokio::fs::metadata(&path).await?;
        let perm = metadata.permissions().mode();

        if metadata.is_file() && perm & 0o100 != 0 {
            match ApiInfo::from_prog(&path).await {
                Ok(api_info) => Ok(Some(api_info.version)),
                Err(e) => {
                    println!(
                        "Could not read the cached Neovim version at {}: {e}. Removing cache",
                        path.display()
                    );
                    tokio::fs::remove_dir_all(TMP_DIR).await?;
                    Ok(None)
                }
            }
        } else {
            println!(
                "{} is not an executable file. Removing cache",
                path.display()
            );
            tokio::fs::remove_dir_all(TMP_DIR).await?;
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

async fn install_new(ver: Option<Version>) -> Result<(), Error> {
    use futures_util::StreamExt as _;
    use tokio::fs::File;
    use tokio::io::{AsyncWriteExt as _, BufWriter};

    let url = {
        let base = "https://github.com/neovim/neovim";
        let fname = if cfg!(target_arch = "x86_64") {
            "nvim-linux-x86_64.appimage"
        } else if cfg!(target_arch = "aarch64") {
            "nvim-linux-arm64.appimage"
        } else {
            unreachable!("download architecture is checked in main");
        };

        if let Some(Version {
            major,
            minor,
            patch,
        }) = ver
        {
            format!("{base}/releases/download/v{major}.{minor}.{patch}/{fname}")
        } else {
            format!("{base}/releases/download/stable/{fname}")
        }
    };
    println!("Fetching {url}");

    let resp = reqwest::get(url).await?;
    let status = resp.status();
    if status.is_success() {
        let mut f = BufWriter::new(
            File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o744)
                .open(nvim_bin_path())
                .await?,
        );
        let mut stream = resp.bytes_stream();
        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;
            f.write_all(&bytes).await?;
        }
        f.flush().await?;

        Ok(())
    } else {
        Err(anyhow!("could not download Neovim AppImage: {status}").into())
    }
}

pub(super) async fn setup(ver: Option<Version>) -> Result<PathBuf, Error> {
    println!("Checking cached Neovim: {ver:?}");
    if let Some(installed_ver) = get_version().await? {
        if Some(installed_ver) != ver {
            println!("Refreshing the Neovim cache for the requested release");
            tokio::fs::remove_dir_all(TMP_DIR).await?;
            tokio::fs::create_dir_all(TMP_DIR).await?;
            install_new(ver).await?;
        }
    } else {
        println!("Neovim is not cached. Downloading");
        tokio::fs::create_dir_all(TMP_DIR).await?;
        install_new(ver).await?;
    }

    Ok(nvim_bin_path())
}
