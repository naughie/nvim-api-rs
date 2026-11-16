use super::Error;
use crate::PresentationError;

use crate::{Backend, Frame};

use tokio::sync::Notify;

use std::sync::{Arc, Mutex, MutexGuard};

struct Pending {
    frame: Option<Frame>,
    error: Option<Error>,
    closed: bool,
    receiver_alive: bool,
}

struct Shared {
    pending: Mutex<Pending>,
    ready: Notify,
}

impl Shared {
    fn pending(&self) -> MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub(super) struct Sender {
    shared: Arc<Shared>,
    max_events: usize,
}

impl Sender {
    pub(super) fn send(&self, frame: Frame) -> Result<(), Error> {
        let mut pending = self.shared.pending();
        if !pending.receiver_alive {
            return Err(Error::ReceiverClosed);
        }
        if pending.closed {
            return Err(Error::Protocol(crate::ProtocolError::Failed));
        }
        if let Some(previous) = &mut pending.frame {
            previous
                .coalesce(frame, self.max_events)
                .map_err(Error::Protocol)?;
        } else {
            pending.frame = Some(frame);
        }
        drop(pending);
        self.shared.ready.notify_one();
        Ok(())
    }

    pub(super) fn fail(&self, error: Error) {
        let mut pending = self.shared.pending();
        if pending.error.is_none() {
            pending.error = Some(error);
        }
        pending.closed = true;
        drop(pending);
        self.shared.ready.notify_one();
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.shared.pending().closed = true;
        self.shared.ready.notify_one();
    }
}

/// Receives stable UI frames from [`super::UiHandler`].
///
/// Frames remain in receive order. Intermediate states may be coalesced when
/// presentation is slower than redraw processing; their ordered events are
/// preserved. Await the plugin driver separately for connection failures.
pub struct Frames {
    shared: Arc<Shared>,
    finished: bool,
}

impl Frames {
    pub(super) fn channel(max_events: usize) -> (Sender, Self) {
        let shared = Arc::new(Shared {
            pending: Mutex::new(Pending {
                frame: None,
                error: None,
                closed: false,
                receiver_alive: true,
            }),
            ready: Notify::new(),
        });
        (
            Sender {
                shared: Arc::clone(&shared),
                max_events,
            },
            Self {
                shared,
                finished: false,
            },
        )
    }

    /// Waits for the next frame or terminal UI processing error.
    ///
    /// Returns `None` after the handler is dropped and the pending frame has
    /// been drained, or after returning a terminal error. Cancelling this wait
    /// does not consume a frame.
    pub async fn recv(&mut self) -> Option<Result<Frame, Error>> {
        if self.finished {
            return None;
        }
        loop {
            let ready = self.shared.ready.notified();
            {
                let mut pending = self.shared.pending();
                if let Some(frame) = pending.frame.take() {
                    return Some(Ok(frame));
                }
                if let Some(error) = pending.error.take() {
                    self.finished = true;
                    return Some(Err(error));
                }
                if pending.closed {
                    self.finished = true;
                    return None;
                }
            }
            // notify_one retains a permit when no receiver is waiting; a
            // publication between checking the slot and awaiting is observed.
            ready.await;
        }
    }

    /// Receives and presents one frame on the calling thread.
    ///
    /// Returns `false` when the stream has ended. A backend error consumes that
    /// frame and leaves the connection active. Handle the frame's events before
    /// returning an error if they must be retained. Recover with a full repaint
    /// or [`crate::State::diff_since()`] relative to the last painted state.
    pub async fn present_next<B: Backend + ?Sized>(
        &mut self,
        backend: &mut B,
    ) -> Result<bool, PresentationError<B::Error>> {
        let Some(frame) = self.recv().await else {
            return Ok(false);
        };
        backend
            .present(frame.map_err(|e| PresentationError::Session(e.into()))?)
            .map_err(PresentationError::Backend)?;
        Ok(true)
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        let mut pending = self.shared.pending();
        pending.receiver_alive = false;
        pending.frame = None;
        pending.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Damage, Event};

    use tokio::time::{Duration, timeout};

    fn frame(sequence: u64) -> Frame {
        Frame {
            sequence,
            state: Arc::default(),
            damage: Damage::default(),
            events: vec![Event::SetTitle(sequence.to_string().into())],
        }
    }

    #[tokio::test]
    async fn concurrent_delivery_never_overtakes_pending_events() {
        let (sender, mut frames) = Frames::channel(2000);
        let producer = std::thread::spawn(move || {
            for sequence in 1..=2000 {
                sender.send(frame(sequence)).unwrap();
            }
        });
        let mut events = Vec::new();
        let mut previous = 0;
        timeout(Duration::from_secs(2), async {
            while let Some(frame) = frames.recv().await {
                let frame = frame.unwrap();
                assert!(frame.sequence > previous);
                previous = frame.sequence;
                events.extend(frame.events.into_iter().map(|event| {
                    let Event::SetTitle(title) = event else {
                        panic!("expected title event");
                    };
                    title.to_string()
                }));
            }
        })
        .await
        .expect("pending frame was stranded after its sender stopped");
        producer.join().unwrap();
        assert_eq!(previous, 2000);
        assert_eq!(
            events,
            (1..=2000).map(|n| n.to_string()).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn cancelling_reception_preserves_later_frames_and_pending_close() {
        let (sender, mut frames) = Frames::channel(2);
        tokio::select! {
            result = frames.recv() => panic!("unexpected frame: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        sender.send(frame(1)).unwrap();
        sender.send(frame(2)).unwrap();
        drop(sender);
        let frame = frames.recv().await.unwrap().unwrap();
        assert_eq!(frame.sequence, 2);
        assert_eq!(frame.events.len(), 2);
        assert!(frames.recv().await.is_none());
        assert!(frames.recv().await.is_none());
    }
}
