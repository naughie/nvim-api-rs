use nvim_api_plugin::{
    Array, Bin, DecodeError, FromParams, Handler, IntoResponse, Neovim, api, spawn,
};

use tokio::io::AsyncWrite;

struct Add {
    lhs: i32,
    rhs: i32,
    text: Bin,
}

impl FromParams for Add {
    fn from_params(params: Array) -> Result<Self, DecodeError> {
        let (lhs, rhs, text) = <(i32, i32, Bin)>::from_params(params)?;
        Ok(Self { lhs, rhs, text })
    }
}

enum AddResponse {
    Sum(i32),
    UnknownMethod,
    Overflow,
    CallbackFailed,
}

impl IntoResponse for AddResponse {
    type Value = i32;
    type Error = &'static str;

    fn into_response(self) -> Result<i32, &'static str> {
        match self {
            Self::Sum(sum) => Ok(sum),
            Self::UnknownMethod => Err("unknown request"),
            Self::Overflow => Err("sum is out of range"),
            Self::CallbackFailed => Err("could not read the current line"),
        }
    }
}

struct Plugin;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Plugin {
    type Request = Add;
    type Response = AddResponse;
    type Notification = (Bin,);

    async fn on_request(
        &self,
        method: &[u8],
        Add { lhs, rhs, text }: Self::Request,
        nvim: Neovim<W>,
    ) -> Self::Response {
        if method != b"add" {
            return AddResponse::UnknownMethod;
        }
        let line = match nvim.request::<api::NvimGetCurrentLine, Bin>(()).await {
            Ok(line) => line,
            Err(e) => {
                eprintln!("get current line: {e}");
                return AddResponse::CallbackFailed;
            }
        };
        eprintln!(
            "current line: {:?}; argument: {:?}",
            line.as_slice(),
            text.as_slice()
        );
        match lhs.checked_add(rhs) {
            Some(sum) => AddResponse::Sum(sum),
            None => AddResponse::Overflow,
        }
    }

    async fn on_notification(&self, method: &[u8], (text,): Self::Notification, nvim: Neovim<W>) {
        if method == b"set_line" {
            if let Err(e) = nvim
                .notify::<api::NvimSetCurrentLine>((text.as_slice(),))
                .await
            {
                eprintln!("set_line: {e}");
            }
        } else {
            eprintln!("unknown notification: {method:?}");
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let from_neovim = tokio::io::stdin();
    let to_neovim = tokio::io::stdout();
    let (_nvim, task) = spawn(from_neovim, to_neovim, Plugin);
    task.await??;
    Ok(())
}
