use crate::{Buffer, KeyInput, Tabpage, UiOptions, api};

use nvim_rpc_plugin::{Neovim, Object};

use tokio::io::AsyncWrite;
use tokio::sync::Mutex;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// A UI action failed validation or its RPC call failed.
#[derive(Debug)]
pub enum ClientError {
    Rpc(nvim_rpc_plugin::Error),
    InvalidSize { width: u32, height: u32 },
    InvalidOptions,
    InvalidInputAcceptance { accepted: usize, remaining: usize },
}

impl From<nvim_rpc_plugin::Error> for ClientError {
    fn from(e: nvim_rpc_plugin::Error) -> Self {
        Self::Rpc(e)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rpc(e) => e.fmt(f),
            Self::InvalidSize { width, height } => {
                write!(f, "UI dimensions must be positive: {width} by {height}")
            }
            Self::InvalidOptions => f.write_str("UI attachment requires ext_linegrid"),
            Self::InvalidInputAcceptance {
                accepted,
                remaining,
            } => write!(
                f,
                "Neovim accepted {accepted} input bytes from a {remaining}-byte sequence"
            ),
        }
    }
}

impl std::error::Error for ClientError {}

/// UI actions and serialized keyboard input on an existing plugin connection.
///
/// The plugin's [`Neovim`] owns RPC encoding, response matching, and transport
/// lifecycle. Use [`Self::nvim()`] for ordinary API calls. Clones of this wrapper
/// share a keyboard sequence lock, so partial input retries do not interleave.
pub struct Client<W> {
    nvim: Neovim<W>,
    input: Arc<Mutex<()>>,
}

impl<W> Clone for Client<W> {
    fn clone(&self) -> Self {
        Self {
            nvim: self.nvim.clone(),
            input: Arc::clone(&self.input),
        }
    }
}

impl<W> Client<W> {
    /// Adds UI actions to an existing RPC client without starting tasks.
    ///
    /// Create one wrapper per connection and clone it for concurrent input
    /// producers. Separately constructed wrappers have independent input locks.
    pub fn new(nvim: Neovim<W>) -> Self {
        Self {
            nvim,
            input: Arc::new(Mutex::new(())),
        }
    }

    /// Borrows the underlying RPC client for typed requests and notifications.
    pub fn nvim(&self) -> &Neovim<W> {
        &self.nvim
    }
}

