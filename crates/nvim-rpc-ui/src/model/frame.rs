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
        self.events
            .len()
            .checked_add(next.events.len())
            .filter(|&n| n <= max_events)
            .ok_or(ProtocolError::Limit("pending events"))?;
        self.events.append(&mut next.events);
        next.events = std::mem::take(&mut self.events);
        next.damage.full = true;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Damage, Event, State};

    use std::sync::Arc;

    fn frame(sequence: u64, events: Vec<Event>) -> Frame {
        Frame {
            sequence,
            state: Arc::default(),
            damage: Damage::default(),
            events,
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
}
