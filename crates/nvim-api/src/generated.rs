//! Generated from Neovim 0.12.6 API metadata.
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
    pub use crate::sys::functions::{
        BufferAddHighlight, BufferClearHighlight, BufferDelLine, BufferDelVar, BufferGetLine,
        BufferGetLineSlice, BufferGetLines, BufferGetMark, BufferGetName, BufferGetNumber,
        BufferGetOption, BufferGetVar, BufferInsert, BufferIsValid, BufferLineCount, BufferSetLine,
        BufferSetLineSlice, BufferSetLines, BufferSetName, BufferSetOption, BufferSetVar,
        NvimBufAddHighlight, NvimBufAttach, NvimBufClearHighlight, NvimBufClearNamespace,
        NvimBufCreateUserCommand, NvimBufDelExtmark, NvimBufDelKeymap, NvimBufDelMark,
        NvimBufDelUserCommand, NvimBufDelVar, NvimBufDelete, NvimBufDetach, NvimBufGetChangedtick,
        NvimBufGetCommands, NvimBufGetExtmarkById, NvimBufGetExtmarks, NvimBufGetKeymap,
        NvimBufGetLines, NvimBufGetMark, NvimBufGetName, NvimBufGetNumber, NvimBufGetOffset,
        NvimBufGetOption, NvimBufGetText, NvimBufGetVar, NvimBufIsLoaded, NvimBufIsValid,
        NvimBufLineCount, NvimBufSetExtmark, NvimBufSetKeymap, NvimBufSetLines, NvimBufSetMark,
        NvimBufSetName, NvimBufSetOption, NvimBufSetText, NvimBufSetVar, NvimBufSetVirtualText,
        NvimCallAtomic, NvimCallDictFunction, NvimCallFunction, NvimChanSend, NvimClearAutocmds,
        NvimCmd, NvimCommand, NvimCommandOutput, NvimCreateAugroup, NvimCreateAutocmd,
        NvimCreateBuf, NvimCreateNamespace, NvimCreateUserCommand, NvimDelAugroupById,
        NvimDelAugroupByName, NvimDelAutocmd, NvimDelCurrentLine, NvimDelKeymap, NvimDelMark,
        NvimDelUserCommand, NvimDelVar, NvimEcho, NvimErrWrite, NvimErrWriteln, NvimEval,
        NvimEvalStatusline, NvimExec, NvimExec2, NvimExecAutocmds, NvimExecLua, NvimExecuteLua,
        NvimFeedkeys, NvimGetAllOptionsInfo, NvimGetApiInfo, NvimGetAutocmds, NvimGetChanInfo,
        NvimGetColorByName, NvimGetColorMap, NvimGetCommands, NvimGetContext, NvimGetCurrentBuf,
        NvimGetCurrentLine, NvimGetCurrentTabpage, NvimGetCurrentWin, NvimGetHl, NvimGetHlById,
        NvimGetHlByName, NvimGetHlIdByName, NvimGetHlNs, NvimGetKeymap, NvimGetMark, NvimGetMode,
        NvimGetNamespaces, NvimGetOption, NvimGetOptionInfo, NvimGetOptionInfo2,
        NvimGetOptionValue, NvimGetProc, NvimGetProcChildren, NvimGetRuntimeFile, NvimGetVar,
        NvimGetVvar, NvimInput, NvimInputMouse, NvimListBufs, NvimListChans, NvimListRuntimePaths,
        NvimListTabpages, NvimListUis, NvimListWins, NvimLoadContext, NvimNotify, NvimOpenTabpage,
        NvimOpenTerm, NvimOpenWin, NvimOutWrite, NvimParseCmd, NvimParseExpression, NvimPaste,
        NvimPut, NvimReplaceTermcodes, NvimSelectPopupmenuItem, NvimSetClientInfo,
        NvimSetCurrentBuf, NvimSetCurrentDir, NvimSetCurrentLine, NvimSetCurrentTabpage,
        NvimSetCurrentWin, NvimSetDecorationProvider, NvimSetHl, NvimSetHlNs, NvimSetHlNsFast,
        NvimSetKeymap, NvimSetOption, NvimSetOptionValue, NvimSetVar, NvimSetVvar, NvimStrwidth,
        NvimSubscribe, NvimTabpageDelVar, NvimTabpageGetNumber, NvimTabpageGetVar,
        NvimTabpageGetWin, NvimTabpageIsValid, NvimTabpageListWins, NvimTabpageSetVar,
        NvimTabpageSetWin, NvimUiAttach, NvimUiDetach, NvimUiPumSetBounds, NvimUiPumSetHeight,
        NvimUiSend, NvimUiSetFocus, NvimUiSetOption, NvimUiTermEvent, NvimUiTryResize,
        NvimUiTryResizeGrid, NvimUnsubscribe, NvimWinClose, NvimWinDelVar, NvimWinGetBuf,
        NvimWinGetConfig, NvimWinGetCursor, NvimWinGetHeight, NvimWinGetNumber, NvimWinGetOption,
        NvimWinGetPosition, NvimWinGetTabpage, NvimWinGetVar, NvimWinGetWidth, NvimWinHide,
        NvimWinIsValid, NvimWinSetBuf, NvimWinSetConfig, NvimWinSetCursor, NvimWinSetHeight,
        NvimWinSetHlNs, NvimWinSetOption, NvimWinSetVar, NvimWinSetWidth, NvimWinTextHeight,
        TabpageDelVar, TabpageGetVar, TabpageGetWindow, TabpageGetWindows, TabpageIsValid,
        TabpageSetVar, UiAttach, UiDetach, UiTryResize, VimCallFunction, VimChangeDirectory,
        VimCommand, VimCommandOutput, VimDelCurrentLine, VimDelVar, VimErrWrite, VimEval,
        VimFeedkeys, VimGetApiInfo, VimGetBuffers, VimGetColorMap, VimGetCurrentBuffer,
        VimGetCurrentLine, VimGetCurrentTabpage, VimGetCurrentWindow, VimGetOption, VimGetTabpages,
        VimGetVar, VimGetVvar, VimGetWindows, VimInput, VimListRuntimePaths, VimNameToColor,
        VimOutWrite, VimReplaceTermcodes, VimReportError, VimSetCurrentBuffer, VimSetCurrentLine,
        VimSetCurrentTabpage, VimSetCurrentWindow, VimSetOption, VimSetVar, VimStrwidth,
        VimSubscribe, VimUnsubscribe, WindowDelVar, WindowGetBuffer, WindowGetCursor,
        WindowGetHeight, WindowGetOption, WindowGetPosition, WindowGetTabpage, WindowGetVar,
        WindowGetWidth, WindowIsValid, WindowSetCursor, WindowSetHeight, WindowSetOption,
        WindowSetVar, WindowSetWidth,
    };
}
impl crate::handles::Buffer {
    /// Build a call to [`nvim_buf_line_count`](https://neovim.io/doc/user/api/#nvim_buf_line_count()).
    ///
    /// Since Neovim API level 1.
    pub fn line_count(self) -> crate::Call<sys::functions::NvimBufLineCount, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_attach`](https://neovim.io/doc/user/api/#nvim_buf_attach()).
    ///
    /// Since Neovim API level 4.
    pub fn attach<A0, A1>(
        self,
        send_buffer: A0,
        opts: A1,
    ) -> crate::Call<sys::functions::NvimBufAttach, (Self, A0, A1)>
    where
        A0: Like<sys::types::Bool>,
        A1: Like<sys::types::Dict>,
    {
        crate::Call::new((self, send_buffer, opts))
    }
    /// Build a call to [`nvim_buf_detach`](https://neovim.io/doc/user/api/#nvim_buf_detach()).
    ///
    /// Since Neovim API level 4.
    pub fn detach(self) -> crate::Call<sys::functions::NvimBufDetach, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_get_lines`](https://neovim.io/doc/user/api/#nvim_buf_get_lines()).
    ///
    /// Since Neovim API level 1.
    pub fn get_lines<A0, A1, A2>(
        self,
        start: A0,
        end: A1,
        strict_indexing: A2,
    ) -> crate::Call<sys::functions::NvimBufGetLines, (Self, A0, A1, A2)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Bool>,
    {
        crate::Call::new((self, start, end, strict_indexing))
    }
    /// Build a call to [`nvim_buf_set_lines`](https://neovim.io/doc/user/api/#nvim_buf_set_lines()).
    ///
    /// Since Neovim API level 1.
    pub fn set_lines<A0, A1, A2, A3>(
        self,
        start: A0,
        end: A1,
        strict_indexing: A2,
        replacement: A3,
    ) -> crate::Call<sys::functions::NvimBufSetLines, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Bool>,
        A3: Like<sys::types::ArrayOf<sys::types::String>>,
    {
        crate::Call::new((self, start, end, strict_indexing, replacement))
    }
    /// Build a call to [`nvim_buf_set_text`](https://neovim.io/doc/user/api/#nvim_buf_set_text()).
    ///
    /// Since Neovim API level 7.
    pub fn set_text<A0, A1, A2, A3, A4>(
        self,
        start_row: A0,
        start_col: A1,
        end_row: A2,
        end_col: A3,
        replacement: A4,
    ) -> crate::Call<sys::functions::NvimBufSetText, (Self, A0, A1, A2, A3, A4)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
        A3: Like<sys::types::Int>,
        A4: Like<sys::types::ArrayOf<sys::types::String>>,
    {
        crate::Call::new((self, start_row, start_col, end_row, end_col, replacement))
    }
    /// Build a call to [`nvim_buf_get_text`](https://neovim.io/doc/user/api/#nvim_buf_get_text()).
    ///
    /// Since Neovim API level 9.
    pub fn get_text<A0, A1, A2, A3, A4>(
        self,
        start_row: A0,
        start_col: A1,
        end_row: A2,
        end_col: A3,
        opts: A4,
    ) -> crate::Call<sys::functions::NvimBufGetText, (Self, A0, A1, A2, A3, A4)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
        A3: Like<sys::types::Int>,
        A4: Like<sys::types::Dict>,
    {
        crate::Call::new((self, start_row, start_col, end_row, end_col, opts))
    }
    /// Build a call to [`nvim_buf_get_offset`](https://neovim.io/doc/user/api/#nvim_buf_get_offset()).
    ///
    /// Since Neovim API level 5.
    pub fn get_offset<A0>(
        self,
        index: A0,
    ) -> crate::Call<sys::functions::NvimBufGetOffset, (Self, A0)>
    where
        A0: Like<sys::types::Int>,
    {
        crate::Call::new((self, index))
    }
    /// Build a call to [`nvim_buf_get_var`](https://neovim.io/doc/user/api/#nvim_buf_get_var()).
    ///
    /// Since Neovim API level 1.
    pub fn get_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimBufGetVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_get_changedtick`](https://neovim.io/doc/user/api/#nvim_buf_get_changedtick()).
    ///
    /// Since Neovim API level 2.
    pub fn get_changedtick(self) -> crate::Call<sys::functions::NvimBufGetChangedtick, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_get_keymap`](https://neovim.io/doc/user/api/#nvim_buf_get_keymap()).
    ///
    /// Since Neovim API level 3.
    pub fn get_keymap<A0>(
        self,
        mode: A0,
    ) -> crate::Call<sys::functions::NvimBufGetKeymap, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, mode))
    }
    /// Build a call to [`nvim_buf_set_keymap`](https://neovim.io/doc/user/api/#nvim_buf_set_keymap()).
    ///
    /// Since Neovim API level 6.
    pub fn set_keymap<A0, A1, A2, A3>(
        self,
        mode: A0,
        lhs: A1,
        rhs: A2,
        opts: A3,
    ) -> crate::Call<sys::functions::NvimBufSetKeymap, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::String>,
        A2: Like<sys::types::String>,
        A3: Like<sys::types::Dict>,
    {
        crate::Call::new((self, mode, lhs, rhs, opts))
    }
    /// Build a call to [`nvim_buf_del_keymap`](https://neovim.io/doc/user/api/#nvim_buf_del_keymap()).
    ///
    /// Since Neovim API level 6.
    pub fn del_keymap<A0, A1>(
        self,
        mode: A0,
        lhs: A1,
    ) -> crate::Call<sys::functions::NvimBufDelKeymap, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::String>,
    {
        crate::Call::new((self, mode, lhs))
    }
    /// Build a call to [`nvim_buf_set_var`](https://neovim.io/doc/user/api/#nvim_buf_set_var()).
    ///
    /// Since Neovim API level 1.
    pub fn set_var<A0, A1>(
        self,
        name: A0,
        value: A1,
    ) -> crate::Call<sys::functions::NvimBufSetVar, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
    {
        crate::Call::new((self, name, value))
    }
    /// Build a call to [`nvim_buf_del_var`](https://neovim.io/doc/user/api/#nvim_buf_del_var()).
    ///
    /// Since Neovim API level 1.
    pub fn del_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimBufDelVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_get_name`](https://neovim.io/doc/user/api/#nvim_buf_get_name()).
    ///
    /// Since Neovim API level 1.
    pub fn get_name(self) -> crate::Call<sys::functions::NvimBufGetName, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_set_name`](https://neovim.io/doc/user/api/#nvim_buf_set_name()).
    ///
    /// Since Neovim API level 1.
    pub fn set_name<A0>(self, name: A0) -> crate::Call<sys::functions::NvimBufSetName, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_is_loaded`](https://neovim.io/doc/user/api/#nvim_buf_is_loaded()).
    ///
    /// Since Neovim API level 5.
    pub fn is_loaded(self) -> crate::Call<sys::functions::NvimBufIsLoaded, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_delete`](https://neovim.io/doc/user/api/#nvim_buf_delete()).
    ///
    /// Since Neovim API level 7.
    pub fn delete<A0>(self, opts: A0) -> crate::Call<sys::functions::NvimBufDelete, (Self, A0)>
    where
        A0: Like<sys::types::Dict>,
    {
        crate::Call::new((self, opts))
    }
    /// Build a call to [`nvim_buf_is_valid`](https://neovim.io/doc/user/api/#nvim_buf_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub fn is_valid(self) -> crate::Call<sys::functions::NvimBufIsValid, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_del_mark`](https://neovim.io/doc/user/api/#nvim_buf_del_mark()).
    ///
    /// Since Neovim API level 8.
    pub fn del_mark<A0>(self, name: A0) -> crate::Call<sys::functions::NvimBufDelMark, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_set_mark`](https://neovim.io/doc/user/api/#nvim_buf_set_mark()).
    ///
    /// Since Neovim API level 8.
    pub fn set_mark<A0, A1, A2, A3>(
        self,
        name: A0,
        line: A1,
        col: A2,
        opts: A3,
    ) -> crate::Call<sys::functions::NvimBufSetMark, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
        A3: Like<sys::types::Dict>,
    {
        crate::Call::new((self, name, line, col, opts))
    }
    /// Build a call to [`nvim_buf_get_mark`](https://neovim.io/doc/user/api/#nvim_buf_get_mark()).
    ///
    /// Since Neovim API level 1.
    pub fn get_mark<A0>(self, name: A0) -> crate::Call<sys::functions::NvimBufGetMark, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_create_user_command`](https://neovim.io/doc/user/api/#nvim_buf_create_user_command()).
    ///
    /// Since Neovim API level 9.
    pub fn create_user_command<A0, A1, A2>(
        self,
        name: A0,
        cmd: A1,
        opts: A2,
    ) -> crate::Call<sys::functions::NvimBufCreateUserCommand, (Self, A0, A1, A2)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
        A2: Like<sys::types::Dict>,
    {
        crate::Call::new((self, name, cmd, opts))
    }
    /// Build a call to [`nvim_buf_del_user_command`](https://neovim.io/doc/user/api/#nvim_buf_del_user_command()).
    ///
    /// Since Neovim API level 9.
    pub fn del_user_command<A0>(
        self,
        name: A0,
    ) -> crate::Call<sys::functions::NvimBufDelUserCommand, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_get_commands`](https://neovim.io/doc/user/api/#nvim_buf_get_commands()).
    ///
    /// Since Neovim API level 4.
    pub fn get_commands<A0>(
        self,
        opts: A0,
    ) -> crate::Call<sys::functions::NvimBufGetCommands, (Self, A0)>
    where
        A0: Like<sys::types::Dict>,
    {
        crate::Call::new((self, opts))
    }
    /// Build a call to [`nvim_buf_get_number`](https://neovim.io/doc/user/api/#nvim_buf_get_number()).
    ///
    /// Since Neovim API level 1; deprecated since 2.
    #[deprecated(note = "Deprecated since Neovim API level 2")]
    pub fn get_number(self) -> crate::Call<sys::functions::NvimBufGetNumber, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_buf_clear_highlight`](https://neovim.io/doc/user/api/#nvim_buf_clear_highlight()).
    ///
    /// Since Neovim API level 1; deprecated since 7.
    #[deprecated(note = "Deprecated since Neovim API level 7")]
    pub fn clear_highlight<A0, A1, A2>(
        self,
        ns_id: A0,
        line_start: A1,
        line_end: A2,
    ) -> crate::Call<sys::functions::NvimBufClearHighlight, (Self, A0, A1, A2)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
    {
        crate::Call::new((self, ns_id, line_start, line_end))
    }
    /// Build a call to [`nvim_buf_add_highlight`](https://neovim.io/doc/user/api/#nvim_buf_add_highlight()).
    ///
    /// Since Neovim API level 1; deprecated since 13.
    #[deprecated(note = "Deprecated since Neovim API level 13")]
    pub fn add_highlight<A0, A1, A2, A3, A4>(
        self,
        ns_id: A0,
        hl_group: A1,
        line: A2,
        col_start: A3,
        col_end: A4,
    ) -> crate::Call<sys::functions::NvimBufAddHighlight, (Self, A0, A1, A2, A3, A4)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::String>,
        A2: Like<sys::types::Int>,
        A3: Like<sys::types::Int>,
        A4: Like<sys::types::Int>,
    {
        crate::Call::new((self, ns_id, hl_group, line, col_start, col_end))
    }
    /// Build a call to [`nvim_buf_set_virtual_text`](https://neovim.io/doc/user/api/#nvim_buf_set_virtual_text()).
    ///
    /// Since Neovim API level 5; deprecated since 8.
    #[deprecated(note = "Deprecated since Neovim API level 8")]
    pub fn set_virtual_text<A0, A1, A2, A3>(
        self,
        src_id: A0,
        line: A1,
        chunks: A2,
        opts: A3,
    ) -> crate::Call<sys::functions::NvimBufSetVirtualText, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Array>,
        A3: Like<sys::types::Dict>,
    {
        crate::Call::new((self, src_id, line, chunks, opts))
    }
    /// Build a call to [`nvim_buf_get_option`](https://neovim.io/doc/user/api/#nvim_buf_get_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub fn get_option<A0>(
        self,
        name: A0,
    ) -> crate::Call<sys::functions::NvimBufGetOption, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_buf_set_option`](https://neovim.io/doc/user/api/#nvim_buf_set_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub fn set_option<A0, A1>(
        self,
        name: A0,
        value: A1,
    ) -> crate::Call<sys::functions::NvimBufSetOption, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
    {
        crate::Call::new((self, name, value))
    }
    /// Build a call to [`nvim_buf_get_extmark_by_id`](https://neovim.io/doc/user/api/#nvim_buf_get_extmark_by_id()).
    ///
    /// Since Neovim API level 7.
    pub fn get_extmark_by_id<A0, A1, A2>(
        self,
        ns_id: A0,
        id: A1,
        opts: A2,
    ) -> crate::Call<sys::functions::NvimBufGetExtmarkById, (Self, A0, A1, A2)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Dict>,
    {
        crate::Call::new((self, ns_id, id, opts))
    }
    /// Build a call to [`nvim_buf_get_extmarks`](https://neovim.io/doc/user/api/#nvim_buf_get_extmarks()).
    ///
    /// Since Neovim API level 7.
    pub fn get_extmarks<A0, A1, A2, A3>(
        self,
        ns_id: A0,
        start: A1,
        end: A2,
        opts: A3,
    ) -> crate::Call<sys::functions::NvimBufGetExtmarks, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Object>,
        A2: Like<sys::types::Object>,
        A3: Like<sys::types::Dict>,
    {
        crate::Call::new((self, ns_id, start, end, opts))
    }
    /// Build a call to [`nvim_buf_set_extmark`](https://neovim.io/doc/user/api/#nvim_buf_set_extmark()).
    ///
    /// Since Neovim API level 7.
    pub fn set_extmark<A0, A1, A2, A3>(
        self,
        ns_id: A0,
        line: A1,
        col: A2,
        opts: A3,
    ) -> crate::Call<sys::functions::NvimBufSetExtmark, (Self, A0, A1, A2, A3)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
        A3: Like<sys::types::Dict>,
    {
        crate::Call::new((self, ns_id, line, col, opts))
    }
    /// Build a call to [`nvim_buf_del_extmark`](https://neovim.io/doc/user/api/#nvim_buf_del_extmark()).
    ///
    /// Since Neovim API level 7.
    pub fn del_extmark<A0, A1>(
        self,
        ns_id: A0,
        id: A1,
    ) -> crate::Call<sys::functions::NvimBufDelExtmark, (Self, A0, A1)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
    {
        crate::Call::new((self, ns_id, id))
    }
    /// Build a call to [`nvim_buf_clear_namespace`](https://neovim.io/doc/user/api/#nvim_buf_clear_namespace()).
    ///
    /// Since Neovim API level 5.
    pub fn clear_namespace<A0, A1, A2>(
        self,
        ns_id: A0,
        line_start: A1,
        line_end: A2,
    ) -> crate::Call<sys::functions::NvimBufClearNamespace, (Self, A0, A1, A2)>
    where
        A0: Like<sys::types::Int>,
        A1: Like<sys::types::Int>,
        A2: Like<sys::types::Int>,
    {
        crate::Call::new((self, ns_id, line_start, line_end))
    }
}
impl crate::handles::Tabpage {
    /// Build a call to [`nvim_tabpage_list_wins`](https://neovim.io/doc/user/api/#nvim_tabpage_list_wins()).
    ///
    /// Since Neovim API level 1.
    pub fn list_wins(self) -> crate::Call<sys::functions::NvimTabpageListWins, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_tabpage_get_var`](https://neovim.io/doc/user/api/#nvim_tabpage_get_var()).
    ///
    /// Since Neovim API level 1.
    pub fn get_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimTabpageGetVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_tabpage_set_var`](https://neovim.io/doc/user/api/#nvim_tabpage_set_var()).
    ///
    /// Since Neovim API level 1.
    pub fn set_var<A0, A1>(
        self,
        name: A0,
        value: A1,
    ) -> crate::Call<sys::functions::NvimTabpageSetVar, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
    {
        crate::Call::new((self, name, value))
    }
    /// Build a call to [`nvim_tabpage_del_var`](https://neovim.io/doc/user/api/#nvim_tabpage_del_var()).
    ///
    /// Since Neovim API level 1.
    pub fn del_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimTabpageDelVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_tabpage_get_win`](https://neovim.io/doc/user/api/#nvim_tabpage_get_win()).
    ///
    /// Since Neovim API level 1.
    pub fn get_win(self) -> crate::Call<sys::functions::NvimTabpageGetWin, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_tabpage_set_win`](https://neovim.io/doc/user/api/#nvim_tabpage_set_win()).
    ///
    /// Since Neovim API level 12.
    pub fn set_win<A0>(self, win: A0) -> crate::Call<sys::functions::NvimTabpageSetWin, (Self, A0)>
    where
        A0: Like<sys::ext_types::Window>,
    {
        crate::Call::new((self, win))
    }
    /// Build a call to [`nvim_tabpage_get_number`](https://neovim.io/doc/user/api/#nvim_tabpage_get_number()).
    ///
    /// Since Neovim API level 1.
    pub fn get_number(self) -> crate::Call<sys::functions::NvimTabpageGetNumber, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_tabpage_is_valid`](https://neovim.io/doc/user/api/#nvim_tabpage_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub fn is_valid(self) -> crate::Call<sys::functions::NvimTabpageIsValid, (Self,)> {
        crate::Call::new((self,))
    }
}
impl crate::handles::Window {
    /// Build a call to [`nvim_win_get_option`](https://neovim.io/doc/user/api/#nvim_win_get_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub fn get_option<A0>(
        self,
        name: A0,
    ) -> crate::Call<sys::functions::NvimWinGetOption, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_win_set_option`](https://neovim.io/doc/user/api/#nvim_win_set_option()).
    ///
    /// Since Neovim API level 1; deprecated since 11.
    #[deprecated(note = "Deprecated since Neovim API level 11")]
    pub fn set_option<A0, A1>(
        self,
        name: A0,
        value: A1,
    ) -> crate::Call<sys::functions::NvimWinSetOption, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
    {
        crate::Call::new((self, name, value))
    }
    /// Build a call to [`nvim_win_set_config`](https://neovim.io/doc/user/api/#nvim_win_set_config()).
    ///
    /// Since Neovim API level 6.
    pub fn set_config<A0>(
        self,
        config: A0,
    ) -> crate::Call<sys::functions::NvimWinSetConfig, (Self, A0)>
    where
        A0: Like<sys::types::Dict>,
    {
        crate::Call::new((self, config))
    }
    /// Build a call to [`nvim_win_get_config`](https://neovim.io/doc/user/api/#nvim_win_get_config()).
    ///
    /// Since Neovim API level 6.
    pub fn get_config(self) -> crate::Call<sys::functions::NvimWinGetConfig, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_get_buf`](https://neovim.io/doc/user/api/#nvim_win_get_buf()).
    ///
    /// Since Neovim API level 1.
    pub fn get_buf(self) -> crate::Call<sys::functions::NvimWinGetBuf, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_set_buf`](https://neovim.io/doc/user/api/#nvim_win_set_buf()).
    ///
    /// Since Neovim API level 5.
    pub fn set_buf<A0>(self, buf: A0) -> crate::Call<sys::functions::NvimWinSetBuf, (Self, A0)>
    where
        A0: Like<sys::ext_types::Buffer>,
    {
        crate::Call::new((self, buf))
    }
    /// Build a call to [`nvim_win_get_cursor`](https://neovim.io/doc/user/api/#nvim_win_get_cursor()).
    ///
    /// Since Neovim API level 1.
    pub fn get_cursor(self) -> crate::Call<sys::functions::NvimWinGetCursor, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_set_cursor`](https://neovim.io/doc/user/api/#nvim_win_set_cursor()).
    ///
    /// Since Neovim API level 1.
    pub fn set_cursor<A0>(
        self,
        pos: A0,
    ) -> crate::Call<sys::functions::NvimWinSetCursor, (Self, A0)>
    where
        A0: Like<sys::types::FixedArrayOf<sys::types::Int, 2usize>>,
    {
        crate::Call::new((self, pos))
    }
    /// Build a call to [`nvim_win_get_height`](https://neovim.io/doc/user/api/#nvim_win_get_height()).
    ///
    /// Since Neovim API level 1.
    pub fn get_height(self) -> crate::Call<sys::functions::NvimWinGetHeight, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_set_height`](https://neovim.io/doc/user/api/#nvim_win_set_height()).
    ///
    /// Since Neovim API level 1.
    pub fn set_height<A0>(
        self,
        height: A0,
    ) -> crate::Call<sys::functions::NvimWinSetHeight, (Self, A0)>
    where
        A0: Like<sys::types::Int>,
    {
        crate::Call::new((self, height))
    }
    /// Build a call to [`nvim_win_get_width`](https://neovim.io/doc/user/api/#nvim_win_get_width()).
    ///
    /// Since Neovim API level 1.
    pub fn get_width(self) -> crate::Call<sys::functions::NvimWinGetWidth, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_set_width`](https://neovim.io/doc/user/api/#nvim_win_set_width()).
    ///
    /// Since Neovim API level 1.
    pub fn set_width<A0>(
        self,
        width: A0,
    ) -> crate::Call<sys::functions::NvimWinSetWidth, (Self, A0)>
    where
        A0: Like<sys::types::Int>,
    {
        crate::Call::new((self, width))
    }
    /// Build a call to [`nvim_win_get_var`](https://neovim.io/doc/user/api/#nvim_win_get_var()).
    ///
    /// Since Neovim API level 1.
    pub fn get_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimWinGetVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_win_set_var`](https://neovim.io/doc/user/api/#nvim_win_set_var()).
    ///
    /// Since Neovim API level 1.
    pub fn set_var<A0, A1>(
        self,
        name: A0,
        value: A1,
    ) -> crate::Call<sys::functions::NvimWinSetVar, (Self, A0, A1)>
    where
        A0: Like<sys::types::String>,
        A1: Like<sys::types::Object>,
    {
        crate::Call::new((self, name, value))
    }
    /// Build a call to [`nvim_win_del_var`](https://neovim.io/doc/user/api/#nvim_win_del_var()).
    ///
    /// Since Neovim API level 1.
    pub fn del_var<A0>(self, name: A0) -> crate::Call<sys::functions::NvimWinDelVar, (Self, A0)>
    where
        A0: Like<sys::types::String>,
    {
        crate::Call::new((self, name))
    }
    /// Build a call to [`nvim_win_get_position`](https://neovim.io/doc/user/api/#nvim_win_get_position()).
    ///
    /// Since Neovim API level 1.
    pub fn get_position(self) -> crate::Call<sys::functions::NvimWinGetPosition, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_get_tabpage`](https://neovim.io/doc/user/api/#nvim_win_get_tabpage()).
    ///
    /// Since Neovim API level 1.
    pub fn get_tabpage(self) -> crate::Call<sys::functions::NvimWinGetTabpage, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_get_number`](https://neovim.io/doc/user/api/#nvim_win_get_number()).
    ///
    /// Since Neovim API level 1.
    pub fn get_number(self) -> crate::Call<sys::functions::NvimWinGetNumber, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_is_valid`](https://neovim.io/doc/user/api/#nvim_win_is_valid()).
    ///
    /// Since Neovim API level 1.
    pub fn is_valid(self) -> crate::Call<sys::functions::NvimWinIsValid, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_hide`](https://neovim.io/doc/user/api/#nvim_win_hide()).
    ///
    /// Since Neovim API level 7.
    pub fn hide(self) -> crate::Call<sys::functions::NvimWinHide, (Self,)> {
        crate::Call::new((self,))
    }
    /// Build a call to [`nvim_win_close`](https://neovim.io/doc/user/api/#nvim_win_close()).
    ///
    /// Since Neovim API level 6.
    pub fn close<A0>(self, force: A0) -> crate::Call<sys::functions::NvimWinClose, (Self, A0)>
    where
        A0: Like<sys::types::Bool>,
    {
        crate::Call::new((self, force))
    }
    /// Build a call to [`nvim_win_set_hl_ns`](https://neovim.io/doc/user/api/#nvim_win_set_hl_ns()).
    ///
    /// Since Neovim API level 10.
    pub fn set_hl_ns<A0>(self, ns_id: A0) -> crate::Call<sys::functions::NvimWinSetHlNs, (Self, A0)>
    where
        A0: Like<sys::types::Int>,
    {
        crate::Call::new((self, ns_id))
    }
    /// Build a call to [`nvim_win_text_height`](https://neovim.io/doc/user/api/#nvim_win_text_height()).
    ///
    /// Since Neovim API level 12.
    pub fn text_height<A0>(
        self,
        opts: A0,
    ) -> crate::Call<sys::functions::NvimWinTextHeight, (Self, A0)>
    where
        A0: Like<sys::types::Dict>,
    {
        crate::Call::new((self, opts))
    }
}
fn encode_ui_options(
    options: &sys::UiOptions,
) -> mp::encode::primitive::Map<[(&'static str, bool); 10usize]> {
    mp::encode::primitive::Map([
        ("rgb", options.rgb),
        ("ext_cmdline", options.ext_cmdline),
        ("ext_popupmenu", options.ext_popupmenu),
        ("ext_tabline", options.ext_tabline),
        ("ext_wildmenu", options.ext_wildmenu),
        ("ext_messages", options.ext_messages),
        ("ext_linegrid", options.ext_linegrid),
        ("ext_multigrid", options.ext_multigrid),
        ("ext_hlstate", options.ext_hlstate),
        ("ext_termcolors", options.ext_termcolors),
    ])
}
impl Like<sys::types::Dict> for sys::UiOptions {
    type EncodeAs = mp::encode::primitive::Map<[(&'static str, bool); 10usize]>;
    fn encode(self) -> Self::EncodeAs {
        encode_ui_options(&self)
    }
}
impl Like<sys::types::Dict> for &sys::UiOptions {
    type EncodeAs = mp::encode::primitive::Map<[(&'static str, bool); 10usize]>;
    fn encode(self) -> Self::EncodeAs {
        encode_ui_options(self)
    }
}
impl Like<sys::types::Object> for sys::UiOptions {
    type EncodeAs = mp::encode::primitive::Map<[(&'static str, bool); 10usize]>;
    fn encode(self) -> Self::EncodeAs {
        encode_ui_options(&self)
    }
}
impl Like<sys::types::Object> for &sys::UiOptions {
    type EncodeAs = mp::encode::primitive::Map<[(&'static str, bool); 10usize]>;
    fn encode(self) -> Self::EncodeAs {
        encode_ui_options(self)
    }
}