impl<W: AsyncWrite + Unpin + Send + 'static> Client<W> {
    /// Attaches the UI and waits for Neovim's acknowledgement.
    ///
    /// Dimensions must be positive. Enable `ext_linegrid`, or an option that
    /// implies it: `ext_multigrid`, `ext_hlstate`, or `ext_messages`.
    pub async fn attach(
        &self,
        width: u32,
        height: u32,
        options: UiOptions,
    ) -> Result<(), ClientError> {
        validate_attachment(width, height, &options)?;
        self.nvim
            .request::<api::NvimUiAttach, Object>((i64::from(width), i64::from(height), &options))
            .await?;
        Ok(())
    }

    /// Detaches the UI and waits for Neovim's acknowledgement.
    pub async fn detach(&self) -> Result<(), ClientError> {
        self.nvim.request::<api::NvimUiDetach, Object>(()).await?;
        Ok(())
    }

    /// Sends a logical key, committed text, or a Neovim key sequence.
    ///
    /// Uses [`KeyInput::to_nvim()`] and [`Self::input()`], including partial
    /// acceptance retries. Borrowed source text remains borrowed while sending.
    pub async fn send_key(&self, key: KeyInput<'_>) -> Result<(), ClientError> {
        let keys = key.to_nvim();
        self.input(keys.as_str()).await
    }

    /// Sends Neovim key notation until all bytes have been accepted.
    ///
    /// Escape literal `<` as `<LT>`, or use [`Self::send_key()`] with
    /// [`KeyInput::Text`] for committed text. Use [`Self::paste()`] for clipboard
    /// blocks. Calls through this client and its clones serialize complete key
    /// sequences. Completion means accepted input, rather than executed keys.
    /// Cancelling a call does not retract input already accepted by Neovim.
    pub async fn input(&self, keys: &str) -> Result<(), ClientError> {
        let _guard = self.input.lock().await;
        let mut remaining = keys.as_bytes();
        while !remaining.is_empty() {
            let accepted = self
                .nvim
                .request::<api::NvimInput, usize>((remaining,))
                .await?;
            let Some(rest) = remaining.get(accepted..) else {
                self.nvim.close();
                return Err(ClientError::InvalidInputAcceptance {
                    accepted,
                    remaining: remaining.len(),
                });
            };
            remaining = rest;
            if accepted == 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        Ok(())
    }

    /// Sends a mouse event using grid cell coordinates.
    ///
    /// Button, action, and modifier names follow Neovim's `nvim_input_mouse`
    /// API. Returns after writing and flushing the notification.
    pub async fn mouse(
        &self,
        button: &str,
        action: &str,
        modifiers: &str,
        grid: i64,
        row: i64,
        col: i64,
    ) -> Result<(), ClientError> {
        self.nvim
            .notify::<api::NvimInputMouse>((button, action, modifiers, grid, row, col))
            .await?;
        Ok(())
    }

    /// Requests positive UI dimensions in cells without waiting for execution.
    ///
    /// Neovim may adjust the request. Render using subsequent `grid_resize`
    /// events rather than assuming these dimensions took effect.
    pub async fn resize(&self, width: u32, height: u32) -> Result<(), ClientError> {
        positive_size(width, height)?;
        self.nvim
            .notify::<api::NvimUiTryResize>((i64::from(width), i64::from(height)))
            .await?;
        Ok(())
    }

    /// Pastes literal text and reports whether Neovim accepted it.
    ///
    /// Carriage returns and carriage-return/newline pairs become line breaks.
    pub async fn paste(&self, text: &str) -> Result<bool, ClientError> {
        self.nvim
            .request::<api::NvimPaste, bool>((text, true, -1i64))
            .await
            .map_err(Into::into)
    }

    /// Reports UI focus without waiting for editor execution.
    pub async fn focus(&self, focused: bool) -> Result<(), ClientError> {
        self.nvim.notify::<api::NvimUiSetFocus>((focused,)).await?;
        Ok(())
    }

    /// Selects or accepts a completion item without waiting for execution.
    ///
    /// `item` is a zero-based index; `-1` restores the original text. `insert`
    /// inserts the selected text, and `finish` completes the selection.
    pub async fn select_popupmenu_item(
        &self,
        item: i64,
        insert: bool,
        finish: bool,
    ) -> Result<(), ClientError> {
        self.nvim
            .notify::<api::NvimSelectPopupmenuItem>((
                item,
                insert,
                finish,
                nvim_rpc::mp::encode::primitive::Map([] as [(&str, i64); 0]),
            ))
            .await?;
        Ok(())
    }

    /// Selects the current tabpage without waiting for editor execution.
    pub async fn set_current_tabpage(&self, tab: Tabpage) -> Result<(), ClientError> {
        self.nvim
            .notify::<api::NvimSetCurrentTabpage>((tab,))
            .await?;
        Ok(())
    }

    /// Selects the current buffer without waiting for editor execution.
    pub async fn set_current_buffer(&self, buffer: Buffer) -> Result<(), ClientError> {
        self.nvim
            .notify::<api::NvimSetCurrentBuf>((buffer,))
            .await?;
        Ok(())
    }

    /// Sends an Ex command without waiting for editor execution.
    ///
    /// Execution errors arrive as `nvim_error_event` notifications on the
    /// existing plugin connection.
    pub async fn command(&self, command: &str) -> Result<(), ClientError> {
        self.nvim.notify::<api::NvimCommand>((command,)).await?;
        Ok(())
    }
}

fn positive_size(width: u32, height: u32) -> Result<(), ClientError> {
    if width == 0 || height == 0 {
        Err(ClientError::InvalidSize { width, height })
    } else {
        Ok(())
    }
}

pub(crate) fn validate_attachment(
    width: u32,
    height: u32,
    options: &UiOptions,
) -> Result<(), ClientError> {
    positive_size(width, height)?;
    if options.ext_linegrid || options.ext_multigrid || options.ext_hlstate || options.ext_messages
    {
        Ok(())
    } else {
        Err(ClientError::InvalidOptions)
    }
}
