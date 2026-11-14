//! Generated from Neovim 0.12.6 API metadata.
#![allow(deprecated)]
/// Neovim version used to generate these descriptors, as `(major, minor, patch)`.
pub const VERSION: (u16, u16, u16) = (0u16, 12u16, 6u16);
/// API level reported by the Neovim used for generation.
pub const API_LEVEL: i64 = 14i64;
/// Oldest API level supported by the Neovim used for generation.
pub const API_COMPATIBLE: i64 = 0i64;
/// Whether the generated API level was marked as a prerelease.
pub const API_PRERELEASE: bool = false;
/// Function descriptors implementing [`crate::types::ApiFn`].
///
/// Descriptors for handle methods also implement [`crate::types::ApiMethod`],
/// separating the receiver from the remaining parameters.
pub mod functions {
    use super::ext_types;
    use crate::types;
    /// Signature of [`nvim_get_autocmds`](https://neovim.io/doc/user/api/#nvim_get_autocmds()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimGetAutocmds {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetAutocmds {
        type Params = (types::Dict,);
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_get_autocmds";
    }
    /// Signature of [`nvim_create_autocmd`](https://neovim.io/doc/user/api/#nvim_create_autocmd()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimCreateAutocmd {
        pub event: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimCreateAutocmd {
        type Params = (types::Object, types::Dict);
        type Return = types::Int;
        const NAME: &str = "nvim_create_autocmd";
    }
    /// Signature of [`nvim_del_autocmd`](https://neovim.io/doc/user/api/#nvim_del_autocmd()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimDelAutocmd {
        pub id: types::Int,
    }
    impl types::ApiFn for NvimDelAutocmd {
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "nvim_del_autocmd";
    }
    /// Signature of [`nvim_clear_autocmds`](https://neovim.io/doc/user/api/#nvim_clear_autocmds()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimClearAutocmds {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimClearAutocmds {
        type Params = (types::Dict,);
        type Return = ();
        const NAME: &str = "nvim_clear_autocmds";
    }
    /// Signature of [`nvim_create_augroup`](https://neovim.io/doc/user/api/#nvim_create_augroup()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimCreateAugroup {
        pub name: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimCreateAugroup {
        type Params = (types::String, types::Dict);
        type Return = types::Int;
        const NAME: &str = "nvim_create_augroup";
    }
    /// Signature of [`nvim_del_augroup_by_id`](https://neovim.io/doc/user/api/#nvim_del_augroup_by_id()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimDelAugroupById {
        pub id: types::Int,
    }
    impl types::ApiFn for NvimDelAugroupById {
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "nvim_del_augroup_by_id";
    }
    /// Signature of [`nvim_del_augroup_by_name`](https://neovim.io/doc/user/api/#nvim_del_augroup_by_name()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimDelAugroupByName {
        pub name: types::String,
    }
    impl types::ApiFn for NvimDelAugroupByName {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_del_augroup_by_name";
    }
    /// Signature of [`nvim_exec_autocmds`](https://neovim.io/doc/user/api/#nvim_exec_autocmds()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimExecAutocmds {
        pub event: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimExecAutocmds {
        type Params = (types::Object, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_exec_autocmds";
    }
    /// Signature of [`nvim_buf_line_count`](https://neovim.io/doc/user/api/#nvim_buf_line_count()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufLineCount {}
    impl types::ApiFn for NvimBufLineCount {
        type Params = (ext_types::Buffer,);
        type Return = types::Int;
        const NAME: &str = "nvim_buf_line_count";
    }
    impl types::ApiMethod for NvimBufLineCount {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "line_count";
    }
    /// Signature of [`nvim_buf_attach`](https://neovim.io/doc/user/api/#nvim_buf_attach()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimBufAttach {
        pub send_buffer: types::Bool,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufAttach {
        type Params = (ext_types::Buffer, types::Bool, types::Dict);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_attach";
    }
    impl types::ApiMethod for NvimBufAttach {
        type On = ext_types::Buffer;
        type Params = (types::Bool, types::Dict);
        type Return = types::Bool;
        const NAME: &str = "attach";
    }
    /// Signature of [`nvim_buf_detach`](https://neovim.io/doc/user/api/#nvim_buf_detach()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimBufDetach {}
    impl types::ApiFn for NvimBufDetach {
        type Params = (ext_types::Buffer,);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_detach";
    }
    impl types::ApiMethod for NvimBufDetach {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Bool;
        const NAME: &str = "detach";
    }
    /// Signature of [`nvim_buf_get_lines`](https://neovim.io/doc/user/api/#nvim_buf_get_lines()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufGetLines {
        pub start: types::Int,
        pub end: types::Int,
        pub strict_indexing: types::Bool,
    }
    impl types::ApiFn for NvimBufGetLines {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Bool);
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "nvim_buf_get_lines";
    }
    impl types::ApiMethod for NvimBufGetLines {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Bool);
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "get_lines";
    }
    /// Signature of [`nvim_buf_set_lines`](https://neovim.io/doc/user/api/#nvim_buf_set_lines()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufSetLines {
        pub start: types::Int,
        pub end: types::Int,
        pub strict_indexing: types::Bool,
        pub replacement: types::ArrayOf<types::String>,
    }
    impl types::ApiFn for NvimBufSetLines {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Bool,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "nvim_buf_set_lines";
    }
    impl types::ApiMethod for NvimBufSetLines {
        type On = ext_types::Buffer;
        type Params = (
            types::Int,
            types::Int,
            types::Bool,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "set_lines";
    }
    /// Signature of [`nvim_buf_set_text`](https://neovim.io/doc/user/api/#nvim_buf_set_text()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufSetText {
        pub start_row: types::Int,
        pub start_col: types::Int,
        pub end_row: types::Int,
        pub end_col: types::Int,
        pub replacement: types::ArrayOf<types::String>,
    }
    impl types::ApiFn for NvimBufSetText {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "nvim_buf_set_text";
    }
    impl types::ApiMethod for NvimBufSetText {
        type On = ext_types::Buffer;
        type Params = (
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "set_text";
    }
    /// Signature of [`nvim_buf_get_text`](https://neovim.io/doc/user/api/#nvim_buf_get_text()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimBufGetText {
        pub start_row: types::Int,
        pub start_col: types::Int,
        pub end_row: types::Int,
        pub end_col: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufGetText {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Dict,
        );
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "nvim_buf_get_text";
    }
    impl types::ApiMethod for NvimBufGetText {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Int, types::Int, types::Dict);
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "get_text";
    }
    /// Signature of [`nvim_buf_get_offset`](https://neovim.io/doc/user/api/#nvim_buf_get_offset()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimBufGetOffset {
        pub index: types::Int,
    }
    impl types::ApiFn for NvimBufGetOffset {
        type Params = (ext_types::Buffer, types::Int);
        type Return = types::Int;
        const NAME: &str = "nvim_buf_get_offset";
    }
    impl types::ApiMethod for NvimBufGetOffset {
        type On = ext_types::Buffer;
        type Params = (types::Int,);
        type Return = types::Int;
        const NAME: &str = "get_offset";
    }
    /// Signature of [`nvim_buf_get_var`](https://neovim.io/doc/user/api/#nvim_buf_get_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufGetVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufGetVar {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Object;
        const NAME: &str = "nvim_buf_get_var";
    }
    impl types::ApiMethod for NvimBufGetVar {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "get_var";
    }
    /// Signature of [`nvim_buf_get_changedtick`](https://neovim.io/doc/user/api/#nvim_buf_get_changedtick()).
    ///
    /// Since Neovim API level 2.
    pub struct NvimBufGetChangedtick {}
    impl types::ApiFn for NvimBufGetChangedtick {
        type Params = (ext_types::Buffer,);
        type Return = types::Int;
        const NAME: &str = "nvim_buf_get_changedtick";
    }
    impl types::ApiMethod for NvimBufGetChangedtick {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_changedtick";
    }
    /// Signature of [`nvim_buf_get_keymap`](https://neovim.io/doc/user/api/#nvim_buf_get_keymap()).
    ///
    /// Since Neovim API level 3.
    pub struct NvimBufGetKeymap {
        pub mode: types::String,
    }
    impl types::ApiFn for NvimBufGetKeymap {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_buf_get_keymap";
    }
    impl types::ApiMethod for NvimBufGetKeymap {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "get_keymap";
    }
    /// Signature of [`nvim_buf_set_keymap`](https://neovim.io/doc/user/api/#nvim_buf_set_keymap()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimBufSetKeymap {
        pub mode: types::String,
        pub lhs: types::String,
        pub rhs: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufSetKeymap {
        type Params = (
            ext_types::Buffer,
            types::String,
            types::String,
            types::String,
            types::Dict,
        );
        type Return = ();
        const NAME: &str = "nvim_buf_set_keymap";
    }
    impl types::ApiMethod for NvimBufSetKeymap {
        type On = ext_types::Buffer;
        type Params = (types::String, types::String, types::String, types::Dict);
        type Return = ();
        const NAME: &str = "set_keymap";
    }
    /// Signature of [`nvim_buf_del_keymap`](https://neovim.io/doc/user/api/#nvim_buf_del_keymap()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimBufDelKeymap {
        pub mode: types::String,
        pub lhs: types::String,
    }
    impl types::ApiFn for NvimBufDelKeymap {
        type Params = (ext_types::Buffer, types::String, types::String);
        type Return = ();
        const NAME: &str = "nvim_buf_del_keymap";
    }
    impl types::ApiMethod for NvimBufDelKeymap {
        type On = ext_types::Buffer;
        type Params = (types::String, types::String);
        type Return = ();
        const NAME: &str = "del_keymap";
    }
    /// Signature of [`nvim_buf_set_var`](https://neovim.io/doc/user/api/#nvim_buf_set_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufSetVar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimBufSetVar {
        type Params = (ext_types::Buffer, types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_buf_set_var";
    }
    impl types::ApiMethod for NvimBufSetVar {
        type On = ext_types::Buffer;
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "set_var";
    }
    /// Signature of [`nvim_buf_del_var`](https://neovim.io/doc/user/api/#nvim_buf_del_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufDelVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufDelVar {
        type Params = (ext_types::Buffer, types::String);
        type Return = ();
        const NAME: &str = "nvim_buf_del_var";
    }
    impl types::ApiMethod for NvimBufDelVar {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "del_var";
    }
    /// Signature of [`nvim_buf_get_name`](https://neovim.io/doc/user/api/#nvim_buf_get_name()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufGetName {}
    impl types::ApiFn for NvimBufGetName {
        type Params = (ext_types::Buffer,);
        type Return = types::String;
        const NAME: &str = "nvim_buf_get_name";
    }
    impl types::ApiMethod for NvimBufGetName {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::String;
        const NAME: &str = "get_name";
    }
    /// Signature of [`nvim_buf_set_name`](https://neovim.io/doc/user/api/#nvim_buf_set_name()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufSetName {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufSetName {
        type Params = (ext_types::Buffer, types::String);
        type Return = ();
        const NAME: &str = "nvim_buf_set_name";
    }
    impl types::ApiMethod for NvimBufSetName {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "set_name";
    }
    /// Signature of [`nvim_buf_is_loaded`](https://neovim.io/doc/user/api/#nvim_buf_is_loaded()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimBufIsLoaded {}
    impl types::ApiFn for NvimBufIsLoaded {
        type Params = (ext_types::Buffer,);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_is_loaded";
    }
    impl types::ApiMethod for NvimBufIsLoaded {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Bool;
        const NAME: &str = "is_loaded";
    }
    /// Signature of [`nvim_buf_delete`](https://neovim.io/doc/user/api/#nvim_buf_delete()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufDelete {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufDelete {
        type Params = (ext_types::Buffer, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_buf_delete";
    }
    impl types::ApiMethod for NvimBufDelete {
        type On = ext_types::Buffer;
        type Params = (types::Dict,);
        type Return = ();
        const NAME: &str = "delete";
    }
    /// Signature of [`nvim_buf_is_valid`](https://neovim.io/doc/user/api/#nvim_buf_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufIsValid {}
    impl types::ApiFn for NvimBufIsValid {
        type Params = (ext_types::Buffer,);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_is_valid";
    }
    impl types::ApiMethod for NvimBufIsValid {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Bool;
        const NAME: &str = "is_valid";
    }
    /// Signature of [`nvim_buf_del_mark`](https://neovim.io/doc/user/api/#nvim_buf_del_mark()).
    ///
    /// Since Neovim API level 8.
    pub struct NvimBufDelMark {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufDelMark {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_del_mark";
    }
    impl types::ApiMethod for NvimBufDelMark {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = types::Bool;
        const NAME: &str = "del_mark";
    }
    /// Signature of [`nvim_buf_set_mark`](https://neovim.io/doc/user/api/#nvim_buf_set_mark()).
    ///
    /// Since Neovim API level 8.
    pub struct NvimBufSetMark {
        pub name: types::String,
        pub line: types::Int,
        pub col: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufSetMark {
        type Params = (
            ext_types::Buffer,
            types::String,
            types::Int,
            types::Int,
            types::Dict,
        );
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_set_mark";
    }
    impl types::ApiMethod for NvimBufSetMark {
        type On = ext_types::Buffer;
        type Params = (types::String, types::Int, types::Int, types::Dict);
        type Return = types::Bool;
        const NAME: &str = "set_mark";
    }
    /// Signature of [`nvim_buf_get_mark`](https://neovim.io/doc/user/api/#nvim_buf_get_mark()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimBufGetMark {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufGetMark {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "nvim_buf_get_mark";
    }
    impl types::ApiMethod for NvimBufGetMark {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "get_mark";
    }
    /// Signature of [`nvim_buf_call`](https://neovim.io/doc/user/api/#nvim_buf_call()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufCall {
        pub fun: types::Ref,
    }
    impl types::ApiFn for NvimBufCall {
        type Params = (ext_types::Buffer, types::Ref);
        type Return = types::Object;
        const NAME: &str = "nvim_buf_call";
    }
    impl types::ApiMethod for NvimBufCall {
        type On = ext_types::Buffer;
        type Params = (types::Ref,);
        type Return = types::Object;
        const NAME: &str = "call";
    }
    /// Signature of [`nvim_parse_cmd`](https://neovim.io/doc/user/api/#nvim_parse_cmd()).
    ///
    /// Since Neovim API level 10.
    pub struct NvimParseCmd {
        pub str: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimParseCmd {
        type Params = (types::String, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_parse_cmd";
    }
    /// Signature of [`nvim_cmd`](https://neovim.io/doc/user/api/#nvim_cmd()).
    ///
    /// Since Neovim API level 10.
    pub struct NvimCmd {
        pub cmd: types::Dict,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimCmd {
        type Params = (types::Dict, types::Dict);
        type Return = types::String;
        const NAME: &str = "nvim_cmd";
    }
    /// Signature of [`nvim_create_user_command`](https://neovim.io/doc/user/api/#nvim_create_user_command()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimCreateUserCommand {
        pub name: types::String,
        pub cmd: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimCreateUserCommand {
        type Params = (types::String, types::Object, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_create_user_command";
    }
    /// Signature of [`nvim_del_user_command`](https://neovim.io/doc/user/api/#nvim_del_user_command()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimDelUserCommand {
        pub name: types::String,
    }
    impl types::ApiFn for NvimDelUserCommand {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_del_user_command";
    }
    /// Signature of [`nvim_buf_create_user_command`](https://neovim.io/doc/user/api/#nvim_buf_create_user_command()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimBufCreateUserCommand {
        pub name: types::String,
        pub cmd: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufCreateUserCommand {
        type Params = (ext_types::Buffer, types::String, types::Object, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_buf_create_user_command";
    }
    impl types::ApiMethod for NvimBufCreateUserCommand {
        type On = ext_types::Buffer;
        type Params = (types::String, types::Object, types::Dict);
        type Return = ();
        const NAME: &str = "create_user_command";
    }
    /// Signature of [`nvim_buf_del_user_command`](https://neovim.io/doc/user/api/#nvim_buf_del_user_command()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimBufDelUserCommand {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufDelUserCommand {
        type Params = (ext_types::Buffer, types::String);
        type Return = ();
        const NAME: &str = "nvim_buf_del_user_command";
    }
    impl types::ApiMethod for NvimBufDelUserCommand {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "del_user_command";
    }
    /// Signature of [`nvim_get_commands`](https://neovim.io/doc/user/api/#nvim_get_commands()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimGetCommands {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetCommands {
        type Params = (types::Dict,);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_commands";
    }
    /// Signature of [`nvim_buf_get_commands`](https://neovim.io/doc/user/api/#nvim_buf_get_commands()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimBufGetCommands {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufGetCommands {
        type Params = (ext_types::Buffer, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_buf_get_commands";
    }
    impl types::ApiMethod for NvimBufGetCommands {
        type On = ext_types::Buffer;
        type Params = (types::Dict,);
        type Return = types::Dict;
        const NAME: &str = "get_commands";
    }
    /// Signature of [`nvim_exec`](https://neovim.io/doc/user/api/#nvim_exec()).
    ///
    /// Since Neovim API level 7; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimExec {
        pub src: types::String,
        pub output: types::Bool,
    }
    impl types::ApiFn for NvimExec {
        type Params = (types::String, types::Bool);
        type Return = types::String;
        const NAME: &str = "nvim_exec";
    }
    /// Signature of [`nvim_command_output`](https://neovim.io/doc/user/api/#nvim_command_output()).
    ///
    /// Since Neovim API level 1; deprecated since 7.
    #[deprecated(note = "Deprecated since Neovim API level 7")]
    pub struct NvimCommandOutput {
        pub command: types::String,
    }
    impl types::ApiFn for NvimCommandOutput {
        type Params = (types::String,);
        type Return = types::String;
        const NAME: &str = "nvim_command_output";
    }
    /// Signature of [`nvim_execute_lua`](https://neovim.io/doc/user/api/#nvim_execute_lua()).
    ///
    /// Since Neovim API level 3; deprecated since 7.
    #[deprecated(note = "Deprecated since Neovim API level 7")]
    pub struct NvimExecuteLua {
        pub code: types::String,
        pub args: types::Array,
    }
    impl types::ApiFn for NvimExecuteLua {
        type Params = (types::String, types::Array);
        type Return = types::Object;
        const NAME: &str = "nvim_execute_lua";
    }
    /// Signature of [`nvim_buf_get_number`](https://neovim.io/doc/user/api/#nvim_buf_get_number()).
    ///
    /// Since Neovim API level 1; deprecated since 2.
    #[deprecated(note = "Deprecated since Neovim API level 2")]
    pub struct NvimBufGetNumber {}
    impl types::ApiFn for NvimBufGetNumber {
        type Params = (ext_types::Buffer,);
        type Return = types::Int;
        const NAME: &str = "nvim_buf_get_number";
    }
    impl types::ApiMethod for NvimBufGetNumber {
        type On = ext_types::Buffer;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_number";
    }
    /// Signature of [`nvim_buf_clear_highlight`](https://neovim.io/doc/user/api/#nvim_buf_clear_highlight()).
    ///
    /// Since Neovim API level 1; deprecated since 7.
    #[deprecated(note = "Deprecated since Neovim API level 7")]
    pub struct NvimBufClearHighlight {
        pub ns_id: types::Int,
        pub line_start: types::Int,
        pub line_end: types::Int,
    }
    impl types::ApiFn for NvimBufClearHighlight {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "nvim_buf_clear_highlight";
    }
    impl types::ApiMethod for NvimBufClearHighlight {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "clear_highlight";
    }
    /// Signature of [`nvim_buf_add_highlight`](https://neovim.io/doc/user/api/#nvim_buf_add_highlight()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimBufAddHighlight {
        pub ns_id: types::Int,
        pub hl_group: types::String,
        pub line: types::Int,
        pub col_start: types::Int,
        pub col_end: types::Int,
    }
    impl types::ApiFn for NvimBufAddHighlight {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::String,
            types::Int,
            types::Int,
            types::Int,
        );
        type Return = types::Int;
        const NAME: &str = "nvim_buf_add_highlight";
    }
    impl types::ApiMethod for NvimBufAddHighlight {
        type On = ext_types::Buffer;
        type Params = (
            types::Int,
            types::String,
            types::Int,
            types::Int,
            types::Int,
        );
        type Return = types::Int;
        const NAME: &str = "add_highlight";
    }
    /// Signature of [`nvim_buf_set_virtual_text`](https://neovim.io/doc/user/api/#nvim_buf_set_virtual_text()).
    ///
    /// Since Neovim API level 5; deprecated since 8.
    #[deprecated(note = "Deprecated since Neovim API level 8")]
    pub struct NvimBufSetVirtualText {
        pub src_id: types::Int,
        pub line: types::Int,
        pub chunks: types::Array,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufSetVirtualText {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Array,
            types::Dict,
        );
        type Return = types::Int;
        const NAME: &str = "nvim_buf_set_virtual_text";
    }
    impl types::ApiMethod for NvimBufSetVirtualText {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Array, types::Dict);
        type Return = types::Int;
        const NAME: &str = "set_virtual_text";
    }
    /// Signature of [`nvim_get_hl_by_id`](https://neovim.io/doc/user/api/#nvim_get_hl_by_id()).
    ///
    /// Since Neovim API level 3; deprecated since 9.
    #[deprecated(note = "Deprecated since Neovim API level 9")]
    pub struct NvimGetHlById {
        pub hl_id: types::Int,
        pub rgb: types::Bool,
    }
    impl types::ApiFn for NvimGetHlById {
        type Params = (types::Int, types::Bool);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_hl_by_id";
    }
    /// Signature of [`nvim_get_hl_by_name`](https://neovim.io/doc/user/api/#nvim_get_hl_by_name()).
    ///
    /// Since Neovim API level 3; deprecated since 9.
    #[deprecated(note = "Deprecated since Neovim API level 9")]
    pub struct NvimGetHlByName {
        pub name: types::String,
        pub rgb: types::Bool,
    }
    impl types::ApiFn for NvimGetHlByName {
        type Params = (types::String, types::Bool);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_hl_by_name";
    }
    /// Signature of [`buffer_insert`](https://neovim.io/doc/user/api/#buffer_insert()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferInsert {
        pub buffer: ext_types::Buffer,
        pub lnum: types::Int,
        pub lines: types::ArrayOf<types::String>,
    }
    impl types::ApiFn for BufferInsert {
        type Params = (ext_types::Buffer, types::Int, types::ArrayOf<types::String>);
        type Return = ();
        const NAME: &str = "buffer_insert";
    }
    /// Signature of [`buffer_get_line`](https://neovim.io/doc/user/api/#buffer_get_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetLine {
        pub buffer: ext_types::Buffer,
        pub index: types::Int,
    }
    impl types::ApiFn for BufferGetLine {
        type Params = (ext_types::Buffer, types::Int);
        type Return = types::String;
        const NAME: &str = "buffer_get_line";
    }
    /// Signature of [`buffer_set_line`](https://neovim.io/doc/user/api/#buffer_set_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetLine {
        pub buffer: ext_types::Buffer,
        pub index: types::Int,
        pub line: types::String,
    }
    impl types::ApiFn for BufferSetLine {
        type Params = (ext_types::Buffer, types::Int, types::String);
        type Return = ();
        const NAME: &str = "buffer_set_line";
    }
    /// Signature of [`buffer_del_line`](https://neovim.io/doc/user/api/#buffer_del_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferDelLine {
        pub buffer: ext_types::Buffer,
        pub index: types::Int,
    }
    impl types::ApiFn for BufferDelLine {
        type Params = (ext_types::Buffer, types::Int);
        type Return = ();
        const NAME: &str = "buffer_del_line";
    }
    /// Signature of [`buffer_get_line_slice`](https://neovim.io/doc/user/api/#buffer_get_line_slice()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetLineSlice {
        pub buffer: ext_types::Buffer,
        pub start: types::Int,
        pub end: types::Int,
        pub include_start: types::Bool,
        pub include_end: types::Bool,
    }
    impl types::ApiFn for BufferGetLineSlice {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Bool,
            types::Bool,
        );
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "buffer_get_line_slice";
    }
    /// Signature of [`buffer_set_line_slice`](https://neovim.io/doc/user/api/#buffer_set_line_slice()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetLineSlice {
        pub buffer: ext_types::Buffer,
        pub start: types::Int,
        pub end: types::Int,
        pub include_start: types::Bool,
        pub include_end: types::Bool,
        pub replacement: types::ArrayOf<types::String>,
    }
    impl types::ApiFn for BufferSetLineSlice {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Bool,
            types::Bool,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "buffer_set_line_slice";
    }
    /// Signature of [`buffer_set_var`](https://neovim.io/doc/user/api/#buffer_set_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetVar {
        pub buffer: ext_types::Buffer,
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for BufferSetVar {
        type Params = (ext_types::Buffer, types::String, types::Object);
        type Return = types::Object;
        const NAME: &str = "buffer_set_var";
    }
    /// Signature of [`buffer_del_var`](https://neovim.io/doc/user/api/#buffer_del_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferDelVar {
        pub buffer: ext_types::Buffer,
        pub name: types::String,
    }
    impl types::ApiFn for BufferDelVar {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Object;
        const NAME: &str = "buffer_del_var";
    }
    /// Signature of [`window_set_var`](https://neovim.io/doc/user/api/#window_set_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowSetVar {
        pub window: ext_types::Window,
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for WindowSetVar {
        type Params = (ext_types::Window, types::String, types::Object);
        type Return = types::Object;
        const NAME: &str = "window_set_var";
    }
    /// Signature of [`window_del_var`](https://neovim.io/doc/user/api/#window_del_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowDelVar {
        pub window: ext_types::Window,
        pub name: types::String,
    }
    impl types::ApiFn for WindowDelVar {
        type Params = (ext_types::Window, types::String);
        type Return = types::Object;
        const NAME: &str = "window_del_var";
    }
    /// Signature of [`tabpage_set_var`](https://neovim.io/doc/user/api/#tabpage_set_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageSetVar {
        pub tabpage: ext_types::Tabpage,
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for TabpageSetVar {
        type Params = (ext_types::Tabpage, types::String, types::Object);
        type Return = types::Object;
        const NAME: &str = "tabpage_set_var";
    }
    /// Signature of [`tabpage_del_var`](https://neovim.io/doc/user/api/#tabpage_del_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageDelVar {
        pub tabpage: ext_types::Tabpage,
        pub name: types::String,
    }
    impl types::ApiFn for TabpageDelVar {
        type Params = (ext_types::Tabpage, types::String);
        type Return = types::Object;
        const NAME: &str = "tabpage_del_var";
    }
    /// Signature of [`vim_set_var`](https://neovim.io/doc/user/api/#vim_set_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetVar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for VimSetVar {
        type Params = (types::String, types::Object);
        type Return = types::Object;
        const NAME: &str = "vim_set_var";
    }
    /// Signature of [`vim_del_var`](https://neovim.io/doc/user/api/#vim_del_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimDelVar {
        pub name: types::String,
    }
    impl types::ApiFn for VimDelVar {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "vim_del_var";
    }
    /// Signature of [`nvim_get_option_info`](https://neovim.io/doc/user/api/#nvim_get_option_info()).
    ///
    /// Since Neovim API level 7; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimGetOptionInfo {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetOptionInfo {
        type Params = (types::String,);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_option_info";
    }
    /// Signature of [`nvim_set_option`](https://neovim.io/doc/user/api/#nvim_set_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimSetOption {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimSetOption {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_set_option";
    }
    /// Signature of [`nvim_get_option`](https://neovim.io/doc/user/api/#nvim_get_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimGetOption {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetOption {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "nvim_get_option";
    }
    /// Signature of [`nvim_buf_get_option`](https://neovim.io/doc/user/api/#nvim_buf_get_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimBufGetOption {
        pub name: types::String,
    }
    impl types::ApiFn for NvimBufGetOption {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Object;
        const NAME: &str = "nvim_buf_get_option";
    }
    impl types::ApiMethod for NvimBufGetOption {
        type On = ext_types::Buffer;
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "get_option";
    }
    /// Signature of [`nvim_buf_set_option`](https://neovim.io/doc/user/api/#nvim_buf_set_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimBufSetOption {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimBufSetOption {
        type Params = (ext_types::Buffer, types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_buf_set_option";
    }
    impl types::ApiMethod for NvimBufSetOption {
        type On = ext_types::Buffer;
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "set_option";
    }
    /// Signature of [`nvim_win_get_option`](https://neovim.io/doc/user/api/#nvim_win_get_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimWinGetOption {
        pub name: types::String,
    }
    impl types::ApiFn for NvimWinGetOption {
        type Params = (ext_types::Window, types::String);
        type Return = types::Object;
        const NAME: &str = "nvim_win_get_option";
    }
    impl types::ApiMethod for NvimWinGetOption {
        type On = ext_types::Window;
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "get_option";
    }
    /// Signature of [`nvim_win_set_option`](https://neovim.io/doc/user/api/#nvim_win_set_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub struct NvimWinSetOption {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimWinSetOption {
        type Params = (ext_types::Window, types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_win_set_option";
    }
    impl types::ApiMethod for NvimWinSetOption {
        type On = ext_types::Window;
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "set_option";
    }
    /// Signature of [`nvim_call_atomic`](https://neovim.io/doc/user/api/#nvim_call_atomic()).
    ///
    /// Since Neovim API level 1; deprecated since 12.
    #[deprecated(note = "Deprecated since Neovim API level 12")]
    pub struct NvimCallAtomic {
        pub calls: types::Array,
    }
    impl types::ApiFn for NvimCallAtomic {
        type Params = (types::Array,);
        type Return = types::Array;
        const NAME: &str = "nvim_call_atomic";
    }
    /// Signature of [`nvim_subscribe`](https://neovim.io/doc/user/api/#nvim_subscribe()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimSubscribe {
        pub event: types::String,
    }
    impl types::ApiFn for NvimSubscribe {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_subscribe";
    }
    /// Signature of [`nvim_unsubscribe`](https://neovim.io/doc/user/api/#nvim_unsubscribe()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimUnsubscribe {
        pub event: types::String,
    }
    impl types::ApiFn for NvimUnsubscribe {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_unsubscribe";
    }
    /// Signature of [`nvim_out_write`](https://neovim.io/doc/user/api/#nvim_out_write()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimOutWrite {
        pub str: types::String,
    }
    impl types::ApiFn for NvimOutWrite {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_out_write";
    }
    /// Signature of [`nvim_err_write`](https://neovim.io/doc/user/api/#nvim_err_write()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimErrWrite {
        pub str: types::String,
    }
    impl types::ApiFn for NvimErrWrite {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_err_write";
    }
    /// Signature of [`nvim_err_writeln`](https://neovim.io/doc/user/api/#nvim_err_writeln()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimErrWriteln {
        pub str: types::String,
    }
    impl types::ApiFn for NvimErrWriteln {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_err_writeln";
    }
    /// Signature of [`nvim_notify`](https://neovim.io/doc/user/api/#nvim_notify()).
    ///
    /// Since Neovim API level 7; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub struct NvimNotify {
        pub msg: types::String,
        pub log_level: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimNotify {
        type Params = (types::String, types::Int, types::Dict);
        type Return = types::Object;
        const NAME: &str = "nvim_notify";
    }
    /// Signature of [`nvim_ui_term_event`](https://neovim.io/doc/user/api/#nvim_ui_term_event()).
    ///
    /// Since Neovim API level 12.
    pub struct NvimUiTermEvent {
        pub event: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimUiTermEvent {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_ui_term_event";
    }
    /// Signature of [`nvim_create_namespace`](https://neovim.io/doc/user/api/#nvim_create_namespace()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimCreateNamespace {
        pub name: types::String,
    }
    impl types::ApiFn for NvimCreateNamespace {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "nvim_create_namespace";
    }
    /// Signature of [`nvim_get_namespaces`](https://neovim.io/doc/user/api/#nvim_get_namespaces()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimGetNamespaces {}
    impl types::ApiFn for NvimGetNamespaces {
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "nvim_get_namespaces";
    }
    /// Signature of [`nvim_buf_get_extmark_by_id`](https://neovim.io/doc/user/api/#nvim_buf_get_extmark_by_id()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufGetExtmarkById {
        pub ns_id: types::Int,
        pub id: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufGetExtmarkById {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Dict);
        type Return = types::Array;
        const NAME: &str = "nvim_buf_get_extmark_by_id";
    }
    impl types::ApiMethod for NvimBufGetExtmarkById {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Dict);
        type Return = types::Array;
        const NAME: &str = "get_extmark_by_id";
    }
    /// Signature of [`nvim_buf_get_extmarks`](https://neovim.io/doc/user/api/#nvim_buf_get_extmarks()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufGetExtmarks {
        pub ns_id: types::Int,
        pub start: types::Object,
        pub end: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufGetExtmarks {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Object,
            types::Object,
            types::Dict,
        );
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_buf_get_extmarks";
    }
    impl types::ApiMethod for NvimBufGetExtmarks {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Object, types::Object, types::Dict);
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "get_extmarks";
    }
    /// Signature of [`nvim_buf_set_extmark`](https://neovim.io/doc/user/api/#nvim_buf_set_extmark()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufSetExtmark {
        pub ns_id: types::Int,
        pub line: types::Int,
        pub col: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimBufSetExtmark {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Int,
            types::Dict,
        );
        type Return = types::Int;
        const NAME: &str = "nvim_buf_set_extmark";
    }
    impl types::ApiMethod for NvimBufSetExtmark {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Int, types::Dict);
        type Return = types::Int;
        const NAME: &str = "set_extmark";
    }
    /// Signature of [`nvim_buf_del_extmark`](https://neovim.io/doc/user/api/#nvim_buf_del_extmark()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimBufDelExtmark {
        pub ns_id: types::Int,
        pub id: types::Int,
    }
    impl types::ApiFn for NvimBufDelExtmark {
        type Params = (ext_types::Buffer, types::Int, types::Int);
        type Return = types::Bool;
        const NAME: &str = "nvim_buf_del_extmark";
    }
    impl types::ApiMethod for NvimBufDelExtmark {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int);
        type Return = types::Bool;
        const NAME: &str = "del_extmark";
    }
    /// Signature of [`nvim_buf_clear_namespace`](https://neovim.io/doc/user/api/#nvim_buf_clear_namespace()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimBufClearNamespace {
        pub ns_id: types::Int,
        pub line_start: types::Int,
        pub line_end: types::Int,
    }
    impl types::ApiFn for NvimBufClearNamespace {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "nvim_buf_clear_namespace";
    }
    impl types::ApiMethod for NvimBufClearNamespace {
        type On = ext_types::Buffer;
        type Params = (types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "clear_namespace";
    }
    /// Signature of [`nvim_set_decoration_provider`](https://neovim.io/doc/user/api/#nvim_set_decoration_provider()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimSetDecorationProvider {
        pub ns_id: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimSetDecorationProvider {
        type Params = (types::Int, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_set_decoration_provider";
    }
    /// Signature of [`nvim_get_option_value`](https://neovim.io/doc/user/api/#nvim_get_option_value()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimGetOptionValue {
        pub name: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetOptionValue {
        type Params = (types::String, types::Dict);
        type Return = types::Object;
        const NAME: &str = "nvim_get_option_value";
    }
    /// Signature of [`nvim_set_option_value`](https://neovim.io/doc/user/api/#nvim_set_option_value()).
    ///
    /// Since Neovim API level 9.
    pub struct NvimSetOptionValue {
        pub name: types::String,
        pub value: types::Object,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimSetOptionValue {
        type Params = (types::String, types::Object, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_set_option_value";
    }
    /// Signature of [`nvim_get_all_options_info`](https://neovim.io/doc/user/api/#nvim_get_all_options_info()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimGetAllOptionsInfo {}
    impl types::ApiFn for NvimGetAllOptionsInfo {
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "nvim_get_all_options_info";
    }
    /// Signature of [`nvim_get_option_info2`](https://neovim.io/doc/user/api/#nvim_get_option_info2()).
    ///
    /// Since Neovim API level 11.
    pub struct NvimGetOptionInfo2 {
        pub name: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetOptionInfo2 {
        type Params = (types::String, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_option_info2";
    }
    /// Signature of [`nvim_tabpage_list_wins`](https://neovim.io/doc/user/api/#nvim_tabpage_list_wins()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageListWins {}
    impl types::ApiFn for NvimTabpageListWins {
        type Params = (ext_types::Tabpage,);
        type Return = types::ArrayOf<ext_types::Window>;
        const NAME: &str = "nvim_tabpage_list_wins";
    }
    impl types::ApiMethod for NvimTabpageListWins {
        type On = ext_types::Tabpage;
        type Params = ();
        type Return = types::ArrayOf<ext_types::Window>;
        const NAME: &str = "list_wins";
    }
    /// Signature of [`nvim_tabpage_get_var`](https://neovim.io/doc/user/api/#nvim_tabpage_get_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageGetVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimTabpageGetVar {
        type Params = (ext_types::Tabpage, types::String);
        type Return = types::Object;
        const NAME: &str = "nvim_tabpage_get_var";
    }
    impl types::ApiMethod for NvimTabpageGetVar {
        type On = ext_types::Tabpage;
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "get_var";
    }
    /// Signature of [`nvim_tabpage_set_var`](https://neovim.io/doc/user/api/#nvim_tabpage_set_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageSetVar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimTabpageSetVar {
        type Params = (ext_types::Tabpage, types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_tabpage_set_var";
    }
    impl types::ApiMethod for NvimTabpageSetVar {
        type On = ext_types::Tabpage;
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "set_var";
    }
    /// Signature of [`nvim_tabpage_del_var`](https://neovim.io/doc/user/api/#nvim_tabpage_del_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageDelVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimTabpageDelVar {
        type Params = (ext_types::Tabpage, types::String);
        type Return = ();
        const NAME: &str = "nvim_tabpage_del_var";
    }
    impl types::ApiMethod for NvimTabpageDelVar {
        type On = ext_types::Tabpage;
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "del_var";
    }
    /// Signature of [`nvim_tabpage_get_win`](https://neovim.io/doc/user/api/#nvim_tabpage_get_win()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageGetWin {}
    impl types::ApiFn for NvimTabpageGetWin {
        type Params = (ext_types::Tabpage,);
        type Return = ext_types::Window;
        const NAME: &str = "nvim_tabpage_get_win";
    }
    impl types::ApiMethod for NvimTabpageGetWin {
        type On = ext_types::Tabpage;
        type Params = ();
        type Return = ext_types::Window;
        const NAME: &str = "get_win";
    }
    /// Signature of [`nvim_tabpage_set_win`](https://neovim.io/doc/user/api/#nvim_tabpage_set_win()).
    ///
    /// Since Neovim API level 12.
    pub struct NvimTabpageSetWin {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for NvimTabpageSetWin {
        type Params = (ext_types::Tabpage, ext_types::Window);
        type Return = ();
        const NAME: &str = "nvim_tabpage_set_win";
    }
    impl types::ApiMethod for NvimTabpageSetWin {
        type On = ext_types::Tabpage;
        type Params = (ext_types::Window,);
        type Return = ();
        const NAME: &str = "set_win";
    }
    /// Signature of [`nvim_tabpage_get_number`](https://neovim.io/doc/user/api/#nvim_tabpage_get_number()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageGetNumber {}
    impl types::ApiFn for NvimTabpageGetNumber {
        type Params = (ext_types::Tabpage,);
        type Return = types::Int;
        const NAME: &str = "nvim_tabpage_get_number";
    }
    impl types::ApiMethod for NvimTabpageGetNumber {
        type On = ext_types::Tabpage;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_number";
    }
    /// Signature of [`nvim_tabpage_is_valid`](https://neovim.io/doc/user/api/#nvim_tabpage_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimTabpageIsValid {}
    impl types::ApiFn for NvimTabpageIsValid {
        type Params = (ext_types::Tabpage,);
        type Return = types::Bool;
        const NAME: &str = "nvim_tabpage_is_valid";
    }
    impl types::ApiMethod for NvimTabpageIsValid {
        type On = ext_types::Tabpage;
        type Params = ();
        type Return = types::Bool;
        const NAME: &str = "is_valid";
    }
    /// Signature of [`nvim_open_tabpage`](https://neovim.io/doc/user/api/#nvim_open_tabpage()).
    ///
    /// Since Neovim API level 14.
    pub struct NvimOpenTabpage {
        pub buf: ext_types::Buffer,
        pub enter: types::Bool,
        pub config: types::Dict,
    }
    impl types::ApiFn for NvimOpenTabpage {
        type Params = (ext_types::Buffer, types::Bool, types::Dict);
        type Return = ext_types::Tabpage;
        const NAME: &str = "nvim_open_tabpage";
    }
    /// Signature of [`nvim_ui_attach`](https://neovim.io/doc/user/api/#nvim_ui_attach()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimUiAttach {
        pub width: types::Int,
        pub height: types::Int,
        pub options: types::Dict,
    }
    impl types::ApiFn for NvimUiAttach {
        type Params = (types::Int, types::Int, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_ui_attach";
    }
    /// Signature of [`ui_attach`](https://neovim.io/doc/user/api/#ui_attach()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct UiAttach {
        pub width: types::Int,
        pub height: types::Int,
        pub enable_rgb: types::Bool,
    }
    impl types::ApiFn for UiAttach {
        type Params = (types::Int, types::Int, types::Bool);
        type Return = ();
        const NAME: &str = "ui_attach";
    }
    /// Signature of [`nvim_ui_set_focus`](https://neovim.io/doc/user/api/#nvim_ui_set_focus()).
    ///
    /// Since Neovim API level 11.
    pub struct NvimUiSetFocus {
        pub gained: types::Bool,
    }
    impl types::ApiFn for NvimUiSetFocus {
        type Params = (types::Bool,);
        type Return = ();
        const NAME: &str = "nvim_ui_set_focus";
    }
    /// Signature of [`nvim_ui_detach`](https://neovim.io/doc/user/api/#nvim_ui_detach()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimUiDetach {}
    impl types::ApiFn for NvimUiDetach {
        type Params = ();
        type Return = ();
        const NAME: &str = "nvim_ui_detach";
    }
    /// Signature of [`nvim_ui_try_resize`](https://neovim.io/doc/user/api/#nvim_ui_try_resize()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimUiTryResize {
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::ApiFn for NvimUiTryResize {
        type Params = (types::Int, types::Int);
        type Return = ();
        const NAME: &str = "nvim_ui_try_resize";
    }
    /// Signature of [`nvim_ui_set_option`](https://neovim.io/doc/user/api/#nvim_ui_set_option()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimUiSetOption {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimUiSetOption {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_ui_set_option";
    }
    /// Signature of [`nvim_ui_try_resize_grid`](https://neovim.io/doc/user/api/#nvim_ui_try_resize_grid()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimUiTryResizeGrid {
        pub grid: types::Int,
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::ApiFn for NvimUiTryResizeGrid {
        type Params = (types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "nvim_ui_try_resize_grid";
    }
    /// Signature of [`nvim_ui_pum_set_height`](https://neovim.io/doc/user/api/#nvim_ui_pum_set_height()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimUiPumSetHeight {
        pub height: types::Int,
    }
    impl types::ApiFn for NvimUiPumSetHeight {
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "nvim_ui_pum_set_height";
    }
    /// Signature of [`nvim_ui_pum_set_bounds`](https://neovim.io/doc/user/api/#nvim_ui_pum_set_bounds()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimUiPumSetBounds {
        pub width: types::Float,
        pub height: types::Float,
        pub row: types::Float,
        pub col: types::Float,
    }
    impl types::ApiFn for NvimUiPumSetBounds {
        type Params = (types::Float, types::Float, types::Float, types::Float);
        type Return = ();
        const NAME: &str = "nvim_ui_pum_set_bounds";
    }
    /// Signature of [`nvim_ui_send`](https://neovim.io/doc/user/api/#nvim_ui_send()).
    ///
    /// Since Neovim API level 14.
    pub struct NvimUiSend {
        pub content: types::String,
    }
    impl types::ApiFn for NvimUiSend {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_ui_send";
    }
    /// Signature of [`nvim_get_hl_id_by_name`](https://neovim.io/doc/user/api/#nvim_get_hl_id_by_name()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimGetHlIdByName {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetHlIdByName {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "nvim_get_hl_id_by_name";
    }
    /// Signature of [`nvim_get_hl`](https://neovim.io/doc/user/api/#nvim_get_hl()).
    ///
    /// Since Neovim API level 11.
    pub struct NvimGetHl {
        pub ns_id: types::Int,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetHl {
        type Params = (types::Int, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_hl";
    }
    /// Signature of [`nvim_set_hl`](https://neovim.io/doc/user/api/#nvim_set_hl()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimSetHl {
        pub ns_id: types::Int,
        pub name: types::String,
        pub val: types::Dict,
    }
    impl types::ApiFn for NvimSetHl {
        type Params = (types::Int, types::String, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_set_hl";
    }
    /// Signature of [`nvim_get_hl_ns`](https://neovim.io/doc/user/api/#nvim_get_hl_ns()).
    ///
    /// Since Neovim API level 12.
    pub struct NvimGetHlNs {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetHlNs {
        type Params = (types::Dict,);
        type Return = types::Int;
        const NAME: &str = "nvim_get_hl_ns";
    }
    /// Signature of [`nvim_set_hl_ns`](https://neovim.io/doc/user/api/#nvim_set_hl_ns()).
    ///
    /// Since Neovim API level 10.
    pub struct NvimSetHlNs {
        pub ns_id: types::Int,
    }
    impl types::ApiFn for NvimSetHlNs {
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "nvim_set_hl_ns";
    }
    /// Signature of [`nvim_set_hl_ns_fast`](https://neovim.io/doc/user/api/#nvim_set_hl_ns_fast()).
    ///
    /// Since Neovim API level 10.
    pub struct NvimSetHlNsFast {
        pub ns_id: types::Int,
    }
    impl types::ApiFn for NvimSetHlNsFast {
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "nvim_set_hl_ns_fast";
    }
    /// Signature of [`nvim_feedkeys`](https://neovim.io/doc/user/api/#nvim_feedkeys()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimFeedkeys {
        pub keys: types::String,
        pub mode: types::String,
        pub escape_ks: types::Bool,
    }
    impl types::ApiFn for NvimFeedkeys {
        type Params = (types::String, types::String, types::Bool);
        type Return = ();
        const NAME: &str = "nvim_feedkeys";
    }
    /// Signature of [`nvim_input`](https://neovim.io/doc/user/api/#nvim_input()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimInput {
        pub keys: types::String,
    }
    impl types::ApiFn for NvimInput {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "nvim_input";
    }
    /// Signature of [`nvim_input_mouse`](https://neovim.io/doc/user/api/#nvim_input_mouse()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimInputMouse {
        pub button: types::String,
        pub action: types::String,
        pub modifier: types::String,
        pub grid: types::Int,
        pub row: types::Int,
        pub col: types::Int,
    }
    impl types::ApiFn for NvimInputMouse {
        type Params = (
            types::String,
            types::String,
            types::String,
            types::Int,
            types::Int,
            types::Int,
        );
        type Return = ();
        const NAME: &str = "nvim_input_mouse";
    }
    /// Signature of [`nvim_replace_termcodes`](https://neovim.io/doc/user/api/#nvim_replace_termcodes()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimReplaceTermcodes {
        pub str: types::String,
        pub from_part: types::Bool,
        pub do_lt: types::Bool,
        pub special: types::Bool,
    }
    impl types::ApiFn for NvimReplaceTermcodes {
        type Params = (types::String, types::Bool, types::Bool, types::Bool);
        type Return = types::String;
        const NAME: &str = "nvim_replace_termcodes";
    }
    /// Signature of [`nvim_exec_lua`](https://neovim.io/doc/user/api/#nvim_exec_lua()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimExecLua {
        pub code: types::String,
        pub args: types::Array,
    }
    impl types::ApiFn for NvimExecLua {
        type Params = (types::String, types::Array);
        type Return = types::Object;
        const NAME: &str = "nvim_exec_lua";
    }
    /// Signature of [`nvim_strwidth`](https://neovim.io/doc/user/api/#nvim_strwidth()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimStrwidth {
        pub text: types::String,
    }
    impl types::ApiFn for NvimStrwidth {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "nvim_strwidth";
    }
    /// Signature of [`nvim_list_runtime_paths`](https://neovim.io/doc/user/api/#nvim_list_runtime_paths()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimListRuntimePaths {}
    impl types::ApiFn for NvimListRuntimePaths {
        type Params = ();
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "nvim_list_runtime_paths";
    }
    /// Signature of [`nvim_get_runtime_file`](https://neovim.io/doc/user/api/#nvim_get_runtime_file()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimGetRuntimeFile {
        pub name: types::String,
        pub all: types::Bool,
    }
    impl types::ApiFn for NvimGetRuntimeFile {
        type Params = (types::String, types::Bool);
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "nvim_get_runtime_file";
    }
    /// Signature of [`nvim_set_current_dir`](https://neovim.io/doc/user/api/#nvim_set_current_dir()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetCurrentDir {
        pub dir: types::String,
    }
    impl types::ApiFn for NvimSetCurrentDir {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_set_current_dir";
    }
    /// Signature of [`nvim_get_current_line`](https://neovim.io/doc/user/api/#nvim_get_current_line()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetCurrentLine {}
    impl types::ApiFn for NvimGetCurrentLine {
        type Params = ();
        type Return = types::String;
        const NAME: &str = "nvim_get_current_line";
    }
    /// Signature of [`nvim_set_current_line`](https://neovim.io/doc/user/api/#nvim_set_current_line()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetCurrentLine {
        pub line: types::String,
    }
    impl types::ApiFn for NvimSetCurrentLine {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_set_current_line";
    }
    /// Signature of [`nvim_del_current_line`](https://neovim.io/doc/user/api/#nvim_del_current_line()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimDelCurrentLine {}
    impl types::ApiFn for NvimDelCurrentLine {
        type Params = ();
        type Return = ();
        const NAME: &str = "nvim_del_current_line";
    }
    /// Signature of [`nvim_get_var`](https://neovim.io/doc/user/api/#nvim_get_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetVar {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "nvim_get_var";
    }
    /// Signature of [`nvim_set_var`](https://neovim.io/doc/user/api/#nvim_set_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetVar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimSetVar {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_set_var";
    }
    /// Signature of [`nvim_del_var`](https://neovim.io/doc/user/api/#nvim_del_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimDelVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimDelVar {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_del_var";
    }
    /// Signature of [`nvim_get_vvar`](https://neovim.io/doc/user/api/#nvim_get_vvar()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetVvar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetVvar {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "nvim_get_vvar";
    }
    /// Signature of [`nvim_set_vvar`](https://neovim.io/doc/user/api/#nvim_set_vvar()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimSetVvar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimSetVvar {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_set_vvar";
    }
    /// Signature of [`nvim_echo`](https://neovim.io/doc/user/api/#nvim_echo()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimEcho {
        pub chunks: types::ArrayOf<types::Array>,
        pub history: types::Bool,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimEcho {
        type Params = (types::ArrayOf<types::Array>, types::Bool, types::Dict);
        type Return = types::Object;
        const NAME: &str = "nvim_echo";
    }
    /// Signature of [`nvim_list_bufs`](https://neovim.io/doc/user/api/#nvim_list_bufs()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimListBufs {}
    impl types::ApiFn for NvimListBufs {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Buffer>;
        const NAME: &str = "nvim_list_bufs";
    }
    /// Signature of [`nvim_get_current_buf`](https://neovim.io/doc/user/api/#nvim_get_current_buf()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetCurrentBuf {}
    impl types::ApiFn for NvimGetCurrentBuf {
        type Params = ();
        type Return = ext_types::Buffer;
        const NAME: &str = "nvim_get_current_buf";
    }
    /// Signature of [`nvim_set_current_buf`](https://neovim.io/doc/user/api/#nvim_set_current_buf()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetCurrentBuf {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for NvimSetCurrentBuf {
        type Params = (ext_types::Buffer,);
        type Return = ();
        const NAME: &str = "nvim_set_current_buf";
    }
    /// Signature of [`nvim_list_wins`](https://neovim.io/doc/user/api/#nvim_list_wins()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimListWins {}
    impl types::ApiFn for NvimListWins {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Window>;
        const NAME: &str = "nvim_list_wins";
    }
    /// Signature of [`nvim_get_current_win`](https://neovim.io/doc/user/api/#nvim_get_current_win()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetCurrentWin {}
    impl types::ApiFn for NvimGetCurrentWin {
        type Params = ();
        type Return = ext_types::Window;
        const NAME: &str = "nvim_get_current_win";
    }
    /// Signature of [`nvim_set_current_win`](https://neovim.io/doc/user/api/#nvim_set_current_win()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetCurrentWin {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for NvimSetCurrentWin {
        type Params = (ext_types::Window,);
        type Return = ();
        const NAME: &str = "nvim_set_current_win";
    }
    /// Signature of [`nvim_create_buf`](https://neovim.io/doc/user/api/#nvim_create_buf()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimCreateBuf {
        pub listed: types::Bool,
        pub scratch: types::Bool,
    }
    impl types::ApiFn for NvimCreateBuf {
        type Params = (types::Bool, types::Bool);
        type Return = ext_types::Buffer;
        const NAME: &str = "nvim_create_buf";
    }
    /// Signature of [`nvim_open_term`](https://neovim.io/doc/user/api/#nvim_open_term()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimOpenTerm {
        pub buf: ext_types::Buffer,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimOpenTerm {
        type Params = (ext_types::Buffer, types::Dict);
        type Return = types::Int;
        const NAME: &str = "nvim_open_term";
    }
    /// Signature of [`nvim_chan_send`](https://neovim.io/doc/user/api/#nvim_chan_send()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimChanSend {
        pub chan: types::Int,
        pub data: types::String,
    }
    impl types::ApiFn for NvimChanSend {
        type Params = (types::Int, types::String);
        type Return = ();
        const NAME: &str = "nvim_chan_send";
    }
    /// Signature of [`nvim_list_tabpages`](https://neovim.io/doc/user/api/#nvim_list_tabpages()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimListTabpages {}
    impl types::ApiFn for NvimListTabpages {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Tabpage>;
        const NAME: &str = "nvim_list_tabpages";
    }
    /// Signature of [`nvim_get_current_tabpage`](https://neovim.io/doc/user/api/#nvim_get_current_tabpage()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetCurrentTabpage {}
    impl types::ApiFn for NvimGetCurrentTabpage {
        type Params = ();
        type Return = ext_types::Tabpage;
        const NAME: &str = "nvim_get_current_tabpage";
    }
    /// Signature of [`nvim_set_current_tabpage`](https://neovim.io/doc/user/api/#nvim_set_current_tabpage()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimSetCurrentTabpage {
        pub tabpage: ext_types::Tabpage,
    }
    impl types::ApiFn for NvimSetCurrentTabpage {
        type Params = (ext_types::Tabpage,);
        type Return = ();
        const NAME: &str = "nvim_set_current_tabpage";
    }
    /// Signature of [`nvim_paste`](https://neovim.io/doc/user/api/#nvim_paste()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimPaste {
        pub data: types::String,
        pub crlf: types::Bool,
        pub phase: types::Int,
    }
    impl types::ApiFn for NvimPaste {
        type Params = (types::String, types::Bool, types::Int);
        type Return = types::Bool;
        const NAME: &str = "nvim_paste";
    }
    /// Signature of [`nvim_put`](https://neovim.io/doc/user/api/#nvim_put()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimPut {
        pub lines: types::ArrayOf<types::String>,
        pub r#type: types::String,
        pub after: types::Bool,
        pub follow: types::Bool,
    }
    impl types::ApiFn for NvimPut {
        type Params = (
            types::ArrayOf<types::String>,
            types::String,
            types::Bool,
            types::Bool,
        );
        type Return = ();
        const NAME: &str = "nvim_put";
    }
    /// Signature of [`nvim_get_color_by_name`](https://neovim.io/doc/user/api/#nvim_get_color_by_name()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetColorByName {
        pub name: types::String,
    }
    impl types::ApiFn for NvimGetColorByName {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "nvim_get_color_by_name";
    }
    /// Signature of [`nvim_get_color_map`](https://neovim.io/doc/user/api/#nvim_get_color_map()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetColorMap {}
    impl types::ApiFn for NvimGetColorMap {
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "nvim_get_color_map";
    }
    /// Signature of [`nvim_get_context`](https://neovim.io/doc/user/api/#nvim_get_context()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimGetContext {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetContext {
        type Params = (types::Dict,);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_context";
    }
    /// Signature of [`nvim_load_context`](https://neovim.io/doc/user/api/#nvim_load_context()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimLoadContext {
        pub dict: types::Dict,
    }
    impl types::ApiFn for NvimLoadContext {
        type Params = (types::Dict,);
        type Return = types::Object;
        const NAME: &str = "nvim_load_context";
    }
    /// Signature of [`nvim_get_mode`](https://neovim.io/doc/user/api/#nvim_get_mode()).
    ///
    /// Since Neovim API level 2.
    pub struct NvimGetMode {}
    impl types::ApiFn for NvimGetMode {
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "nvim_get_mode";
    }
    /// Signature of [`nvim_get_keymap`](https://neovim.io/doc/user/api/#nvim_get_keymap()).
    ///
    /// Since Neovim API level 3.
    pub struct NvimGetKeymap {
        pub mode: types::String,
    }
    impl types::ApiFn for NvimGetKeymap {
        type Params = (types::String,);
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_get_keymap";
    }
    /// Signature of [`nvim_set_keymap`](https://neovim.io/doc/user/api/#nvim_set_keymap()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimSetKeymap {
        pub mode: types::String,
        pub lhs: types::String,
        pub rhs: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimSetKeymap {
        type Params = (types::String, types::String, types::String, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_set_keymap";
    }
    /// Signature of [`nvim_del_keymap`](https://neovim.io/doc/user/api/#nvim_del_keymap()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimDelKeymap {
        pub mode: types::String,
        pub lhs: types::String,
    }
    impl types::ApiFn for NvimDelKeymap {
        type Params = (types::String, types::String);
        type Return = ();
        const NAME: &str = "nvim_del_keymap";
    }
    /// Signature of [`nvim_get_api_info`](https://neovim.io/doc/user/api/#nvim_get_api_info()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimGetApiInfo {}
    impl types::ApiFn for NvimGetApiInfo {
        type Params = ();
        type Return = types::FixedArrayOf<types::Object, 2usize>;
        const NAME: &str = "nvim_get_api_info";
    }
    /// Signature of [`nvim_set_client_info`](https://neovim.io/doc/user/api/#nvim_set_client_info()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimSetClientInfo {
        pub name: types::String,
        pub version: types::Dict,
        pub r#type: types::String,
        pub methods: types::Dict,
        pub attributes: types::Dict,
    }
    impl types::ApiFn for NvimSetClientInfo {
        type Params = (
            types::String,
            types::Dict,
            types::String,
            types::Dict,
            types::Dict,
        );
        type Return = ();
        const NAME: &str = "nvim_set_client_info";
    }
    /// Signature of [`nvim_get_chan_info`](https://neovim.io/doc/user/api/#nvim_get_chan_info()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimGetChanInfo {
        pub chan: types::Int,
    }
    impl types::ApiFn for NvimGetChanInfo {
        type Params = (types::Int,);
        type Return = types::Dict;
        const NAME: &str = "nvim_get_chan_info";
    }
    /// Signature of [`nvim_list_chans`](https://neovim.io/doc/user/api/#nvim_list_chans()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimListChans {}
    impl types::ApiFn for NvimListChans {
        type Params = ();
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_list_chans";
    }
    /// Signature of [`nvim_list_uis`](https://neovim.io/doc/user/api/#nvim_list_uis()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimListUis {}
    impl types::ApiFn for NvimListUis {
        type Params = ();
        type Return = types::ArrayOf<types::Dict>;
        const NAME: &str = "nvim_list_uis";
    }
    /// Signature of [`nvim_get_proc_children`](https://neovim.io/doc/user/api/#nvim_get_proc_children()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimGetProcChildren {
        pub pid: types::Int,
    }
    impl types::ApiFn for NvimGetProcChildren {
        type Params = (types::Int,);
        type Return = types::Array;
        const NAME: &str = "nvim_get_proc_children";
    }
    /// Signature of [`nvim_get_proc`](https://neovim.io/doc/user/api/#nvim_get_proc()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimGetProc {
        pub pid: types::Int,
    }
    impl types::ApiFn for NvimGetProc {
        type Params = (types::Int,);
        type Return = types::Object;
        const NAME: &str = "nvim_get_proc";
    }
    /// Signature of [`nvim_select_popupmenu_item`](https://neovim.io/doc/user/api/#nvim_select_popupmenu_item()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimSelectPopupmenuItem {
        pub item: types::Int,
        pub insert: types::Bool,
        pub finish: types::Bool,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimSelectPopupmenuItem {
        type Params = (types::Int, types::Bool, types::Bool, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_select_popupmenu_item";
    }
    /// Signature of [`nvim_del_mark`](https://neovim.io/doc/user/api/#nvim_del_mark()).
    ///
    /// Since Neovim API level 8.
    pub struct NvimDelMark {
        pub name: types::String,
    }
    impl types::ApiFn for NvimDelMark {
        type Params = (types::String,);
        type Return = types::Bool;
        const NAME: &str = "nvim_del_mark";
    }
    /// Signature of [`nvim_get_mark`](https://neovim.io/doc/user/api/#nvim_get_mark()).
    ///
    /// Since Neovim API level 8.
    pub struct NvimGetMark {
        pub name: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimGetMark {
        type Params = (types::String, types::Dict);
        type Return = types::Array;
        const NAME: &str = "nvim_get_mark";
    }
    /// Signature of [`nvim_eval_statusline`](https://neovim.io/doc/user/api/#nvim_eval_statusline()).
    ///
    /// Since Neovim API level 8.
    pub struct NvimEvalStatusline {
        pub str: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimEvalStatusline {
        type Params = (types::String, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_eval_statusline";
    }
    /// Signature of [`nvim_exec2`](https://neovim.io/doc/user/api/#nvim_exec2()).
    ///
    /// Since Neovim API level 11.
    pub struct NvimExec2 {
        pub src: types::String,
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimExec2 {
        type Params = (types::String, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_exec2";
    }
    /// Signature of [`nvim_command`](https://neovim.io/doc/user/api/#nvim_command()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimCommand {
        pub cmd: types::String,
    }
    impl types::ApiFn for NvimCommand {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "nvim_command";
    }
    /// Signature of [`nvim_eval`](https://neovim.io/doc/user/api/#nvim_eval()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimEval {
        pub expr: types::String,
    }
    impl types::ApiFn for NvimEval {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "nvim_eval";
    }
    /// Signature of [`nvim_call_function`](https://neovim.io/doc/user/api/#nvim_call_function()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimCallFunction {
        pub r#fn: types::String,
        pub args: types::Array,
    }
    impl types::ApiFn for NvimCallFunction {
        type Params = (types::String, types::Array);
        type Return = types::Object;
        const NAME: &str = "nvim_call_function";
    }
    /// Signature of [`nvim_call_dict_function`](https://neovim.io/doc/user/api/#nvim_call_dict_function()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimCallDictFunction {
        pub dict: types::Object,
        pub r#fn: types::String,
        pub args: types::Array,
    }
    impl types::ApiFn for NvimCallDictFunction {
        type Params = (types::Object, types::String, types::Array);
        type Return = types::Object;
        const NAME: &str = "nvim_call_dict_function";
    }
    /// Signature of [`nvim_parse_expression`](https://neovim.io/doc/user/api/#nvim_parse_expression()).
    ///
    /// Since Neovim API level 4.
    pub struct NvimParseExpression {
        pub expr: types::String,
        pub flags: types::String,
        pub hl: types::Bool,
    }
    impl types::ApiFn for NvimParseExpression {
        type Params = (types::String, types::String, types::Bool);
        type Return = types::Dict;
        const NAME: &str = "nvim_parse_expression";
    }
    /// Signature of [`nvim_open_win`](https://neovim.io/doc/user/api/#nvim_open_win()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimOpenWin {
        pub buf: ext_types::Buffer,
        pub enter: types::Bool,
        pub config: types::Dict,
    }
    impl types::ApiFn for NvimOpenWin {
        type Params = (ext_types::Buffer, types::Bool, types::Dict);
        type Return = ext_types::Window;
        const NAME: &str = "nvim_open_win";
    }
    /// Signature of [`nvim_win_set_config`](https://neovim.io/doc/user/api/#nvim_win_set_config()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimWinSetConfig {
        pub config: types::Dict,
    }
    impl types::ApiFn for NvimWinSetConfig {
        type Params = (ext_types::Window, types::Dict);
        type Return = ();
        const NAME: &str = "nvim_win_set_config";
    }
    impl types::ApiMethod for NvimWinSetConfig {
        type On = ext_types::Window;
        type Params = (types::Dict,);
        type Return = ();
        const NAME: &str = "set_config";
    }
    /// Signature of [`nvim_win_get_config`](https://neovim.io/doc/user/api/#nvim_win_get_config()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimWinGetConfig {}
    impl types::ApiFn for NvimWinGetConfig {
        type Params = (ext_types::Window,);
        type Return = types::Dict;
        const NAME: &str = "nvim_win_get_config";
    }
    impl types::ApiMethod for NvimWinGetConfig {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "get_config";
    }
    /// Signature of [`nvim_win_get_buf`](https://neovim.io/doc/user/api/#nvim_win_get_buf()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetBuf {}
    impl types::ApiFn for NvimWinGetBuf {
        type Params = (ext_types::Window,);
        type Return = ext_types::Buffer;
        const NAME: &str = "nvim_win_get_buf";
    }
    impl types::ApiMethod for NvimWinGetBuf {
        type On = ext_types::Window;
        type Params = ();
        type Return = ext_types::Buffer;
        const NAME: &str = "get_buf";
    }
    /// Signature of [`nvim_win_set_buf`](https://neovim.io/doc/user/api/#nvim_win_set_buf()).
    ///
    /// Since Neovim API level 5.
    pub struct NvimWinSetBuf {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for NvimWinSetBuf {
        type Params = (ext_types::Window, ext_types::Buffer);
        type Return = ();
        const NAME: &str = "nvim_win_set_buf";
    }
    impl types::ApiMethod for NvimWinSetBuf {
        type On = ext_types::Window;
        type Params = (ext_types::Buffer,);
        type Return = ();
        const NAME: &str = "set_buf";
    }
    /// Signature of [`nvim_win_get_cursor`](https://neovim.io/doc/user/api/#nvim_win_get_cursor()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetCursor {}
    impl types::ApiFn for NvimWinGetCursor {
        type Params = (ext_types::Window,);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "nvim_win_get_cursor";
    }
    impl types::ApiMethod for NvimWinGetCursor {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "get_cursor";
    }
    /// Signature of [`nvim_win_set_cursor`](https://neovim.io/doc/user/api/#nvim_win_set_cursor()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinSetCursor {
        pub pos: types::FixedArrayOf<types::Int, 2usize>,
    }
    impl types::ApiFn for NvimWinSetCursor {
        type Params = (ext_types::Window, types::FixedArrayOf<types::Int, 2usize>);
        type Return = ();
        const NAME: &str = "nvim_win_set_cursor";
    }
    impl types::ApiMethod for NvimWinSetCursor {
        type On = ext_types::Window;
        type Params = (types::FixedArrayOf<types::Int, 2usize>,);
        type Return = ();
        const NAME: &str = "set_cursor";
    }
    /// Signature of [`nvim_win_get_height`](https://neovim.io/doc/user/api/#nvim_win_get_height()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetHeight {}
    impl types::ApiFn for NvimWinGetHeight {
        type Params = (ext_types::Window,);
        type Return = types::Int;
        const NAME: &str = "nvim_win_get_height";
    }
    impl types::ApiMethod for NvimWinGetHeight {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_height";
    }
    /// Signature of [`nvim_win_set_height`](https://neovim.io/doc/user/api/#nvim_win_set_height()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinSetHeight {
        pub height: types::Int,
    }
    impl types::ApiFn for NvimWinSetHeight {
        type Params = (ext_types::Window, types::Int);
        type Return = ();
        const NAME: &str = "nvim_win_set_height";
    }
    impl types::ApiMethod for NvimWinSetHeight {
        type On = ext_types::Window;
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "set_height";
    }
    /// Signature of [`nvim_win_get_width`](https://neovim.io/doc/user/api/#nvim_win_get_width()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetWidth {}
    impl types::ApiFn for NvimWinGetWidth {
        type Params = (ext_types::Window,);
        type Return = types::Int;
        const NAME: &str = "nvim_win_get_width";
    }
    impl types::ApiMethod for NvimWinGetWidth {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_width";
    }
    /// Signature of [`nvim_win_set_width`](https://neovim.io/doc/user/api/#nvim_win_set_width()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinSetWidth {
        pub width: types::Int,
    }
    impl types::ApiFn for NvimWinSetWidth {
        type Params = (ext_types::Window, types::Int);
        type Return = ();
        const NAME: &str = "nvim_win_set_width";
    }
    impl types::ApiMethod for NvimWinSetWidth {
        type On = ext_types::Window;
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "set_width";
    }
    /// Signature of [`nvim_win_get_var`](https://neovim.io/doc/user/api/#nvim_win_get_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimWinGetVar {
        type Params = (ext_types::Window, types::String);
        type Return = types::Object;
        const NAME: &str = "nvim_win_get_var";
    }
    impl types::ApiMethod for NvimWinGetVar {
        type On = ext_types::Window;
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "get_var";
    }
    /// Signature of [`nvim_win_set_var`](https://neovim.io/doc/user/api/#nvim_win_set_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinSetVar {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for NvimWinSetVar {
        type Params = (ext_types::Window, types::String, types::Object);
        type Return = ();
        const NAME: &str = "nvim_win_set_var";
    }
    impl types::ApiMethod for NvimWinSetVar {
        type On = ext_types::Window;
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "set_var";
    }
    /// Signature of [`nvim_win_del_var`](https://neovim.io/doc/user/api/#nvim_win_del_var()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinDelVar {
        pub name: types::String,
    }
    impl types::ApiFn for NvimWinDelVar {
        type Params = (ext_types::Window, types::String);
        type Return = ();
        const NAME: &str = "nvim_win_del_var";
    }
    impl types::ApiMethod for NvimWinDelVar {
        type On = ext_types::Window;
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "del_var";
    }
    /// Signature of [`nvim_win_get_position`](https://neovim.io/doc/user/api/#nvim_win_get_position()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetPosition {}
    impl types::ApiFn for NvimWinGetPosition {
        type Params = (ext_types::Window,);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "nvim_win_get_position";
    }
    impl types::ApiMethod for NvimWinGetPosition {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "get_position";
    }
    /// Signature of [`nvim_win_get_tabpage`](https://neovim.io/doc/user/api/#nvim_win_get_tabpage()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetTabpage {}
    impl types::ApiFn for NvimWinGetTabpage {
        type Params = (ext_types::Window,);
        type Return = ext_types::Tabpage;
        const NAME: &str = "nvim_win_get_tabpage";
    }
    impl types::ApiMethod for NvimWinGetTabpage {
        type On = ext_types::Window;
        type Params = ();
        type Return = ext_types::Tabpage;
        const NAME: &str = "get_tabpage";
    }
    /// Signature of [`nvim_win_get_number`](https://neovim.io/doc/user/api/#nvim_win_get_number()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinGetNumber {}
    impl types::ApiFn for NvimWinGetNumber {
        type Params = (ext_types::Window,);
        type Return = types::Int;
        const NAME: &str = "nvim_win_get_number";
    }
    impl types::ApiMethod for NvimWinGetNumber {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::Int;
        const NAME: &str = "get_number";
    }
    /// Signature of [`nvim_win_is_valid`](https://neovim.io/doc/user/api/#nvim_win_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub struct NvimWinIsValid {}
    impl types::ApiFn for NvimWinIsValid {
        type Params = (ext_types::Window,);
        type Return = types::Bool;
        const NAME: &str = "nvim_win_is_valid";
    }
    impl types::ApiMethod for NvimWinIsValid {
        type On = ext_types::Window;
        type Params = ();
        type Return = types::Bool;
        const NAME: &str = "is_valid";
    }
    /// Signature of [`nvim_win_hide`](https://neovim.io/doc/user/api/#nvim_win_hide()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimWinHide {}
    impl types::ApiFn for NvimWinHide {
        type Params = (ext_types::Window,);
        type Return = ();
        const NAME: &str = "nvim_win_hide";
    }
    impl types::ApiMethod for NvimWinHide {
        type On = ext_types::Window;
        type Params = ();
        type Return = ();
        const NAME: &str = "hide";
    }
    /// Signature of [`nvim_win_close`](https://neovim.io/doc/user/api/#nvim_win_close()).
    ///
    /// Since Neovim API level 6.
    pub struct NvimWinClose {
        pub force: types::Bool,
    }
    impl types::ApiFn for NvimWinClose {
        type Params = (ext_types::Window, types::Bool);
        type Return = ();
        const NAME: &str = "nvim_win_close";
    }
    impl types::ApiMethod for NvimWinClose {
        type On = ext_types::Window;
        type Params = (types::Bool,);
        type Return = ();
        const NAME: &str = "close";
    }
    /// Signature of [`nvim_win_call`](https://neovim.io/doc/user/api/#nvim_win_call()).
    ///
    /// Since Neovim API level 7.
    pub struct NvimWinCall {
        pub fun: types::Ref,
    }
    impl types::ApiFn for NvimWinCall {
        type Params = (ext_types::Window, types::Ref);
        type Return = types::Object;
        const NAME: &str = "nvim_win_call";
    }
    impl types::ApiMethod for NvimWinCall {
        type On = ext_types::Window;
        type Params = (types::Ref,);
        type Return = types::Object;
        const NAME: &str = "call";
    }
    /// Signature of [`nvim_win_set_hl_ns`](https://neovim.io/doc/user/api/#nvim_win_set_hl_ns()).
    ///
    /// Since Neovim API level 10.
    pub struct NvimWinSetHlNs {
        pub ns_id: types::Int,
    }
    impl types::ApiFn for NvimWinSetHlNs {
        type Params = (ext_types::Window, types::Int);
        type Return = ();
        const NAME: &str = "nvim_win_set_hl_ns";
    }
    impl types::ApiMethod for NvimWinSetHlNs {
        type On = ext_types::Window;
        type Params = (types::Int,);
        type Return = ();
        const NAME: &str = "set_hl_ns";
    }
    /// Signature of [`nvim_win_text_height`](https://neovim.io/doc/user/api/#nvim_win_text_height()).
    ///
    /// Since Neovim API level 12.
    pub struct NvimWinTextHeight {
        pub opts: types::Dict,
    }
    impl types::ApiFn for NvimWinTextHeight {
        type Params = (ext_types::Window, types::Dict);
        type Return = types::Dict;
        const NAME: &str = "nvim_win_text_height";
    }
    impl types::ApiMethod for NvimWinTextHeight {
        type On = ext_types::Window;
        type Params = (types::Dict,);
        type Return = types::Dict;
        const NAME: &str = "text_height";
    }
    /// Signature of [`buffer_line_count`](https://neovim.io/doc/user/api/#buffer_line_count()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferLineCount {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for BufferLineCount {
        type Params = (ext_types::Buffer,);
        type Return = types::Int;
        const NAME: &str = "buffer_line_count";
    }
    /// Signature of [`buffer_get_lines`](https://neovim.io/doc/user/api/#buffer_get_lines()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetLines {
        pub buf: ext_types::Buffer,
        pub start: types::Int,
        pub end: types::Int,
        pub strict_indexing: types::Bool,
    }
    impl types::ApiFn for BufferGetLines {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Bool);
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "buffer_get_lines";
    }
    /// Signature of [`buffer_set_lines`](https://neovim.io/doc/user/api/#buffer_set_lines()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetLines {
        pub buf: ext_types::Buffer,
        pub start: types::Int,
        pub end: types::Int,
        pub strict_indexing: types::Bool,
        pub replacement: types::ArrayOf<types::String>,
    }
    impl types::ApiFn for BufferSetLines {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::Int,
            types::Bool,
            types::ArrayOf<types::String>,
        );
        type Return = ();
        const NAME: &str = "buffer_set_lines";
    }
    /// Signature of [`buffer_get_var`](https://neovim.io/doc/user/api/#buffer_get_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetVar {
        pub buf: ext_types::Buffer,
        pub name: types::String,
    }
    impl types::ApiFn for BufferGetVar {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Object;
        const NAME: &str = "buffer_get_var";
    }
    /// Signature of [`buffer_get_name`](https://neovim.io/doc/user/api/#buffer_get_name()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetName {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for BufferGetName {
        type Params = (ext_types::Buffer,);
        type Return = types::String;
        const NAME: &str = "buffer_get_name";
    }
    /// Signature of [`buffer_set_name`](https://neovim.io/doc/user/api/#buffer_set_name()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetName {
        pub buf: ext_types::Buffer,
        pub name: types::String,
    }
    impl types::ApiFn for BufferSetName {
        type Params = (ext_types::Buffer, types::String);
        type Return = ();
        const NAME: &str = "buffer_set_name";
    }
    /// Signature of [`buffer_is_valid`](https://neovim.io/doc/user/api/#buffer_is_valid()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferIsValid {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for BufferIsValid {
        type Params = (ext_types::Buffer,);
        type Return = types::Bool;
        const NAME: &str = "buffer_is_valid";
    }
    /// Signature of [`buffer_get_mark`](https://neovim.io/doc/user/api/#buffer_get_mark()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetMark {
        pub buf: ext_types::Buffer,
        pub name: types::String,
    }
    impl types::ApiFn for BufferGetMark {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "buffer_get_mark";
    }
    /// Signature of [`vim_command_output`](https://neovim.io/doc/user/api/#vim_command_output()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimCommandOutput {
        pub command: types::String,
    }
    impl types::ApiFn for VimCommandOutput {
        type Params = (types::String,);
        type Return = types::String;
        const NAME: &str = "vim_command_output";
    }
    /// Signature of [`buffer_get_number`](https://neovim.io/doc/user/api/#buffer_get_number()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetNumber {
        pub buffer: ext_types::Buffer,
    }
    impl types::ApiFn for BufferGetNumber {
        type Params = (ext_types::Buffer,);
        type Return = types::Int;
        const NAME: &str = "buffer_get_number";
    }
    /// Signature of [`buffer_clear_highlight`](https://neovim.io/doc/user/api/#buffer_clear_highlight()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferClearHighlight {
        pub buffer: ext_types::Buffer,
        pub ns_id: types::Int,
        pub line_start: types::Int,
        pub line_end: types::Int,
    }
    impl types::ApiFn for BufferClearHighlight {
        type Params = (ext_types::Buffer, types::Int, types::Int, types::Int);
        type Return = ();
        const NAME: &str = "buffer_clear_highlight";
    }
    /// Signature of [`buffer_add_highlight`](https://neovim.io/doc/user/api/#buffer_add_highlight()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferAddHighlight {
        pub buffer: ext_types::Buffer,
        pub ns_id: types::Int,
        pub hl_group: types::String,
        pub line: types::Int,
        pub col_start: types::Int,
        pub col_end: types::Int,
    }
    impl types::ApiFn for BufferAddHighlight {
        type Params = (
            ext_types::Buffer,
            types::Int,
            types::String,
            types::Int,
            types::Int,
            types::Int,
        );
        type Return = types::Int;
        const NAME: &str = "buffer_add_highlight";
    }
    /// Signature of [`vim_set_option`](https://neovim.io/doc/user/api/#vim_set_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetOption {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for VimSetOption {
        type Params = (types::String, types::Object);
        type Return = ();
        const NAME: &str = "vim_set_option";
    }
    /// Signature of [`vim_get_option`](https://neovim.io/doc/user/api/#vim_get_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetOption {
        pub name: types::String,
    }
    impl types::ApiFn for VimGetOption {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "vim_get_option";
    }
    /// Signature of [`buffer_get_option`](https://neovim.io/doc/user/api/#buffer_get_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferGetOption {
        pub buffer: ext_types::Buffer,
        pub name: types::String,
    }
    impl types::ApiFn for BufferGetOption {
        type Params = (ext_types::Buffer, types::String);
        type Return = types::Object;
        const NAME: &str = "buffer_get_option";
    }
    /// Signature of [`buffer_set_option`](https://neovim.io/doc/user/api/#buffer_set_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct BufferSetOption {
        pub buffer: ext_types::Buffer,
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for BufferSetOption {
        type Params = (ext_types::Buffer, types::String, types::Object);
        type Return = ();
        const NAME: &str = "buffer_set_option";
    }
    /// Signature of [`window_get_option`](https://neovim.io/doc/user/api/#window_get_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetOption {
        pub window: ext_types::Window,
        pub name: types::String,
    }
    impl types::ApiFn for WindowGetOption {
        type Params = (ext_types::Window, types::String);
        type Return = types::Object;
        const NAME: &str = "window_get_option";
    }
    /// Signature of [`window_set_option`](https://neovim.io/doc/user/api/#window_set_option()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowSetOption {
        pub window: ext_types::Window,
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::ApiFn for WindowSetOption {
        type Params = (ext_types::Window, types::String, types::Object);
        type Return = ();
        const NAME: &str = "window_set_option";
    }
    /// Signature of [`vim_subscribe`](https://neovim.io/doc/user/api/#vim_subscribe()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSubscribe {
        pub event: types::String,
    }
    impl types::ApiFn for VimSubscribe {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_subscribe";
    }
    /// Signature of [`vim_unsubscribe`](https://neovim.io/doc/user/api/#vim_unsubscribe()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimUnsubscribe {
        pub event: types::String,
    }
    impl types::ApiFn for VimUnsubscribe {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_unsubscribe";
    }
    /// Signature of [`vim_out_write`](https://neovim.io/doc/user/api/#vim_out_write()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimOutWrite {
        pub str: types::String,
    }
    impl types::ApiFn for VimOutWrite {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_out_write";
    }
    /// Signature of [`vim_err_write`](https://neovim.io/doc/user/api/#vim_err_write()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimErrWrite {
        pub str: types::String,
    }
    impl types::ApiFn for VimErrWrite {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_err_write";
    }
    /// Signature of [`vim_report_error`](https://neovim.io/doc/user/api/#vim_report_error()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimReportError {
        pub str: types::String,
    }
    impl types::ApiFn for VimReportError {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_report_error";
    }
    /// Signature of [`tabpage_get_windows`](https://neovim.io/doc/user/api/#tabpage_get_windows()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageGetWindows {
        pub tabpage: ext_types::Tabpage,
    }
    impl types::ApiFn for TabpageGetWindows {
        type Params = (ext_types::Tabpage,);
        type Return = types::ArrayOf<ext_types::Window>;
        const NAME: &str = "tabpage_get_windows";
    }
    /// Signature of [`tabpage_get_var`](https://neovim.io/doc/user/api/#tabpage_get_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageGetVar {
        pub tabpage: ext_types::Tabpage,
        pub name: types::String,
    }
    impl types::ApiFn for TabpageGetVar {
        type Params = (ext_types::Tabpage, types::String);
        type Return = types::Object;
        const NAME: &str = "tabpage_get_var";
    }
    /// Signature of [`tabpage_get_window`](https://neovim.io/doc/user/api/#tabpage_get_window()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageGetWindow {
        pub tabpage: ext_types::Tabpage,
    }
    impl types::ApiFn for TabpageGetWindow {
        type Params = (ext_types::Tabpage,);
        type Return = ext_types::Window;
        const NAME: &str = "tabpage_get_window";
    }
    /// Signature of [`tabpage_is_valid`](https://neovim.io/doc/user/api/#tabpage_is_valid()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct TabpageIsValid {
        pub tabpage: ext_types::Tabpage,
    }
    impl types::ApiFn for TabpageIsValid {
        type Params = (ext_types::Tabpage,);
        type Return = types::Bool;
        const NAME: &str = "tabpage_is_valid";
    }
    /// Signature of [`ui_detach`](https://neovim.io/doc/user/api/#ui_detach()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct UiDetach {}
    impl types::ApiFn for UiDetach {
        type Params = ();
        type Return = ();
        const NAME: &str = "ui_detach";
    }
    /// Signature of [`ui_try_resize`](https://neovim.io/doc/user/api/#ui_try_resize()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct UiTryResize {
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::ApiFn for UiTryResize {
        type Params = (types::Int, types::Int);
        type Return = types::Object;
        const NAME: &str = "ui_try_resize";
    }
    /// Signature of [`vim_feedkeys`](https://neovim.io/doc/user/api/#vim_feedkeys()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimFeedkeys {
        pub keys: types::String,
        pub mode: types::String,
        pub escape_ks: types::Bool,
    }
    impl types::ApiFn for VimFeedkeys {
        type Params = (types::String, types::String, types::Bool);
        type Return = ();
        const NAME: &str = "vim_feedkeys";
    }
    /// Signature of [`vim_input`](https://neovim.io/doc/user/api/#vim_input()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimInput {
        pub keys: types::String,
    }
    impl types::ApiFn for VimInput {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "vim_input";
    }
    /// Signature of [`vim_replace_termcodes`](https://neovim.io/doc/user/api/#vim_replace_termcodes()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimReplaceTermcodes {
        pub str: types::String,
        pub from_part: types::Bool,
        pub do_lt: types::Bool,
        pub special: types::Bool,
    }
    impl types::ApiFn for VimReplaceTermcodes {
        type Params = (types::String, types::Bool, types::Bool, types::Bool);
        type Return = types::String;
        const NAME: &str = "vim_replace_termcodes";
    }
    /// Signature of [`vim_strwidth`](https://neovim.io/doc/user/api/#vim_strwidth()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimStrwidth {
        pub text: types::String,
    }
    impl types::ApiFn for VimStrwidth {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "vim_strwidth";
    }
    /// Signature of [`vim_list_runtime_paths`](https://neovim.io/doc/user/api/#vim_list_runtime_paths()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimListRuntimePaths {}
    impl types::ApiFn for VimListRuntimePaths {
        type Params = ();
        type Return = types::ArrayOf<types::String>;
        const NAME: &str = "vim_list_runtime_paths";
    }
    /// Signature of [`vim_change_directory`](https://neovim.io/doc/user/api/#vim_change_directory()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimChangeDirectory {
        pub dir: types::String,
    }
    impl types::ApiFn for VimChangeDirectory {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_change_directory";
    }
    /// Signature of [`vim_get_current_line`](https://neovim.io/doc/user/api/#vim_get_current_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetCurrentLine {}
    impl types::ApiFn for VimGetCurrentLine {
        type Params = ();
        type Return = types::String;
        const NAME: &str = "vim_get_current_line";
    }
    /// Signature of [`vim_set_current_line`](https://neovim.io/doc/user/api/#vim_set_current_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetCurrentLine {
        pub line: types::String,
    }
    impl types::ApiFn for VimSetCurrentLine {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_set_current_line";
    }
    /// Signature of [`vim_del_current_line`](https://neovim.io/doc/user/api/#vim_del_current_line()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimDelCurrentLine {}
    impl types::ApiFn for VimDelCurrentLine {
        type Params = ();
        type Return = ();
        const NAME: &str = "vim_del_current_line";
    }
    /// Signature of [`vim_get_var`](https://neovim.io/doc/user/api/#vim_get_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetVar {
        pub name: types::String,
    }
    impl types::ApiFn for VimGetVar {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "vim_get_var";
    }
    /// Signature of [`vim_get_vvar`](https://neovim.io/doc/user/api/#vim_get_vvar()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetVvar {
        pub name: types::String,
    }
    impl types::ApiFn for VimGetVvar {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "vim_get_vvar";
    }
    /// Signature of [`vim_get_buffers`](https://neovim.io/doc/user/api/#vim_get_buffers()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetBuffers {}
    impl types::ApiFn for VimGetBuffers {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Buffer>;
        const NAME: &str = "vim_get_buffers";
    }
    /// Signature of [`vim_get_current_buffer`](https://neovim.io/doc/user/api/#vim_get_current_buffer()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetCurrentBuffer {}
    impl types::ApiFn for VimGetCurrentBuffer {
        type Params = ();
        type Return = ext_types::Buffer;
        const NAME: &str = "vim_get_current_buffer";
    }
    /// Signature of [`vim_set_current_buffer`](https://neovim.io/doc/user/api/#vim_set_current_buffer()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetCurrentBuffer {
        pub buf: ext_types::Buffer,
    }
    impl types::ApiFn for VimSetCurrentBuffer {
        type Params = (ext_types::Buffer,);
        type Return = ();
        const NAME: &str = "vim_set_current_buffer";
    }
    /// Signature of [`vim_get_windows`](https://neovim.io/doc/user/api/#vim_get_windows()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetWindows {}
    impl types::ApiFn for VimGetWindows {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Window>;
        const NAME: &str = "vim_get_windows";
    }
    /// Signature of [`vim_get_current_window`](https://neovim.io/doc/user/api/#vim_get_current_window()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetCurrentWindow {}
    impl types::ApiFn for VimGetCurrentWindow {
        type Params = ();
        type Return = ext_types::Window;
        const NAME: &str = "vim_get_current_window";
    }
    /// Signature of [`vim_set_current_window`](https://neovim.io/doc/user/api/#vim_set_current_window()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetCurrentWindow {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for VimSetCurrentWindow {
        type Params = (ext_types::Window,);
        type Return = ();
        const NAME: &str = "vim_set_current_window";
    }
    /// Signature of [`vim_get_tabpages`](https://neovim.io/doc/user/api/#vim_get_tabpages()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetTabpages {}
    impl types::ApiFn for VimGetTabpages {
        type Params = ();
        type Return = types::ArrayOf<ext_types::Tabpage>;
        const NAME: &str = "vim_get_tabpages";
    }
    /// Signature of [`vim_get_current_tabpage`](https://neovim.io/doc/user/api/#vim_get_current_tabpage()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetCurrentTabpage {}
    impl types::ApiFn for VimGetCurrentTabpage {
        type Params = ();
        type Return = ext_types::Tabpage;
        const NAME: &str = "vim_get_current_tabpage";
    }
    /// Signature of [`vim_set_current_tabpage`](https://neovim.io/doc/user/api/#vim_set_current_tabpage()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimSetCurrentTabpage {
        pub tabpage: ext_types::Tabpage,
    }
    impl types::ApiFn for VimSetCurrentTabpage {
        type Params = (ext_types::Tabpage,);
        type Return = ();
        const NAME: &str = "vim_set_current_tabpage";
    }
    /// Signature of [`vim_name_to_color`](https://neovim.io/doc/user/api/#vim_name_to_color()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimNameToColor {
        pub name: types::String,
    }
    impl types::ApiFn for VimNameToColor {
        type Params = (types::String,);
        type Return = types::Int;
        const NAME: &str = "vim_name_to_color";
    }
    /// Signature of [`vim_get_color_map`](https://neovim.io/doc/user/api/#vim_get_color_map()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetColorMap {}
    impl types::ApiFn for VimGetColorMap {
        type Params = ();
        type Return = types::Dict;
        const NAME: &str = "vim_get_color_map";
    }
    /// Signature of [`vim_get_api_info`](https://neovim.io/doc/user/api/#vim_get_api_info()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimGetApiInfo {}
    impl types::ApiFn for VimGetApiInfo {
        type Params = ();
        type Return = types::FixedArrayOf<types::Object, 2usize>;
        const NAME: &str = "vim_get_api_info";
    }
    /// Signature of [`vim_command`](https://neovim.io/doc/user/api/#vim_command()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimCommand {
        pub cmd: types::String,
    }
    impl types::ApiFn for VimCommand {
        type Params = (types::String,);
        type Return = ();
        const NAME: &str = "vim_command";
    }
    /// Signature of [`vim_eval`](https://neovim.io/doc/user/api/#vim_eval()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimEval {
        pub expr: types::String,
    }
    impl types::ApiFn for VimEval {
        type Params = (types::String,);
        type Return = types::Object;
        const NAME: &str = "vim_eval";
    }
    /// Signature of [`vim_call_function`](https://neovim.io/doc/user/api/#vim_call_function()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct VimCallFunction {
        pub r#fn: types::String,
        pub args: types::Array,
    }
    impl types::ApiFn for VimCallFunction {
        type Params = (types::String, types::Array);
        type Return = types::Object;
        const NAME: &str = "vim_call_function";
    }
    /// Signature of [`window_get_buffer`](https://neovim.io/doc/user/api/#window_get_buffer()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetBuffer {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetBuffer {
        type Params = (ext_types::Window,);
        type Return = ext_types::Buffer;
        const NAME: &str = "window_get_buffer";
    }
    /// Signature of [`window_get_cursor`](https://neovim.io/doc/user/api/#window_get_cursor()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetCursor {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetCursor {
        type Params = (ext_types::Window,);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "window_get_cursor";
    }
    /// Signature of [`window_set_cursor`](https://neovim.io/doc/user/api/#window_set_cursor()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowSetCursor {
        pub win: ext_types::Window,
        pub pos: types::FixedArrayOf<types::Int, 2usize>,
    }
    impl types::ApiFn for WindowSetCursor {
        type Params = (ext_types::Window, types::FixedArrayOf<types::Int, 2usize>);
        type Return = ();
        const NAME: &str = "window_set_cursor";
    }
    /// Signature of [`window_get_height`](https://neovim.io/doc/user/api/#window_get_height()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetHeight {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetHeight {
        type Params = (ext_types::Window,);
        type Return = types::Int;
        const NAME: &str = "window_get_height";
    }
    /// Signature of [`window_set_height`](https://neovim.io/doc/user/api/#window_set_height()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowSetHeight {
        pub win: ext_types::Window,
        pub height: types::Int,
    }
    impl types::ApiFn for WindowSetHeight {
        type Params = (ext_types::Window, types::Int);
        type Return = ();
        const NAME: &str = "window_set_height";
    }
    /// Signature of [`window_get_width`](https://neovim.io/doc/user/api/#window_get_width()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetWidth {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetWidth {
        type Params = (ext_types::Window,);
        type Return = types::Int;
        const NAME: &str = "window_get_width";
    }
    /// Signature of [`window_set_width`](https://neovim.io/doc/user/api/#window_set_width()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowSetWidth {
        pub win: ext_types::Window,
        pub width: types::Int,
    }
    impl types::ApiFn for WindowSetWidth {
        type Params = (ext_types::Window, types::Int);
        type Return = ();
        const NAME: &str = "window_set_width";
    }
    /// Signature of [`window_get_var`](https://neovim.io/doc/user/api/#window_get_var()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetVar {
        pub win: ext_types::Window,
        pub name: types::String,
    }
    impl types::ApiFn for WindowGetVar {
        type Params = (ext_types::Window, types::String);
        type Return = types::Object;
        const NAME: &str = "window_get_var";
    }
    /// Signature of [`window_get_position`](https://neovim.io/doc/user/api/#window_get_position()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetPosition {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetPosition {
        type Params = (ext_types::Window,);
        type Return = types::FixedArrayOf<types::Int, 2usize>;
        const NAME: &str = "window_get_position";
    }
    /// Signature of [`window_get_tabpage`](https://neovim.io/doc/user/api/#window_get_tabpage()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowGetTabpage {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowGetTabpage {
        type Params = (ext_types::Window,);
        type Return = ext_types::Tabpage;
        const NAME: &str = "window_get_tabpage";
    }
    /// Signature of [`window_is_valid`](https://neovim.io/doc/user/api/#window_is_valid()).
    ///
    /// Since Neovim API level 0; deprecated since 1.
    #[deprecated(note = "Deprecated since Neovim API level 1")]
    pub struct WindowIsValid {
        pub win: ext_types::Window,
    }
    impl types::ApiFn for WindowIsValid {
        type Params = (ext_types::Window,);
        type Return = types::Bool;
        const NAME: &str = "window_is_valid";
    }
}
/// UI event descriptors implementing [`crate::types::UiEvent`].
///
/// `GridResize`, for example, names the `grid_resize` event and describes
/// its `(grid, width, height)` arguments as integer type markers.
pub mod ui_events {
    use super::ext_types;
    use crate::types;
    /// Argument markers for the `mode_info_set` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct ModeInfoSet {
        pub enabled: types::Bool,
        pub cursor_styles: types::Array,
    }
    impl types::UiEvent for ModeInfoSet {
        type Params = (types::Bool, types::Array);
        const NAME: &str = "mode_info_set";
    }
    /// Argument markers for the `update_menu` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct UpdateMenu {}
    impl types::UiEvent for UpdateMenu {
        type Params = ();
        const NAME: &str = "update_menu";
    }
    /// Argument markers for the `busy_start` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct BusyStart {}
    impl types::UiEvent for BusyStart {
        type Params = ();
        const NAME: &str = "busy_start";
    }
    /// Argument markers for the `busy_stop` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct BusyStop {}
    impl types::UiEvent for BusyStop {
        type Params = ();
        const NAME: &str = "busy_stop";
    }
    /// Argument markers for the `mouse_on` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct MouseOn {}
    impl types::UiEvent for MouseOn {
        type Params = ();
        const NAME: &str = "mouse_on";
    }
    /// Argument markers for the `mouse_off` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct MouseOff {}
    impl types::UiEvent for MouseOff {
        type Params = ();
        const NAME: &str = "mouse_off";
    }
    /// Argument markers for the `mode_change` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct ModeChange {
        pub mode: types::String,
        pub mode_idx: types::Int,
    }
    impl types::UiEvent for ModeChange {
        type Params = (types::String, types::Int);
        const NAME: &str = "mode_change";
    }
    /// Argument markers for the `bell` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Bell {}
    impl types::UiEvent for Bell {
        type Params = ();
        const NAME: &str = "bell";
    }
    /// Argument markers for the `visual_bell` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct VisualBell {}
    impl types::UiEvent for VisualBell {
        type Params = ();
        const NAME: &str = "visual_bell";
    }
    /// Argument markers for the `flush` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Flush {}
    impl types::UiEvent for Flush {
        type Params = ();
        const NAME: &str = "flush";
    }
    /// Argument markers for the `connect` UI event.
    ///
    /// Since Neovim API level 14.
    pub struct Connect {
        pub server_addr: types::String,
    }
    impl types::UiEvent for Connect {
        type Params = (types::String,);
        const NAME: &str = "connect";
    }
    /// Argument markers for the `restart` UI event.
    ///
    /// Since Neovim API level 14.
    pub struct Restart {
        pub listen_addr: types::String,
    }
    impl types::UiEvent for Restart {
        type Params = (types::String,);
        const NAME: &str = "restart";
    }
    /// Argument markers for the `suspend` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Suspend {}
    impl types::UiEvent for Suspend {
        type Params = ();
        const NAME: &str = "suspend";
    }
    /// Argument markers for the `set_title` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct SetTitle {
        pub title: types::String,
    }
    impl types::UiEvent for SetTitle {
        type Params = (types::String,);
        const NAME: &str = "set_title";
    }
    /// Argument markers for the `set_icon` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct SetIcon {
        pub icon: types::String,
    }
    impl types::UiEvent for SetIcon {
        type Params = (types::String,);
        const NAME: &str = "set_icon";
    }
    /// Argument markers for the `screenshot` UI event.
    ///
    /// Since Neovim API level 7.
    pub struct Screenshot {
        pub path: types::String,
    }
    impl types::UiEvent for Screenshot {
        type Params = (types::String,);
        const NAME: &str = "screenshot";
    }
    /// Argument markers for the `option_set` UI event.
    ///
    /// Since Neovim API level 4.
    pub struct OptionSet {
        pub name: types::String,
        pub value: types::Object,
    }
    impl types::UiEvent for OptionSet {
        type Params = (types::String, types::Object);
        const NAME: &str = "option_set";
    }
    /// Argument markers for the `chdir` UI event.
    ///
    /// Since Neovim API level 12.
    pub struct Chdir {
        pub path: types::String,
    }
    impl types::UiEvent for Chdir {
        type Params = (types::String,);
        const NAME: &str = "chdir";
    }
    /// Argument markers for the `ui_send` UI event.
    ///
    /// Since Neovim API level 14.
    pub struct UiSend {
        pub content: types::String,
    }
    impl types::UiEvent for UiSend {
        type Params = (types::String,);
        const NAME: &str = "ui_send";
    }
    /// Argument markers for the `update_fg` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct UpdateFg {
        pub fg: types::Int,
    }
    impl types::UiEvent for UpdateFg {
        type Params = (types::Int,);
        const NAME: &str = "update_fg";
    }
    /// Argument markers for the `update_bg` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct UpdateBg {
        pub bg: types::Int,
    }
    impl types::UiEvent for UpdateBg {
        type Params = (types::Int,);
        const NAME: &str = "update_bg";
    }
    /// Argument markers for the `update_sp` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct UpdateSp {
        pub sp: types::Int,
    }
    impl types::UiEvent for UpdateSp {
        type Params = (types::Int,);
        const NAME: &str = "update_sp";
    }
    /// Argument markers for the `resize` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Resize {
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::UiEvent for Resize {
        type Params = (types::Int, types::Int);
        const NAME: &str = "resize";
    }
    /// Argument markers for the `clear` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Clear {}
    impl types::UiEvent for Clear {
        type Params = ();
        const NAME: &str = "clear";
    }
    /// Argument markers for the `eol_clear` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct EolClear {}
    impl types::UiEvent for EolClear {
        type Params = ();
        const NAME: &str = "eol_clear";
    }
    /// Argument markers for the `cursor_goto` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CursorGoto {
        pub row: types::Int,
        pub col: types::Int,
    }
    impl types::UiEvent for CursorGoto {
        type Params = (types::Int, types::Int);
        const NAME: &str = "cursor_goto";
    }
    /// Argument markers for the `highlight_set` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct HighlightSet {
        pub attrs: types::Dict,
    }
    impl types::UiEvent for HighlightSet {
        type Params = (types::Dict,);
        const NAME: &str = "highlight_set";
    }
    /// Argument markers for the `put` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Put {
        pub str: types::String,
    }
    impl types::UiEvent for Put {
        type Params = (types::String,);
        const NAME: &str = "put";
    }
    /// Argument markers for the `set_scroll_region` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct SetScrollRegion {
        pub top: types::Int,
        pub bot: types::Int,
        pub left: types::Int,
        pub right: types::Int,
    }
    impl types::UiEvent for SetScrollRegion {
        type Params = (types::Int, types::Int, types::Int, types::Int);
        const NAME: &str = "set_scroll_region";
    }
    /// Argument markers for the `scroll` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct Scroll {
        pub count: types::Int,
    }
    impl types::UiEvent for Scroll {
        type Params = (types::Int,);
        const NAME: &str = "scroll";
    }
    /// Argument markers for the `default_colors_set` UI event.
    ///
    /// Since Neovim API level 4.
    pub struct DefaultColorsSet {
        pub rgb_fg: types::Int,
        pub rgb_bg: types::Int,
        pub rgb_sp: types::Int,
        pub cterm_fg: types::Int,
        pub cterm_bg: types::Int,
    }
    impl types::UiEvent for DefaultColorsSet {
        type Params = (types::Int, types::Int, types::Int, types::Int, types::Int);
        const NAME: &str = "default_colors_set";
    }
    /// Argument markers for the `hl_attr_define` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct HlAttrDefine {
        pub id: types::Int,
        pub rgb_attrs: types::Dict,
        pub cterm_attrs: types::Dict,
        pub info: types::Array,
    }
    impl types::UiEvent for HlAttrDefine {
        type Params = (types::Int, types::Dict, types::Dict, types::Array);
        const NAME: &str = "hl_attr_define";
    }
    /// Argument markers for the `hl_group_set` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct HlGroupSet {
        pub name: types::String,
        pub id: types::Int,
    }
    impl types::UiEvent for HlGroupSet {
        type Params = (types::String, types::Int);
        const NAME: &str = "hl_group_set";
    }
    /// Argument markers for the `grid_resize` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct GridResize {
        pub grid: types::Int,
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::UiEvent for GridResize {
        type Params = (types::Int, types::Int, types::Int);
        const NAME: &str = "grid_resize";
    }
    /// Argument markers for the `grid_clear` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct GridClear {
        pub grid: types::Int,
    }
    impl types::UiEvent for GridClear {
        type Params = (types::Int,);
        const NAME: &str = "grid_clear";
    }
    /// Argument markers for the `grid_cursor_goto` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct GridCursorGoto {
        pub grid: types::Int,
        pub row: types::Int,
        pub col: types::Int,
    }
    impl types::UiEvent for GridCursorGoto {
        type Params = (types::Int, types::Int, types::Int);
        const NAME: &str = "grid_cursor_goto";
    }
    /// Argument markers for the `grid_line` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct GridLine {
        pub grid: types::Int,
        pub row: types::Int,
        pub col_start: types::Int,
        pub data: types::Array,
        pub wrap: types::Bool,
    }
    impl types::UiEvent for GridLine {
        type Params = (
            types::Int,
            types::Int,
            types::Int,
            types::Array,
            types::Bool,
        );
        const NAME: &str = "grid_line";
    }
    /// Argument markers for the `grid_scroll` UI event.
    ///
    /// Since Neovim API level 5.
    pub struct GridScroll {
        pub grid: types::Int,
        pub top: types::Int,
        pub bot: types::Int,
        pub left: types::Int,
        pub right: types::Int,
        pub rows: types::Int,
        pub cols: types::Int,
    }
    impl types::UiEvent for GridScroll {
        type Params = (
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "grid_scroll";
    }
    /// Argument markers for the `grid_destroy` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct GridDestroy {
        pub grid: types::Int,
    }
    impl types::UiEvent for GridDestroy {
        type Params = (types::Int,);
        const NAME: &str = "grid_destroy";
    }
    /// Argument markers for the `win_pos` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct WinPos {
        pub grid: types::Int,
        pub win: ext_types::Window,
        pub startrow: types::Int,
        pub startcol: types::Int,
        pub width: types::Int,
        pub height: types::Int,
    }
    impl types::UiEvent for WinPos {
        type Params = (
            types::Int,
            ext_types::Window,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "win_pos";
    }
    /// Argument markers for the `win_float_pos` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct WinFloatPos {
        pub grid: types::Int,
        pub win: ext_types::Window,
        pub anchor: types::String,
        pub anchor_grid: types::Int,
        pub anchor_row: types::Float,
        pub anchor_col: types::Float,
        pub mouse_enabled: types::Bool,
        pub zindex: types::Int,
        pub compindex: types::Int,
        pub screen_row: types::Int,
        pub screen_col: types::Int,
    }
    impl types::UiEvent for WinFloatPos {
        type Params = (
            types::Int,
            ext_types::Window,
            types::String,
            types::Int,
            types::Float,
            types::Float,
            types::Bool,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "win_float_pos";
    }
    /// Argument markers for the `win_external_pos` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct WinExternalPos {
        pub grid: types::Int,
        pub win: ext_types::Window,
    }
    impl types::UiEvent for WinExternalPos {
        type Params = (types::Int, ext_types::Window);
        const NAME: &str = "win_external_pos";
    }
    /// Argument markers for the `win_hide` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct WinHide {
        pub grid: types::Int,
    }
    impl types::UiEvent for WinHide {
        type Params = (types::Int,);
        const NAME: &str = "win_hide";
    }
    /// Argument markers for the `win_close` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct WinClose {
        pub grid: types::Int,
    }
    impl types::UiEvent for WinClose {
        type Params = (types::Int,);
        const NAME: &str = "win_close";
    }
    /// Argument markers for the `msg_set_pos` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgSetPos {
        pub grid: types::Int,
        pub row: types::Int,
        pub scrolled: types::Bool,
        pub sep_char: types::String,
        pub zindex: types::Int,
        pub compindex: types::Int,
    }
    impl types::UiEvent for MsgSetPos {
        type Params = (
            types::Int,
            types::Int,
            types::Bool,
            types::String,
            types::Int,
            types::Int,
        );
        const NAME: &str = "msg_set_pos";
    }
    /// Argument markers for the `win_viewport` UI event.
    ///
    /// Since Neovim API level 7.
    pub struct WinViewport {
        pub grid: types::Int,
        pub win: ext_types::Window,
        pub topline: types::Int,
        pub botline: types::Int,
        pub curline: types::Int,
        pub curcol: types::Int,
        pub line_count: types::Int,
        pub scroll_delta: types::Int,
    }
    impl types::UiEvent for WinViewport {
        type Params = (
            types::Int,
            ext_types::Window,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "win_viewport";
    }
    /// Argument markers for the `win_viewport_margins` UI event.
    ///
    /// Since Neovim API level 12.
    pub struct WinViewportMargins {
        pub grid: types::Int,
        pub win: ext_types::Window,
        pub top: types::Int,
        pub bottom: types::Int,
        pub left: types::Int,
        pub right: types::Int,
    }
    impl types::UiEvent for WinViewportMargins {
        type Params = (
            types::Int,
            ext_types::Window,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "win_viewport_margins";
    }
    /// Argument markers for the `win_extmark` UI event.
    ///
    /// Since Neovim API level 10.
    pub struct WinExtmark {
        pub grid: types::Int,
        pub win: ext_types::Window,
        pub ns_id: types::Int,
        pub mark_id: types::Int,
        pub row: types::Int,
        pub col: types::Int,
    }
    impl types::UiEvent for WinExtmark {
        type Params = (
            types::Int,
            ext_types::Window,
            types::Int,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "win_extmark";
    }
    /// Argument markers for the `popupmenu_show` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct PopupmenuShow {
        pub items: types::Array,
        pub selected: types::Int,
        pub row: types::Int,
        pub col: types::Int,
        pub grid: types::Int,
    }
    impl types::UiEvent for PopupmenuShow {
        type Params = (types::Array, types::Int, types::Int, types::Int, types::Int);
        const NAME: &str = "popupmenu_show";
    }
    /// Argument markers for the `popupmenu_hide` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct PopupmenuHide {}
    impl types::UiEvent for PopupmenuHide {
        type Params = ();
        const NAME: &str = "popupmenu_hide";
    }
    /// Argument markers for the `popupmenu_select` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct PopupmenuSelect {
        pub selected: types::Int,
    }
    impl types::UiEvent for PopupmenuSelect {
        type Params = (types::Int,);
        const NAME: &str = "popupmenu_select";
    }
    /// Argument markers for the `tabline_update` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct TablineUpdate {
        pub current: ext_types::Tabpage,
        pub tabs: types::Array,
        pub current_buffer: ext_types::Buffer,
        pub buffers: types::Array,
    }
    impl types::UiEvent for TablineUpdate {
        type Params = (
            ext_types::Tabpage,
            types::Array,
            ext_types::Buffer,
            types::Array,
        );
        const NAME: &str = "tabline_update";
    }
    /// Argument markers for the `cmdline_show` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineShow {
        pub content: types::Array,
        pub pos: types::Int,
        pub firstc: types::String,
        pub prompt: types::String,
        pub indent: types::Int,
        pub level: types::Int,
        pub hl_id: types::Int,
    }
    impl types::UiEvent for CmdlineShow {
        type Params = (
            types::Array,
            types::Int,
            types::String,
            types::String,
            types::Int,
            types::Int,
            types::Int,
        );
        const NAME: &str = "cmdline_show";
    }
    /// Argument markers for the `cmdline_pos` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlinePos {
        pub pos: types::Int,
        pub level: types::Int,
    }
    impl types::UiEvent for CmdlinePos {
        type Params = (types::Int, types::Int);
        const NAME: &str = "cmdline_pos";
    }
    /// Argument markers for the `cmdline_special_char` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineSpecialChar {
        pub c: types::String,
        pub shift: types::Bool,
        pub level: types::Int,
    }
    impl types::UiEvent for CmdlineSpecialChar {
        type Params = (types::String, types::Bool, types::Int);
        const NAME: &str = "cmdline_special_char";
    }
    /// Argument markers for the `cmdline_hide` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineHide {
        pub level: types::Int,
        pub abort: types::Bool,
    }
    impl types::UiEvent for CmdlineHide {
        type Params = (types::Int, types::Bool);
        const NAME: &str = "cmdline_hide";
    }
    /// Argument markers for the `cmdline_block_show` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineBlockShow {
        pub lines: types::Array,
    }
    impl types::UiEvent for CmdlineBlockShow {
        type Params = (types::Array,);
        const NAME: &str = "cmdline_block_show";
    }
    /// Argument markers for the `cmdline_block_append` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineBlockAppend {
        pub lines: types::Array,
    }
    impl types::UiEvent for CmdlineBlockAppend {
        type Params = (types::Array,);
        const NAME: &str = "cmdline_block_append";
    }
    /// Argument markers for the `cmdline_block_hide` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct CmdlineBlockHide {}
    impl types::UiEvent for CmdlineBlockHide {
        type Params = ();
        const NAME: &str = "cmdline_block_hide";
    }
    /// Argument markers for the `wildmenu_show` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct WildmenuShow {
        pub items: types::Array,
    }
    impl types::UiEvent for WildmenuShow {
        type Params = (types::Array,);
        const NAME: &str = "wildmenu_show";
    }
    /// Argument markers for the `wildmenu_select` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct WildmenuSelect {
        pub selected: types::Int,
    }
    impl types::UiEvent for WildmenuSelect {
        type Params = (types::Int,);
        const NAME: &str = "wildmenu_select";
    }
    /// Argument markers for the `wildmenu_hide` UI event.
    ///
    /// Since Neovim API level 3.
    pub struct WildmenuHide {}
    impl types::UiEvent for WildmenuHide {
        type Params = ();
        const NAME: &str = "wildmenu_hide";
    }
    /// Argument markers for the `msg_show` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgShow {
        pub kind: types::String,
        pub content: types::Array,
        pub replace_last: types::Bool,
        pub history: types::Bool,
        pub append: types::Bool,
        pub id: types::Object,
        pub trigger: types::String,
    }
    impl types::UiEvent for MsgShow {
        type Params = (
            types::String,
            types::Array,
            types::Bool,
            types::Bool,
            types::Bool,
            types::Object,
            types::String,
        );
        const NAME: &str = "msg_show";
    }
    /// Argument markers for the `msg_clear` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgClear {}
    impl types::UiEvent for MsgClear {
        type Params = ();
        const NAME: &str = "msg_clear";
    }
    /// Argument markers for the `msg_showcmd` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgShowcmd {
        pub content: types::Array,
    }
    impl types::UiEvent for MsgShowcmd {
        type Params = (types::Array,);
        const NAME: &str = "msg_showcmd";
    }
    /// Argument markers for the `msg_showmode` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgShowmode {
        pub content: types::Array,
    }
    impl types::UiEvent for MsgShowmode {
        type Params = (types::Array,);
        const NAME: &str = "msg_showmode";
    }
    /// Argument markers for the `msg_ruler` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgRuler {
        pub content: types::Array,
    }
    impl types::UiEvent for MsgRuler {
        type Params = (types::Array,);
        const NAME: &str = "msg_ruler";
    }
    /// Argument markers for the `msg_history_show` UI event.
    ///
    /// Since Neovim API level 6.
    pub struct MsgHistoryShow {
        pub entries: types::Array,
        pub prev_cmd: types::Bool,
    }
    impl types::UiEvent for MsgHistoryShow {
        type Params = (types::Array, types::Bool);
        const NAME: &str = "msg_history_show";
    }
    /// Argument markers for the `error_exit` UI event.
    ///
    /// Since Neovim API level 12.
    pub struct ErrorExit {
        pub status: types::Int,
    }
    impl types::UiEvent for ErrorExit {
        type Params = (types::Int,);
        const NAME: &str = "error_exit";
    }
}
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
    /// Neovim's [`rgb` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub rgb: bool,
    /// Neovim's [`ext_cmdline` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_cmdline: bool,
    /// Neovim's [`ext_popupmenu` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_popupmenu: bool,
    /// Neovim's [`ext_tabline` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_tabline: bool,
    /// Neovim's [`ext_wildmenu` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_wildmenu: bool,
    /// Neovim's [`ext_messages` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_messages: bool,
    /// Neovim's [`ext_linegrid` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_linegrid: bool,
    /// Neovim's [`ext_multigrid` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_multigrid: bool,
    /// Neovim's [`ext_hlstate` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_hlstate: bool,
    /// Neovim's [`ext_termcolors` UI option](https://neovim.io/doc/user/api-ui-events/#ui-option).
    pub ext_termcolors: bool,
}
impl Default for UiOptions {
    fn default() -> Self {
        Self {
            rgb: true,
            ext_cmdline: false,
            ext_popupmenu: false,
            ext_tabline: false,
            ext_wildmenu: false,
            ext_messages: false,
            ext_linegrid: false,
            ext_multigrid: false,
            ext_hlstate: false,
            ext_termcolors: false,
        }
    }
}
/// Error category IDs reported by Neovim's API metadata.
pub mod err_types {
    /// The `Exception` error category in Neovim's API metadata.
    pub struct Exception;
    impl Exception {
        /// The error category's numeric ID in the API metadata.
        pub const ID: i64 = 0i64;
    }
    /// The `Validation` error category in Neovim's API metadata.
    pub struct Validation;
    impl Validation {
        /// The error category's numeric ID in the API metadata.
        pub const ID: i64 = 1i64;
    }
}
/// Type markers and MessagePack extension tags for Neovim handles.
///
/// For example, [`Buffer::ID`](crate::ext_types::Buffer::ID) is the tag on an encoded buffer handle.
/// These marker types do not store runtime handle values.
pub mod ext_types {
    /// MessagePack extension marker for Neovim's `Buffer` handles.
    pub struct Buffer;
    impl Buffer {
        /// The MessagePack extension tag assigned by Neovim.
        pub const ID: i64 = 0i64;
    }
    /// MessagePack extension marker for Neovim's `Tabpage` handles.
    pub struct Tabpage;
    impl Tabpage {
        /// The MessagePack extension tag assigned by Neovim.
        pub const ID: i64 = 2i64;
    }
    /// MessagePack extension marker for Neovim's `Window` handles.
    pub struct Window;
    impl Window {
        /// The MessagePack extension tag assigned by Neovim.
        pub const ID: i64 = 1i64;
    }
}
