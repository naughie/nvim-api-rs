use super::{Args, Obj, Result};

use crate::ProtocolError;
use crate::model::*;

use nvim_rpc::api_types::Decode;
use nvim_rpc::mp::decode::Object;
use nvim_rpc::sys::{ext_types, types};
use nvim_rpc::{Buffer, Tabpage, Window};

use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) fn bad(event: &str, reason: &'static str) -> ProtocolError {
    ProtocolError::Invalid {
        event: event.into(),
        reason,
    }
}

pub(super) fn invalid<T>(event: &str, reason: &'static str) -> Result<T> {
    Err(bad(event, reason))
}

pub(super) fn next<'a>(a: &mut Args<'a>) -> Result<Obj<'a>> {
    a.next().ok_or_else(|| bad("redraw", "missing argument"))
}

pub(super) fn integer(v: Obj<'_>) -> Result<i64> {
    <i64 as Decode<types::Int, _>>::decode(v).map_err(Into::into)
}

pub(super) fn index(v: Obj<'_>) -> Result<usize> {
    usize::try_from(integer(v)?).map_err(|_| bad("redraw", "negative or overflowing index"))
}

pub(super) fn size(v: Obj<'_>) -> Result<usize> {
    let n = index(v)?;
    if n == 0 {
        invalid("redraw", "zero size")
    } else {
        Ok(n)
    }
}

pub(super) fn boolean(v: Obj<'_>) -> Result<bool> {
    <bool as Decode<types::Bool, _>>::decode(v).map_err(Into::into)
}

pub(super) fn float(v: Obj<'_>) -> Result<f64> {
    let n = match v {
        Object::F32(n) => f64::from(n),
        Object::F64(n) => n,
        v => integer(v)? as f64,
    };
    if n.is_finite() {
        Ok(n)
    } else {
        invalid("redraw", "non-finite coordinate")
    }
}

pub(super) fn grid_id(v: Obj<'_>) -> Result<GridId> {
    let id = integer(v)?;
    if id <= 0 {
        invalid("redraw", "invalid grid id")
    } else {
        Ok(GridId(id))
    }
}

pub(super) fn hl_id(v: Obj<'_>) -> Result<HighlightId> {
    let id = integer(v)?;
    if id < 0 {
        invalid("redraw", "negative highlight id")
    } else {
        Ok(HighlightId(id))
    }
}

pub(super) fn window(v: Obj<'_>) -> Result<Window> {
    <Window as Decode<ext_types::Window, _>>::decode(v).map_err(Into::into)
}

pub(super) fn buffer(v: Obj<'_>) -> Result<Buffer> {
    <Buffer as Decode<ext_types::Buffer, _>>::decode(v).map_err(Into::into)
}

pub(super) fn tabpage(v: Obj<'_>) -> Result<Tabpage> {
    <Tabpage as Decode<ext_types::Tabpage, _>>::decode(v).map_err(Into::into)
}

pub(super) fn string(v: Obj<'_>, limits: Limits) -> Result<&str> {
    let text = <&str as Decode<types::String, _>>::decode(v)?;
    if text.len() > limits.max_string_bytes {
        return Err(ProtocolError::Limit("string bytes"));
    }
    Ok(text)
}

pub(super) fn text(v: Obj<'_>, limits: Limits) -> Result<Arc<str>> {
    Ok(super::text::shared(string(v, limits)?))
}

pub(super) fn array(v: Obj<'_>) -> Result<Args<'_>> {
    <Args<'_> as Decode<types::Array, _>>::decode(v).map_err(Into::into)
}

pub(super) fn dictionary(
    v: Obj<'_>,
    limits: Limits,
) -> Result<impl Iterator<Item = Result<(&str, Obj<'_>)>>> {
    let map = <nvim_rpc::mp::decode::Map<_> as Decode<types::Dict, _>>::decode(v)?;
    bounded_len(map.len(), limits)?;
    Ok(map.map(move |item| {
        let key = <&str as Decode<types::String, _>>::decode(item.key)?;
        if key.len() > limits.max_string_bytes {
            return Err(ProtocolError::Limit("dictionary key"));
        }
        Ok((key, item.value))
    }))
}

pub(super) fn bounded_len(len: usize, limits: Limits) -> Result<()> {
    if len > limits.max_collection_len {
        Err(ProtocolError::Limit("collection length"))
    } else {
        Ok(())
    }
}

pub(super) fn bounded_insert<K: Ord, V>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    limits: Limits,
) -> Result<()> {
    if !map.contains_key(&key) && map.len() >= limits.max_collection_len {
        return Err(ProtocolError::Limit("dictionary length"));
    }
    map.insert(key, value);
    Ok(())
}

pub(super) fn values(v: Obj<'_>, limits: Limits) -> Result<Vec<Value>> {
    let a = array(v)?;
    bounded_len(a.len(), limits)?;
    a.map(|v| value(v, limits, 0)).collect()
}

