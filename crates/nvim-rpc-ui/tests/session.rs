use nvim_rpc::api;

use nvim_rpc_ui::Embedded;
use nvim_rpc_ui::UiOptions;

use tokio::time::{Duration, timeout};

#[tokio::test]
async fn invalid_configuration_is_rejected_before_starting_the_command() {
    use nvim_rpc_ui::{ClientError, ConnectionOptions, Error};
    let mut command = tokio::process::Command::new("/nonexistent/nvim-rpc-ui-test-executable");
    let options = UiOptions {
        ext_linegrid: true,
        ..Default::default()
    };
    assert!(matches!(
        Embedded::spawn(&mut command, 0, 20, options).await,
        Err(Error::Client(ClientError::InvalidSize {
            width: 0,
            height: 20
        }))
    ));
    let options = UiOptions {
        ext_linegrid: true,
        ..Default::default()
    };
    let limits = ConnectionOptions {
        rpc: nvim_rpc_plugin::ConnectionOptions {
            max_rpc_bytes: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(matches!(
        Embedded::spawn_with_options(&mut command, 80, 20, options, limits).await,
        Err(Error::Connection(
            nvim_rpc_plugin::ConnectionError::InvalidOptions(_)
        ))
    ));
}

#[tokio::test]
#[ignore = "requires NVIM_BIN pointing to Neovim"]
async fn live_all_extensions_input_widgets_and_shutdown() {
    struct View {
        cmdline_visible: bool,
    }

    impl nvim_rpc_ui::Backend for View {
        type Error = std::convert::Infallible;

        fn present(&mut self, frame: nvim_rpc_ui::Frame) -> Result<(), Self::Error> {
            if !frame.state.cmdlines.is_empty() {
                assert!(!frame.state.grids.is_empty());
                assert!(!frame.state.highlights.is_empty());
                self.cmdline_visible = true;
            }
            Ok(())
        }
    }

    let bin = std::env::var_os("NVIM_BIN").expect("set NVIM_BIN");
    let mut command = tokio::process::Command::new(bin);
    command.args(["--clean", "-n", "-i", "NONE"]).env(
        "NVIM_LOG_FILE",
        std::env::temp_dir().join("nvim-rpc-ui-test.log"),
    );
    let options = UiOptions {
        rgb: true,
        ext_linegrid: true,
        ext_multigrid: true,
        ext_hlstate: true,
        ext_cmdline: true,
        ext_popupmenu: true,
        ext_tabline: true,
        ext_wildmenu: true,
        ext_messages: true,
        ext_termcolors: true,
    };
    let mut editor = timeout(
        Duration::from_secs(15),
        Embedded::spawn(&mut command, 80, 24, options),
    )
    .await
    .unwrap()
    .unwrap();
    let client = editor.client().clone();
    client.input("ihello<ESC>").await.unwrap();
    let line: String = client
        .nvim()
        .request::<api::NvimGetCurrentLine, String>(())
        .await
        .unwrap();
    assert_eq!(line, "hello");
    client.command("echo 'UI integration test'").await.unwrap();
    client.input(":").await.unwrap();
    let mut backend = View {
        cmdline_visible: false,
    };
    timeout(Duration::from_secs(5), async {
        while !backend.cmdline_visible {
            assert!(editor.present_next(&mut backend).await.unwrap());
        }
    })
    .await
    .unwrap();
    client.input("<ESC>").await.unwrap();
    timeout(Duration::from_secs(5), editor.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert!(
        client
            .nvim()
            .request::<api::NvimGetCurrentLine, String>(())
            .await
            .is_err()
    );
}

#[tokio::test]
#[ignore = "requires NVIM_BIN pointing to Neovim"]
async fn live_transient_float_closed_before_redraw_keeps_ui_and_rpc_available() {
    let mut command =
        tokio::process::Command::new(std::env::var_os("NVIM_BIN").expect("set NVIM_BIN"));
    command.args(["-u", "NONE", "-i", "NONE", "-n"]).env(
        "NVIM_LOG_FILE",
        std::env::temp_dir().join("nvim-rpc-ui-transient-float.log"),
    );

    let options = UiOptions {
        rgb: true,
        ext_linegrid: true,
        ext_multigrid: true,
        ext_cmdline: true,
        ext_popupmenu: true,
        ext_tabline: true,
        ext_messages: true,
        ..UiOptions::default()
    };
    let mut editor = timeout(
        Duration::from_secs(15),
        Embedded::spawn(&mut command, 80, 24, options),
    )
    .await
    .unwrap()
    .unwrap();

    timeout(Duration::from_secs(10), async {
        let initial = editor.frames().recv().await.unwrap().unwrap();
        let client = editor.client().clone();
        let closed = client
            .nvim()
            .request::<api::NvimExecLua, bool>((
                r#"
local buf = vim.api.nvim_create_buf(false, true)
local win = vim.api.nvim_open_win(buf, false, {
    relative = "editor",
    row = 1,
    col = 1,
    width = 10,
    height = 3,
    style = "minimal",
})
vim.api.nvim_win_close(win, true)
return not vim.api.nvim_win_is_valid(win)
"#,
                (),
            ))
            .await
            .unwrap();
        assert!(closed);

        client
            .command("call setline(1, 'after transient float') | redraw")
            .await
            .unwrap();
        let line: String = client
            .nvim()
            .request::<api::NvimGetCurrentLine, String>(())
            .await
            .unwrap();
        assert_eq!(line, "after transient float");

        loop {
            let frame = editor.frames().recv().await.unwrap().unwrap();
            assert!(frame.sequence > initial.sequence);

            if frame.state.grids.values().any(|grid| {
                grid.rows.iter().any(|row| {
                    row.cells
                        .iter()
                        .map(|cell| cell.text.as_ref())
                        .collect::<String>()
                        .contains("after transient float")
                })
            }) {
                break;
            }
        }
    })
    .await
    .expect("transient float interrupted UI or RPC delivery");

    timeout(Duration::from_secs(5), editor.shutdown())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
#[ignore = "requires NVIM_BIN pointing to Neovim"]
async fn live_implicit_linegrid_and_natural_exit_are_reaped() {
    let mut command =
        tokio::process::Command::new(std::env::var_os("NVIM_BIN").expect("set NVIM_BIN"));
    command.args(["--clean", "-n", "-i", "NONE"]).env(
        "NVIM_LOG_FILE",
        std::env::temp_dir().join("nvim-rpc-ui-test.log"),
    );
    let options = UiOptions {
        ext_messages: true,
        ..UiOptions::default()
    };
    let mut editor = timeout(
        Duration::from_secs(15),
        Embedded::spawn(&mut command, 40, 12, options),
    )
    .await
    .unwrap()
    .unwrap();
    editor.client().command("qa!").await.unwrap();
    let status = timeout(Duration::from_secs(5), editor.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());
    assert_eq!(editor.shutdown().await.unwrap().code(), status.code());
}

#[tokio::test]
#[ignore = "requires NVIM_BIN pointing to Neovim"]
async fn live_portable_keys_insert_literal_text_and_match_neovim_key_notation() {
    use nvim_rpc_ui::{Buffer, KeyInput, Modifiers, SpecialKey};
    let mut command =
        tokio::process::Command::new(std::env::var_os("NVIM_BIN").expect("set NVIM_BIN"));
    command.args(["--clean", "-n", "-i", "NONE"]).env(
        "NVIM_LOG_FILE",
        std::env::temp_dir().join("nvim-rpc-ui-keys.log"),
    );
    let mut editor = timeout(
        Duration::from_secs(15),
        Embedded::spawn(
            &mut command,
            80,
            24,
            UiOptions {
                ext_linegrid: true,
                ..Default::default()
            },
        ),
    )
    .await
    .unwrap()
    .unwrap();
    let client = editor.client();
    timeout(Duration::from_secs(10), async {
        for (input, expected) in [
            (
                KeyInput::from_ascii(1, Modifiers::default()).unwrap(),
                "<C-a>",
            ),
            (
                KeyInput::from_char(
                    'ä',
                    Modifiers {
                        alt: true,
                        ..Default::default()
                    },
                ),
                "<M-ä>",
            ),
            (
                KeyInput::from_char(
                    '>',
                    Modifiers {
                        control: true,
                        ..Default::default()
                    },
                ),
                "<C->>",
            ),
            (
                KeyInput::SpecialKey {
                    key: SpecialKey::PageDown,
                    modifiers: Modifiers {
                        shift: true,
                        control: true,
                        alt: true,
                        command: true,
                    },
                },
                "<S-C-M-D-PageDown>",
            ),
            (
                KeyInput::SpecialKey {
                    key: SpecialKey::F35,
                    modifiers: Modifiers::default(),
                },
                "<F35>",
            ),
        ] {
            let actual: Vec<u8> = client
                .nvim()
                .request::<api::NvimReplaceTermcodes, Vec<u8>>((
                    input.to_nvim().as_str(),
                    true,
                    false,
                    true,
                ))
                .await
                .unwrap();
            let expected: Vec<u8> = client
                .nvim()
                .request::<api::NvimReplaceTermcodes, Vec<u8>>((expected, true, false, true))
                .await
                .unwrap();
            assert_eq!(actual, expected);
            assert_ne!(
                actual,
                input.to_nvim().as_str().as_bytes(),
                "notation must be recognized"
            );
        }
        client
            .send_key(KeyInput::from_char('i', Modifiers::default()))
            .await
            .unwrap();
        client
            .send_key(KeyInput::Text {
                text: "日本語<CR>\\|>",
            })
            .await
            .unwrap();
        for key in [SpecialKey::Backspace, SpecialKey::Enter] {
            client
                .send_key(KeyInput::SpecialKey {
                    key,
                    modifiers: Modifiers::default(),
                })
                .await
                .unwrap();
        }
        client
            .send_key(KeyInput::Text { text: "second" })
            .await
            .unwrap();
        client
            .send_key(KeyInput::SpecialKey {
                key: SpecialKey::Escape,
                modifiers: Modifiers::default(),
            })
            .await
            .unwrap();
        let lines = client
            .nvim()
            .request::<api::NvimBufGetLines, Vec<String>>((Buffer::CURRENT, 0i64, -1i64, true))
            .await
            .unwrap();
        assert_eq!(lines, ["日本語<CR>\\|", "second"]);
    })
    .await
    .unwrap();
    editor.shutdown().await.unwrap();
}
