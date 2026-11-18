use super::Frame;

use crate::ProtocolError;

impl Frame {
    /// Combines the next received frame with this pending frame.
    ///
    /// Keeps the next frame's state and sequence, appends its events after this
    /// frame's events, and requests a full repaint. Call in receive order for
    /// frames from the same UI attachment. This permits painting only the latest
    /// state while retaining every event in order.
    ///
    /// Returns [`ProtocolError::Limit`] if the combined event count exceeds
    /// `max_events`. On failure this frame is unchanged and `next` is dropped.
    /// Application-held snapshots are not included in this event limit.
    pub fn coalesce(&mut self, mut next: Self, max_events: usize) -> Result<(), ProtocolError> {
        self.check_events(&next, max_events)?;
        self.events.append(&mut next.events);
        self.sequence = next.sequence;
        self.state = next.state;
        self.damage = next.damage;
        self.damage.full = true;
        Ok(())
    }

    /// Combines the next received frame while preserving incremental damage.
    ///
    /// Keeps the latest state and sequence and appends events in receive order,
    /// as in [`Self::coalesce()`]. Call for consecutive published frames from
    /// the same UI attachment. Nonconsecutive sequences or either input's
    /// full-repaint flag request a full repaint instead.
    ///
    /// Combines cursor, layout, and widget flags and retains at most one bounding
    /// rectangle per surviving grid, clipped to its current dimensions. Resized
    /// or recreated grids are repainted completely. Removed IDs are retained
    /// only while absent from the latest state. Each damage list is bounded by
    /// `max_grids`; exceeding this bound requests a full repaint and clears both
    /// lists. For a full repaint, the backend must also discard grids and
    /// placements absent from the latest state.
    ///
    /// Returns [`ProtocolError::Limit`] if the combined event count exceeds
    /// `max_events`. On failure this frame is unchanged and `next` is dropped.
    /// Application-held snapshots are outside these limits.
    pub fn coalesce_incremental(
        &mut self,
        mut next: Self,
        max_events: usize,
        max_grids: usize,
    ) -> Result<(), ProtocolError> {
        self.check_events(&next, max_events)?;
        self.damage.full |= self.sequence.checked_add(1) != Some(next.sequence);
        self.damage
            .merge(next.damage, &self.state, &next.state, max_grids);
        self.events.append(&mut next.events);
        self.sequence = next.sequence;
        self.state = next.state;
        Ok(())
    }

