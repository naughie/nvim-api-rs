use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::*;

use std::sync::Arc;

fn grid(width: usize, height: usize) -> Arc<Grid> {
    Arc::new(Grid {
        width,
        height,
        rows: (0..height)
            .map(|_| {
                Arc::new(Row {
                    cells: vec![Cell::default(); width],
                    wrap: false,
                })
            })
            .collect(),
    })
}

fn state_with_grids(grids: impl IntoIterator<Item = (GridId, Arc<Grid>)>) -> State {
    State {
        grids: Arc::new(grids.into_iter().collect()),
        ..State::default()
    }
}

fn replay<P: Primitive>(model: &mut Model, params: P) -> Vec<Frame> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    let Message::Notification(n) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap() else {
        panic!("expected notification")
    };
    let mut frames = Vec::new();
    model.apply_redraw(n, |f| frames.push(f)).unwrap();
    frames
}

fn assert_flags(damage: &Damage, flags: (bool, bool, bool, bool)) {
    assert_eq!(
        (damage.full, damage.layout, damage.cursor, damage.widgets),
        flags
    );
}

fn assert_empty(damage: &Damage) {
    assert_flags(damage, (false, false, false, false));
    assert_eq!(damage.grids, []);
    assert_eq!(damage.removed_grids, []);
}

#[test]
fn shared_snapshots_and_rows_have_no_damage() {
    let previous = state_with_grids([(GridId(1), grid(4, 3))]);
    assert_empty(&previous.diff_since(&previous));
    assert_empty(&previous.clone().diff_since(&previous));

    let mut current = previous.clone();
    let current_grid = Arc::make_mut(&mut current.grids)
        .get_mut(&GridId(1))
        .unwrap();
    Arc::make_mut(current_grid);
    assert!(!Arc::ptr_eq(&current.grids, &previous.grids));
    assert!(!Arc::ptr_eq(
        &current.grids[&GridId(1)],
        &previous.grids[&GridId(1)]
    ));
    assert_empty(&current.diff_since(&previous));

    let current_grid = Arc::make_mut(&mut current.grids)
        .get_mut(&GridId(1))
        .unwrap();
    Arc::make_mut(&mut Arc::make_mut(current_grid).rows[1]).wrap = true;
    let damage = current.diff_since(&previous);
    assert_flags(&damage, (false, false, false, false));
    assert_eq!(
        damage.grids,
        [GridDamage {
            grid: GridId(1),
            top: 1,
            bottom: 2,
            left: 0,
            right: 4,
        }]
    );
}

#[test]
fn compares_the_last_painted_snapshot_across_skipped_frames() {
    let mut model = Model::default();
    let initial = replay(&mut model, (("grid_resize", (1, 4, 4)), ("flush", ())));
    let previous = Arc::clone(&initial[0].state);
    let frames = replay(
        &mut model,
        (
            ("grid_line", (1, 0, 1, (("a", 0),), false)),
            ("flush", ()),
            ("grid_line", (1, 3, 2, (("b", 0),), false)),
            ("flush", ()),
        ),
    );
    let latest = &frames[1];
    let damage = latest.state.diff_since(&previous);
    assert_flags(&damage, (false, false, false, false));
    assert_eq!(
        damage.grids,
        [GridDamage {
            grid: GridId(1),
            top: 0,
            bottom: 4,
            left: 0,
            right: 4,
        }]
    );
    assert_eq!(latest.damage.grids[0].top, 3);
    assert_eq!(
        previous.grids[&GridId(1)].cell(0, 1).unwrap().text.as_ref(),
        " "
    );
    assert_eq!(
        latest.state.grids[&GridId(1)]
            .cell(0, 1)
            .unwrap()
            .text
            .as_ref(),
        "a"
    );
    assert_eq!(
        latest.state.grids[&GridId(1)]
            .cell(3, 2)
            .unwrap()
            .text
            .as_ref(),
        "b"
    );
}

#[test]
fn reports_added_resized_and_removed_grids_in_grid_order() {
    let previous = state_with_grids([
        (GridId(1), grid(2, 2)),
        (GridId(9), grid(1, 1)),
        (GridId(5), grid(1, 1)),
    ]);
    let mut current = previous.clone();
    let grids = Arc::make_mut(&mut current.grids);
    grids.remove(&GridId(9));
    grids.remove(&GridId(5));
    grids.insert(GridId(1), grid(3, 2));
    grids.insert(GridId(3), grid(4, 1));

    let damage = current.diff_since(&previous);
    assert_flags(&damage, (false, true, false, false));
    assert_eq!(damage.removed_grids, [GridId(5), GridId(9)]);
    assert_eq!(
        damage.grids,
        [
            GridDamage {
                grid: GridId(1),
                top: 0,
                bottom: 2,
                left: 0,
                right: 3
            },
            GridDamage {
                grid: GridId(3),
                top: 0,
                bottom: 1,
                left: 0,
                right: 4
            },
        ]
    );
    assert_eq!(previous.grids.len(), 3);
    assert_eq!(previous.grids[&GridId(1)].width, 2);
}