pub(super) fn value(v: Obj<'_>, limits: Limits, depth: usize) -> Result<Value> {
    if depth > limits.max_depth {
        return Err(ProtocolError::Limit("value depth"));
    }
    Ok(match v {
        Object::Nil => Value::Nil,
        Object::Bool(v) => Value::Bool(v),
        Object::U64(v) if v > i64::MAX as u64 => Value::Uint(v),
        Object::F32(v) => Value::Float(f64::from(v)),
        Object::F64(v) => Value::Float(v),
        Object::Str(v) => {
            if v.as_slice().len() > limits.max_string_bytes {
                return Err(ProtocolError::Limit("string bytes"));
            }
            Value::String(Arc::from(v.as_slice()))
        }
        Object::Bin(v) => {
            if v.as_slice().len() > limits.max_string_bytes {
                return Err(ProtocolError::Limit("binary bytes"));
            }
            Value::Binary(Arc::from(v.as_slice()))
        }
        Object::Ext(ty, v) => {
            if v.as_slice().len() > limits.max_string_bytes {
                return Err(ProtocolError::Limit("extension bytes"));
            }
            Value::Extension(ty.0, Arc::from(v.as_slice()))
        }
        Object::Array(a) => {
            bounded_len(a.len(), limits)?;
            let depth = depth
                .checked_add(1)
                .ok_or(ProtocolError::Limit("value depth"))?;
            Value::Array(a.map(|v| value(v, limits, depth)).collect::<Result<_>>()?)
        }
        Object::Map(m) => {
            bounded_len(m.len(), limits)?;
            let depth = depth
                .checked_add(1)
                .ok_or(ProtocolError::Limit("value depth"))?;
            Value::Map(
                m.map(|v| Ok((value(v.key, limits, depth)?, value(v.value, limits, depth)?)))
                    .collect::<Result<_>>()?,
            )
        }
        v => Value::Int(integer(v)?),
    })
}

pub(super) fn owned_dictionary(v: Obj<'_>, limits: Limits) -> Result<BTreeMap<Arc<str>, Value>> {
    let mut ret = BTreeMap::new();
    for item in dictionary(v, limits)? {
        let (key, v) = item?;
        if key.len() > limits.max_string_bytes {
            return Err(ProtocolError::Limit("dictionary key"));
        }
        bounded_insert(&mut ret, Arc::from(key), value(v, limits, 0)?, limits)?;
    }
    Ok(ret)
}

pub(super) fn color(v: Obj<'_>) -> Result<Option<u32>> {
    match integer(v)? {
        -1 => Ok(None),
        n @ 0..=0xff_ffff => Ok(Some(n as u32)),
        _ => invalid("color", "invalid color"),
    }
}

pub(super) fn attributes(v: Obj<'_>, limits: Limits) -> Result<HighlightAttributes> {
    let mut ret = HighlightAttributes::default();
    for item in dictionary(v, limits)? {
        let (key, v) = item?;
        match key {
            "foreground" => ret.foreground = color(v)?,
            "background" => ret.background = color(v)?,
            "special" => ret.special = color(v)?,
            "reverse" => ret.reverse = boolean(v)?,
            "italic" => ret.italic = boolean(v)?,
            "bold" => ret.bold = boolean(v)?,
            "strikethrough" => ret.strikethrough = boolean(v)?,
            "underline" => ret.underline = boolean(v)?,
            "undercurl" => ret.undercurl = boolean(v)?,
            "underdouble" => ret.underdouble = boolean(v)?,
            "underdotted" => ret.underdotted = boolean(v)?,
            "underdashed" => ret.underdashed = boolean(v)?,
            "altfont" => ret.altfont = boolean(v)?,
            "dim" => ret.dim = boolean(v)?,
            "blink" => ret.blink = boolean(v)?,
            "conceal" => ret.conceal = boolean(v)?,
            "overline" => ret.overline = boolean(v)?,
            "blend" => {
                let n = integer(v)?;
                if !(0..=100).contains(&n) {
                    return invalid("highlight", "invalid blend");
                }
                ret.blend = Some(n as u8);
            }
            "url" => ret.url = Some(text(v, limits)?),
            _ => {
                bounded_insert(&mut ret.extra, Arc::from(key), value(v, limits, 0)?, limits)?;
            }
        }
    }
    Ok(ret)
}

pub(super) fn content(v: Obj<'_>, limits: Limits) -> Result<Content> {
    let a = array(v)?;
    bounded_len(a.len(), limits)?;
    a.map(|v| {
        let mut a = array(v)?;
        Ok(Chunk {
            highlight: hl_id(next(&mut a)?)?,
            text: text(next(&mut a)?, limits)?,
            hl_id: a.next().map(integer).transpose()?,
        })
    })
    .collect()
}

pub(super) fn contents(v: Obj<'_>, limits: Limits) -> Result<Vec<Content>> {
    let a = array(v)?;
    bounded_len(a.len(), limits)?;
    a.map(|v| content(v, limits)).collect()
}

pub(super) fn check_position(content: &Content, pos: usize) -> Result<()> {
    let mut remaining = pos;
    for chunk in content {
        if remaining <= chunk.text.len() {
            return if chunk.text.is_char_boundary(remaining) {
                Ok(())
            } else {
                invalid("cmdline", "cursor is not on a UTF-8 boundary")
            };
        }
        // The returning branch above covers remaining <= chunk.text.len().
        remaining -= chunk.text.len();
    }
    if remaining == 0 {
        Ok(())
    } else {
        invalid("cmdline", "cursor outside content")
    }
}

pub(super) fn selected(v: Obj<'_>, count: usize) -> Result<Option<usize>> {
    match integer(v)? {
        -1 => Ok(None),
        n if n >= 0 && (n as u128) < count as u128 => Ok(Some(n as usize)),
        _ => invalid("popupmenu", "selection outside items"),
    }
}
