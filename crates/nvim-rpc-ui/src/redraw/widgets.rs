use super::decode::*;
use super::{Args, Model, Result};

use crate::ProtocolError;
use crate::model::*;

use nvim_rpc::mp::decode::Object;

use std::sync::Arc;

impl Model {
    pub(super) fn widget_call(&mut self, name: &str, mut a: Args<'_>) -> Result<()> {
        let limits = self.limits;
        match name {
            "cmdline_show" => {
                let content = content(next(&mut a)?, limits)?;
                let pos = index(next(&mut a)?)?;
                check_position(&content, pos)?;
                let firstc = text(next(&mut a)?, limits)?;
                let prompt = match next(&mut a)? {
                    Object::Bool(false) => Arc::from(""),
                    v => text(v, limits)?,
                };
                let indent = index(next(&mut a)?)?;
                let level = index(next(&mut a)?)? as u64;
                let hl_id = a.next().map(integer).transpose()?;
                bounded_insert(
                    Arc::make_mut(&mut Arc::make_mut(&mut self.state).cmdlines),
                    level,
                    Arc::new(Cmdline {
                        content,
                        pos,
                        firstc,
                        prompt,
                        indent,
                        level,
                        hl_id,
                        special: None,
                    }),
                    limits,
                )?;
                self.damage.widgets = true;
            }
            "cmdline_pos" => {
                let pos = index(next(&mut a)?)?;
                let level = index(next(&mut a)?)? as u64;
                let cmd = Arc::make_mut(&mut Arc::make_mut(&mut self.state).cmdlines)
                    .get_mut(&level)
                    .ok_or_else(|| bad(name, "unknown cmdline level"))?;
                check_position(&cmd.content, pos)?;
                Arc::make_mut(cmd).pos = pos;
                self.damage.widgets = true;
            }
            "cmdline_special_char" => {
                let text = text(next(&mut a)?, limits)?;
                let shift = boolean(next(&mut a)?)?;
                let level = index(next(&mut a)?)? as u64;
                let cmd = Arc::make_mut(&mut Arc::make_mut(&mut self.state).cmdlines)
                    .get_mut(&level)
                    .ok_or_else(|| bad(name, "unknown cmdline level"))?;
                Arc::make_mut(cmd).special = Some((text, shift));
                self.damage.widgets = true;
            }
            "cmdline_hide" => {
                let level = index(next(&mut a)?)? as u64;
                Arc::make_mut(&mut Arc::make_mut(&mut self.state).cmdlines).remove(&level);
                self.damage.widgets = true;
            }
            "cmdline_block_show" => {
                Arc::make_mut(&mut self.state).cmdline_block =
                    Arc::new(contents(next(&mut a)?, limits)?);
                self.damage.widgets = true;
            }
            "cmdline_block_append" => {
                let content = content(next(&mut a)?, limits)?;
                let lines = Arc::make_mut(&mut Arc::make_mut(&mut self.state).cmdline_block);
                if lines.len() >= limits.max_collection_len {
                    return Err(ProtocolError::Limit("cmdline block"));
                }
                lines.push(content);
                self.damage.widgets = true;
            }
            "cmdline_block_hide" => {
                Arc::make_mut(&mut self.state).cmdline_block = Arc::default();
                self.damage.widgets = true;
            }
            "popupmenu_show" => {
                let raw = array(next(&mut a)?)?;
                bounded_len(raw.len(), limits)?;
                let mut items = Vec::with_capacity(raw.len());
                for item in raw {
                    let mut item = array(item)?;
                    items.push(CompletionItem {
                        word: text(next(&mut item)?, limits)?,
                        kind: text(next(&mut item)?, limits)?,
                        menu: text(next(&mut item)?, limits)?,
                        info: text(next(&mut item)?, limits)?,
                    });
                }
                let selected = selected(next(&mut a)?, items.len())?;
                let row = integer(next(&mut a)?)?;
                let col = integer(next(&mut a)?)?;
                let grid = integer(next(&mut a)?)?;
                let anchor = if grid == -1 {
                    PopupAnchor::Cmdline {
                        byte_offset: usize::try_from(col)
                            .map_err(|_| bad(name, "negative cmdline offset"))?,
                    }
                } else {
                    PopupAnchor::Grid {
                        grid: GridId(grid),
                        row,
                        col,
                    }
                };
                Arc::make_mut(&mut self.state).popupmenu = Some(Arc::new(Popupmenu {
                    items,
                    selected,
                    anchor,
                }));
                self.damage.widgets = true;
            }
            "popupmenu_select" => {
                let value = next(&mut a)?;
                let popup = Arc::make_mut(&mut self.state)
                    .popupmenu
                    .as_mut()
                    .ok_or_else(|| bad(name, "popupmenu not visible"))?;
                let selected = selected(value, popup.items.len())?;
                Arc::make_mut(popup).selected = selected;
                self.damage.widgets = true;
            }
            "popupmenu_hide" => {
                Arc::make_mut(&mut self.state).popupmenu = None;
                self.damage.widgets = true;
            }
            "wildmenu_show" => {
                let items = array(next(&mut a)?)?;
                bounded_len(items.len(), limits)?;
                let items = items.map(|v| text(v, limits)).collect::<Result<_>>()?;
                Arc::make_mut(&mut self.state).wildmenu = Some(Arc::new(Wildmenu {
                    items,
                    selected: None,
                }));
                self.damage.widgets = true;
            }
            "wildmenu_select" => {
                let v = next(&mut a)?;
                let menu = Arc::make_mut(&mut self.state)
                    .wildmenu
                    .as_mut()
                    .ok_or_else(|| bad(name, "wildmenu not visible"))?;
                let selected = selected(v, menu.items.len())?;
                Arc::make_mut(menu).selected = selected;
                self.damage.widgets = true;
            }
            "wildmenu_hide" => {
                Arc::make_mut(&mut self.state).wildmenu = None;
                self.damage.widgets = true;
            }
            "tabline_update" => {
                let current = tabpage(next(&mut a)?)?;
                let raw = array(next(&mut a)?)?;
                bounded_len(raw.len(), limits)?;
                let mut tabs = Vec::with_capacity(raw.len());
                for item in raw {
                    let mut name = Arc::from("");
                    let mut tab = None;
                    for item in dictionary(item, limits)? {
                        let (key, value) = item?;
                        match key {
                            "name" => name = text(value, limits)?,
                            "tab" => tab = Some(tabpage(value)?),
                            _ => {}
                        }
                    }
                    tabs.push(Tab {
                        tab: tab.ok_or_else(|| bad(name.as_ref(), "missing tab handle"))?,
                        name,
                    });
                }
                let current_buffer = a.next().map(buffer).transpose()?;
                let mut buffers = Vec::new();
                if let Some(raw) = a.next() {
                    let raw = array(raw)?;
                    bounded_len(raw.len(), limits)?;
                    for item in raw {
                        let mut name = Arc::from("");
                        let mut buf = None;
                        for item in dictionary(item, limits)? {
                            let (key, value) = item?;
                            match key {
                                "name" => name = text(value, limits)?,
                                "buffer" => buf = Some(buffer(value)?),
                                _ => {}
                            }
                        }
                        buffers.push(BufferItem {
                            buffer: buf
                                .ok_or_else(|| bad("tabline_update", "missing buffer handle"))?,
                            name,
                        });
                    }
                }
                Arc::make_mut(&mut self.state).tabline = Some(Arc::new(Tabline {
                    current,
                    tabs,
                    current_buffer,
                    buffers,
                }));
                self.damage.widgets = true;
            }
            "msg_show" => {
                let kind = text(next(&mut a)?, limits)?;
                let content = content(next(&mut a)?, limits)?;
                let replace_last = boolean(next(&mut a)?)?;
                let history = a.next().map(boolean).transpose()?.unwrap_or(false);
                let append = a.next().map(boolean).transpose()?.unwrap_or(false);
                let id = a
                    .next()
                    .map(|v| value(v, limits, 0))
                    .transpose()?
                    .unwrap_or(Value::Nil);
                let trigger = a
                    .next()
                    .map(|v| text(v, limits))
                    .transpose()?
                    .unwrap_or_default();
                let messages = Arc::make_mut(&mut Arc::make_mut(&mut self.state).messages);
                let existing = if id != Value::Nil {
                    messages.visible.iter().position(|m| m.id == id)
                } else {
                    None
                };
                if let Some(message) = existing.and_then(|idx| messages.visible.get_mut(idx)) {
                    *message = Message {
                        kind,
                        content,
                        history,
                        id,
                        trigger,
                    };
                } else if let Some(message) = messages.visible.last_mut().filter(|_| append) {
                    let dst = &mut message.content;
                    if dst.len().saturating_add(content.len()) > limits.max_collection_len {
                        return Err(ProtocolError::Limit("message chunks"));
                    }
                    dst.extend(content);
                } else {
                    if replace_last {
                        messages.visible.pop();
                    }
                    if limits.max_messages == 0 {
                        return Err(ProtocolError::Limit("messages"));
                    }
                    if messages.visible.len() >= limits.max_messages {
                        messages.visible.remove(0);
                    }
                    messages.visible.push(Message {
                        kind,
                        content,
                        history,
                        id,
                        trigger,
                    });
                }
                self.damage.widgets = true;
            }
            "msg_clear" => {
                Arc::make_mut(&mut Arc::make_mut(&mut self.state).messages)
                    .visible
                    .clear();
                self.damage.widgets = true;
            }
            "msg_showmode" | "msg_showcmd" | "msg_ruler" => {
                let content = content(next(&mut a)?, limits)?;
                let messages = Arc::make_mut(&mut Arc::make_mut(&mut self.state).messages);
                match name {
                    "msg_showmode" => messages.showmode = content,
                    "msg_showcmd" => messages.showcmd = content,
                    _ => messages.ruler = content,
                }
                self.damage.widgets = true;
            }
            "msg_history_show" => {
                let entries = array(next(&mut a)?)?;
                bounded_len(entries.len(), limits)?;
                let mut history = Vec::with_capacity(entries.len());
                for entry in entries {
                    let mut entry = array(entry)?;
                    history.push(HistoryEntry {
                        kind: text(next(&mut entry)?, limits)?,
                        content: content(next(&mut entry)?, limits)?,
                        append: entry.next().map(boolean).transpose()?.unwrap_or(false),
                    });
                }
                let prev = a.next().map(boolean).transpose()?.unwrap_or(false);
                let messages = Arc::make_mut(&mut Arc::make_mut(&mut self.state).messages);
                messages.history = Some(history);
                messages.history_prev_cmd = prev;
                self.damage.widgets = true;
            }
            _ => return invalid(name, "unsupported widget event"),
        }
        Ok(())
    }
}