#[test]
fn color_highlight_and_option_changes_require_full_repaint() {
    let previous = state_with_grids([(GridId(1), grid(1, 1))]);
    let mut colors = previous.clone();
    colors.default_colors.cterm_foreground = 4;
    let mut highlights = previous.clone();
    Arc::make_mut(&mut highlights.highlights).insert(HighlightId(3), Highlight::default());
    let mut options = previous.clone();
    Arc::make_mut(&mut options.options)
        .insert(Arc::from("guifont"), Value::String(Arc::from(&b"Mono"[..])));
    for current in [colors, highlights, options] {
        let damage = current.diff_since(&previous);
        assert_flags(&damage, (true, false, false, false));
        assert_eq!(damage.grids, []);
    }
}

#[test]
fn cursor_mode_layout_and_widget_changes_set_their_flags() {
    let previous = state_with_grids([(GridId(1), grid(2, 2))]);
    let mut cursor = previous.clone();
    cursor.cursor = Some(Cursor {
        grid: GridId(1),
        row: 1,
        col: 0,
    });
    let mut mode = previous.clone();
    mode.mode.name = Arc::from("insert");
    let mut styles = previous.clone();
    Arc::make_mut(&mut styles.modes).push(ModeInfo::default());
    let mut busy = previous.clone();
    busy.busy = true;
    for current in [cursor, mode, styles, busy] {
        assert_flags(&current.diff_since(&previous), (false, false, true, false));
    }

    let mut placements = previous.clone();
    Arc::make_mut(&mut placements.placements).insert(
        GridId(1),
        GridPlacement {
            kind: GridPlacementKind::External {
                window: nvim_rpc::Window::default(),
            },
            hidden: false,
        },
    );
    let mut viewports = previous.clone();
    Arc::make_mut(&mut viewports.viewports).insert(GridId(1), WindowViewport::default());
    for current in [placements, viewports] {
        assert_flags(&current.diff_since(&previous), (false, true, false, false));
    }

    let mut messages = previous.clone();
    Arc::make_mut(&mut messages.messages).history_prev_cmd = true;
    let mut groups = previous.clone();
    Arc::make_mut(&mut groups.highlight_groups).insert(Arc::from("Pmenu"), HighlightId(3));
    let mut popup = previous.clone();
    popup.popupmenu = Some(Arc::new(Popupmenu {
        items: Vec::new(),
        selected: None,
        anchor: PopupAnchor::Grid {
            grid: GridId(1),
            row: 0,
            col: 0,
        },
    }));
    for current in [messages, groups, popup.clone()] {
        assert_flags(&current.diff_since(&previous), (false, false, false, true));
    }
    assert_flags(&previous.diff_since(&popup), (false, false, false, true));
}

#[test]
fn compares_unrelated_snapshots_conservatively() {
    let previous = state_with_grids([(GridId(1), grid(2, 2))]);
    let current = state_with_grids([(GridId(2), grid(3, 1))]);
    let damage = current.diff_since(&previous);
    assert!(damage.full);
    assert!(damage.layout);
    assert_eq!(damage.removed_grids, [GridId(1)]);
    assert_eq!(
        damage.grids,
        [GridDamage {
            grid: GridId(2),
            top: 0,
            bottom: 1,
            left: 0,
            right: 3,
        }]
    );

    let equal = state_with_grids([(GridId(1), grid(2, 2))]);
    let damage = equal.diff_since(&previous);
    assert!(damage.full);
    assert_eq!(damage.removed_grids, []);
    assert_eq!(
        damage.grids,
        [GridDamage {
            grid: GridId(1),
            top: 0,
            bottom: 2,
            left: 0,
            right: 2,
        }]
    );
}

#[test]
fn malformed_public_row_counts_keep_damage_within_grid_bounds() {
    let previous = state_with_grids([(GridId(1), grid(2, 0))]);
    let mut current = previous.clone();
    let grid = Arc::make_mut(&mut current.grids)
        .get_mut(&GridId(1))
        .unwrap();
    Arc::make_mut(grid).rows = vec![Arc::new(Row {
        cells: Vec::new(),
        wrap: false,
    })];
    let mut next = current.clone();
    let grid = Arc::make_mut(&mut next.grids).get_mut(&GridId(1)).unwrap();
    Arc::make_mut(&mut Arc::make_mut(grid).rows[0]).wrap = true;
    for damage in [current.diff_since(&previous), next.diff_since(&current)] {
        assert_eq!(
            damage.grids,
            [GridDamage {
                grid: GridId(1),
                top: 0,
                bottom: 0,
                left: 0,
                right: 2,
            }]
        );
    }
}
