use nvim_api::mp::encode::StdWrite;
use nvim_api::read::StdRead;
use nvim_api::rpc::Message;
use nvim_api::{Neovim, Receiver, api};

use std::io;
use std::process::{Child, Command, Stdio};

struct Process(Child);

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bin = std::env::var_os("NVIM_BIN").unwrap_or_else(|| "nvim".into());
    let mut process = Process(
        Command::new(bin)
            .args(["--embed", "--headless", "--clean", "-n", "-i", "NONE"])
            .env("NVIM_LOG_FILE", "/tmp/nvim-api-example.log")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?,
    );
    let writer = process
        .0
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("missing Neovim stdin"))?;
    let reader = process
        .0
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing Neovim stdout"))?;
    let mut nvim = Neovim::new(StdWrite::new(writer));
    let mut receiver = Receiver::new(StdRead(reader), [0; 4096]);

    let pending = nvim.request::<api::NvimEval>(("'hello from Neovim'",))?;
    nvim.flush()?;
    loop {
        match receiver.recv()? {
            Message::Response(response) if pending.matches(&response) => {
                let text: &str = pending
                    .decode(response)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                println!("{text}");
                return Ok(());
            }
            Message::Notification(_) => {}
            _ => return Err(io::Error::other("unexpected RPC message").into()),
        }
    }
}