    fn check_events(&self, next: &Self, max_events: usize) -> Result<(), ProtocolError> {
        self.events
            .len()
            .checked_add(next.events.len())
            .filter(|&n| n <= max_events)
            .ok_or(ProtocolError::Limit("pending events"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Damage, Event, GridDamage, GridId, Model, State};

    use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
    use nvim_rpc::rpc::Message;

    use std::sync::Arc;

    fn frame(sequence: u64, events: Vec<Event>) -> Frame {
        Frame {
            sequence,
            state: Arc::default(),
            damage: Damage::default(),
            events,
        }
    }

    fn replay(model: &mut Model, params: impl Primitive) -> Vec<Frame> {
        let mut bytes = Vec::new();
        bytes.write_primitive((2u8, "redraw", params)).consume();
        let Message::Notification(n) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap() else {
            panic!("expected redraw notification");
        };
        let mut frames = Vec::new();
        model.apply_redraw(n, |frame| frames.push(frame)).unwrap();
        frames
    }

    fn region(grid: i64, top: usize, bottom: usize, left: usize, right: usize) -> GridDamage {
        GridDamage {
            grid: GridId(grid),
            top,
            bottom,
            left,
            right,
        }
    }

    #[test]
    fn coalesces_states_without_losing_event_order() {
        let mut pending = frame(1, vec![Event::Bell]);
        let state = Arc::new(State {
            title: Arc::from("latest"),
            ..State::default()
        });
        let next = Frame {
            state: Arc::clone(&state),
            ..frame(3, vec![Event::VisualBell])
        };
        pending.coalesce(next, 2).unwrap();
        assert_eq!(pending.sequence, 3);
        assert!(Arc::ptr_eq(&pending.state, &state));
        assert!(pending.damage.full);
        assert!(matches!(
            pending.events.as_slice(),
            [Event::Bell, Event::VisualBell]
        ));
    }

    #[test]
    fn event_limit_leaves_pending_frame_unchanged() {
        let mut pending = frame(1, vec![Event::Bell]);
        let state = Arc::clone(&pending.state);
        assert!(matches!(
            pending.coalesce(frame(2, vec![Event::VisualBell]), 1),
            Err(ProtocolError::Limit("pending events"))
        ));
        assert_eq!(pending.sequence, 1);
        assert!(Arc::ptr_eq(&pending.state, &state));
        assert!(!pending.damage.full);
        assert!(matches!(pending.events.as_slice(), [Event::Bell]));
        let mut empty = frame(0, vec![]);
        empty.coalesce(frame(1, vec![]), 0).unwrap();
        assert!(empty.events.is_empty());
    }

    #[test]
    fn incremental_coalescing_unions_distant_cells_per_grid() {
        let mut model = Model::default();
        replay(
            &mut model,
            (("grid_resize", (1, 8, 6), (2, 8, 6)), ("flush", ())),
        );
        let mut frames = replay(
            &mut model,
            (
                ("grid_line", (1, 0, 1, (("a", 0),), false)),
                ("bell", ()),
                ("flush", ()),
                ("grid_line", (1, 5, 6, (("b", 0),), false)),
                ("grid_line", (2, 2, 3, (("c", 0),), false)),
                ("visual_bell", ()),
                ("flush", ()),
            ),
        )
        .into_iter();
        let mut pending = frames.next().unwrap();
        let retained = Arc::clone(&pending.state);
        let next = frames.next().unwrap();
        let latest = Arc::clone(&next.state);
        pending.coalesce_incremental(next, 2, 2).unwrap();
        assert_eq!(pending.sequence, 3);
        assert!(Arc::ptr_eq(&pending.state, &latest));
        assert!(!pending.damage.full);
        assert_eq!(
            pending.damage.grids,
            [region(1, 0, 6, 1, 7), region(2, 2, 3, 3, 4)]
        );
        assert_eq!(pending.damage.removed_grids, []);
        assert!(matches!(
            pending.events.as_slice(),
            [Event::Bell, Event::VisualBell]
        ));
        assert_eq!(
            retained.grids[&GridId(1)].cell(5, 6).unwrap().text.as_ref(),
            " "
        );
        assert_eq!(
            latest.grids[&GridId(1)].cell(5, 6).unwrap().text.as_ref(),
            "b"
        );
    }

    #[test]
    fn incremental_coalescing_combines_flags_and_preserves_full_repaint() {
        for lhs in 0..16 {
            for rhs in 0..16 {
                let damage = |bits| Damage {
                    full: bits & 1 != 0,
                    layout: bits & 2 != 0,
                    cursor: bits & 4 != 0,
                    widgets: bits & 8 != 0,
                    ..Damage::default()
                };
                let mut pending = Frame {
                    damage: damage(lhs),
                    ..frame(1, vec![])
                };
                let next = Frame {
                    damage: damage(rhs),
                    ..frame(2, vec![])
                };
                pending.coalesce_incremental(next, 0, 0).unwrap();
                let expected = damage(lhs | rhs);
                assert_eq!(
                    (
                        pending.damage.full,
                        pending.damage.layout,
                        pending.damage.cursor,
                        pending.damage.widgets
                    ),
                    (
                        expected.full,
                        expected.layout,
                        expected.cursor,
                        expected.widgets
                    ),
                    "flag inputs {lhs}, {rhs}",
                );
                assert!(pending.damage.grids.is_empty());
                assert!(pending.damage.removed_grids.is_empty());
            }
        }
    }

    #[test]
    fn incremental_coalescing_repaints_resizes_and_discards_obsolete_regions() {
        let mut model = Model::default();
        replay(&mut model, (("grid_resize", (1, 8, 6)), ("flush", ())));
        let mut pending = replay(
            &mut model,
            (("grid_line", (1, 5, 7, (("x", 0),), false)), ("flush", ())),
        )
        .pop()
        .unwrap();
        for (width, height) in [(3, 2), (5, 4)] {
            let next = replay(
                &mut model,
                (("grid_resize", (1, width, height)), ("flush", ())),
            )
            .pop()
            .unwrap();
            pending.coalesce_incremental(next, 0, 1).unwrap();
            assert!(!pending.damage.full);
            assert!(pending.damage.layout);
            assert_eq!(pending.damage.grids, [region(1, 0, height, 0, width)]);
        }
        let next = replay(&mut model, (("grid_destroy", (1,)), ("flush", ())))
            .pop()
            .unwrap();
        pending.coalesce_incremental(next, 0, 1).unwrap();
        assert!(!pending.damage.full);
        assert_eq!(pending.damage.grids, []);
        assert_eq!(pending.damage.removed_grids, [GridId(1)]);
    }

    #[test]
    fn incremental_coalescing_reconciles_recreated_grid_ids() {
        for split_flush in [false, true] {
            let mut model = Model::default();
            replay(&mut model, (("grid_resize", (1, 4, 3)), ("flush", ())));
            let mut pending = replay(
                &mut model,
                (("grid_line", (1, 2, 2, (("x", 0),), false)), ("flush", ())),
            )
            .pop()
            .unwrap();
            replay(&mut model, (("grid_destroy", (1,)),));
            if split_flush {
                let next = replay(&mut model, (("flush", ()),)).pop().unwrap();
                pending.coalesce_incremental(next, 0, 1).unwrap();
                assert_eq!(pending.damage.removed_grids, [GridId(1)]);
            }
            let next = replay(&mut model, (("grid_resize", (1, 4, 3)), ("flush", ())))
                .pop()
                .unwrap();
            pending.coalesce_incremental(next, 0, 1).unwrap();
            assert!(!pending.damage.full);
            assert!(pending.damage.layout);
            assert_eq!(pending.damage.removed_grids, []);
            assert_eq!(pending.damage.grids, [region(1, 0, 3, 0, 4)]);
            assert_eq!(
                pending.state.grids[&GridId(1)]
                    .cell(2, 2)
                    .unwrap()
                    .text
                    .as_ref(),
                " "
            );
        }
    }

    #[test]
    fn incremental_coalescing_clips_and_normalizes_public_regions() {
        let mut model = Model::default();
        let mut pending = replay(&mut model, (("grid_resize", (1, 4, 3)), ("flush", ())))
            .pop()
            .unwrap();
        pending.damage.grids = vec![
            region(1, 1, usize::MAX, 2, usize::MAX),
            region(1, 0, 1, 0, 1),
            region(9, 0, 1, 0, 1),
            region(1, 3, 2, 4, 3),
        ];
        let next = Frame {
            state: Arc::clone(&pending.state),
            damage: Damage {
                grids: vec![region(1, 0, 0, 0, 4)],
                ..Damage::default()
            },
            ..frame(2, vec![])
        };
        pending.coalesce_incremental(next, 0, 1).unwrap();
        assert!(!pending.damage.full);
        assert_eq!(pending.damage.grids, [region(1, 0, 3, 0, 4)]);
    }

    #[test]
    fn incremental_coalescing_bounds_grid_damage_and_removal_churn() {
        let mut model = Model::default();
        let mut pending = replay(
            &mut model,
            (("grid_resize", (1, 2, 2), (2, 2, 2)), ("flush", ())),
        )
        .pop()
        .unwrap();
        let next = replay(&mut model, (("flush", ()),)).pop().unwrap();
        pending.coalesce_incremental(next, 0, 1).unwrap();
        assert!(pending.damage.full);
        assert!(pending.damage.grids.is_empty());
        assert!(pending.damage.removed_grids.is_empty());

        let mut model = Model::default();
        let mut pending = replay(&mut model, (("flush", ()),)).pop().unwrap();
        for id in 1..=3 {
            let next = replay(
                &mut model,
                (
                    ("grid_resize", (id, 2, 2)),
                    ("grid_destroy", (id,)),
                    ("flush", ()),
                ),
            )
            .pop()
            .unwrap();
            pending.coalesce_incremental(next, 0, 2).unwrap();
            assert!(pending.damage.grids.is_empty());
            if id <= 2 {
                assert!(!pending.damage.full);
                assert_eq!(pending.damage.removed_grids.len(), id as usize);
            } else {
                assert!(pending.damage.full);
                assert!(pending.damage.removed_grids.is_empty());
            }
        }
    }

    #[test]
    fn incremental_coalescing_falls_back_for_nonconsecutive_sequences() {
        for (previous, next) in [(1, 3), (2, 1), (2, 2), (u64::MAX, 0)] {
            let mut pending = frame(previous, vec![]);
            pending
                .coalesce_incremental(frame(next, vec![]), 0, 0)
                .unwrap();
            assert!(pending.damage.full, "sequences {previous}, {next}");
            assert_eq!(pending.sequence, next);
        }
    }

    #[test]
    fn incremental_event_limit_preserves_all_pending_damage() {
        let mut model = Model::default();
        let mut pending = replay(&mut model, (("grid_resize", (1, 4, 3)), ("flush", ())))
            .pop()
            .unwrap();
        pending.events.push(Event::Bell);
        pending.damage.removed_grids.push(GridId(9));
        pending.damage.full = true;
        pending.damage.layout = true;
        pending.damage.cursor = true;
        pending.damage.widgets = true;
        let state = Arc::clone(&pending.state);
        let next = replay(&mut model, (("visual_bell", ()), ("flush", ())))
            .pop()
            .unwrap();
        assert!(matches!(
            pending.coalesce_incremental(next, 1, 0),
            Err(ProtocolError::Limit("pending events"))
        ));
        assert_eq!(pending.sequence, 1);
        assert!(Arc::ptr_eq(&pending.state, &state));
        assert!(matches!(pending.events.as_slice(), [Event::Bell]));
        assert!(pending.damage.full);
        assert!(pending.damage.layout);
        assert!(pending.damage.cursor);
        assert!(pending.damage.widgets);
        assert_eq!(pending.damage.grids, [region(1, 0, 3, 0, 4)]);
        assert_eq!(pending.damage.removed_grids, [GridId(9)]);
    }
}
