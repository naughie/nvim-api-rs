use nvim_rpc::mp::encode::primitive::ArrayIter;
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};

use std::error::Error;

pub(super) fn encode(params: impl Primitive) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    bytes
}

fn line<'a>(text: impl ExactSizeIterator<Item = &'a str>) -> Vec<u8> {
    encode((
        (
            "grid_line",
            (1, 10, 0, ArrayIter(text.map(|s| (s, 0))), false),
        ),
        ("flush", ()),
    ))
}

fn repeated(text: &[&str]) -> Vec<u8> {
    let row = text.iter().copied().cycle().take(120).collect::<Vec<_>>();
    line(row.into_iter())
}

fn vocabulary(count: usize, recurring: bool) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    // This CJK range contains 15,360 valid scalars. Frames are encoded before
    // measurement; padding the final row keeps every update at 120 entries.
    // The fixed chunk size 120 is nonzero.
    let text = (0x4e00..0x8a00)
        .take(count)
        .map(|n| char::from_u32(n).map(String::from).ok_or("invalid scalar"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(text
        .chunks(120)
        .map(|row| {
            let row = row
                .iter()
                .chain(text.iter())
                .take(120)
                .enumerate()
                .map(|(n, s)| {
                    // The fixed divisor 8 is nonzero.
                    if recurring && n % 8 == 0 {
                        "日"
                    } else {
                        s.as_str()
                    }
                })
                .collect::<Vec<_>>();
            line(row.into_iter())
        })
        .collect())
}

pub(super) struct Workload {
    pub name: &'static str,
    pub frames: Vec<Vec<u8>>,
}

pub(super) fn immediate() -> impl Iterator<Item = Result<Workload, Box<dyn Error>>> {
    let single = [
        (
            "typing 120x40",
            encode((
                ("grid_line", (1, 10, 20, (("x", 0),), false)),
                ("grid_cursor_goto", (1, 10, 21)),
                ("flush", ()),
            )),
        ),
        (
            "scroll 120x40",
            encode((
                ("grid_scroll", (1, 0, 40, 0, 120, 1, 0)),
                ("grid_line", (1, 39, 0, ((" ", 0, 120),), false)),
                ("flush", ()),
            )),
        ),
    ]
    .into_iter()
    .map(|(name, frame)| {
        Ok(Workload {
            name,
            frames: vec![frame],
        })
    });

    let rows = [
        ("ASCII row", &["a", "b", "c", "d"][..]),
        ("CJK row", &["日", "本", "語", "界"][..]),
        (
            "combining row",
            &["e\u{301}", "a\u{308}", "o\u{302}", "n\u{303}"][..],
        ),
        ("emoji row", &["😀", "🚀", "👩‍💻", "🇯🇵"][..]),
        (
            "multi-character row",
            &["hello", "world", "日本語", "<tab>"][..],
        ),
    ]
    .into_iter()
    .map(|(name, text)| {
        Ok(Workload {
            name,
            frames: vec![repeated(text)],
        })
    });

    let vocabularies = [
        ("256-scalar CJK working set", 256, false),
        ("1024-scalar CJK working set", 1024, false),
        ("4096-scalar CJK working set", 4096, false),
        ("mostly unique CJK rows", 15_360, true),
        ("cache-churn CJK rows", 15_360, false),
    ]
    .into_iter()
    .map(|(name, count, recurring)| {
        vocabulary(count, recurring).map(|frames| Workload { name, frames })
    });

    single.chain(rows).chain(vocabularies)
}

pub(super) fn bursts() -> [(&'static str, [Vec<u8>; 2]); 2] {
    let cursor_a = encode((("grid_cursor_goto", (1, 10, 20)), ("flush", ())));
    let cursor_b = encode((("grid_cursor_goto", (1, 10, 21)), ("flush", ())));
    let cells_a = encode((
        (
            "grid_line",
            (1, 10, 20, (("x", 0),), false),
            (2, 30, 50, (("u", 0),), false),
        ),
        ("grid_cursor_goto", (1, 10, 21)),
        ("flush", ()),
    ));
    let cells_b = encode((
        (
            "grid_line",
            (1, 10, 22, (("y", 0),), false),
            (2, 30, 52, (("v", 0),), false),
        ),
        ("grid_cursor_goto", (1, 10, 23)),
        ("flush", ()),
    ));
    [
        ("cursor, two 240x80 grids", [cursor_a, cursor_b]),
        ("sparse cells, two 240x80 grids", [cells_a, cells_b]),
    ]
}
