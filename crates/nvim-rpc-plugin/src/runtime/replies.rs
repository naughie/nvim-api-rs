use crate::ConnectionError;
use crate::rpc::MessageId;

use tokio::sync::Notify;

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

pub(super) enum Reply<R> {
    Value(R),
    InvalidParams(String),
}

/// Identifies an unfinished request's position in the reply stack.
#[derive(Debug)]
pub(super) struct SlotId(usize);

struct Slot<R> {
    id: MessageId,
    reply: Option<Reply<R>>,
}

/// Keeps request IDs registered until their replies are taken for writing.
struct Stack<R> {
    slots: Vec<Slot<R>>,
    // Empty in shallow stacks; otherwise indexes every slot until the stack
    // drains. The empty set retains its allocation for the next deep stack.
    ids: HashSet<MessageId>,
    limit: usize,
}

// Shallow stacks avoid hashing and allocating an index.
const INDEX_THRESHOLD: usize = 64;

impl<R> Stack<R> {
    fn new(limit: usize) -> Self {
        Self {
            slots: Vec::new(),
            ids: HashSet::new(),
            limit,
        }
    }

    fn push(&mut self, id: MessageId) -> Result<SlotId, ConnectionError> {
        if self.slots.len() >= self.limit {
            return Err(ConnectionError::Limit("pending RPC replies"));
        }
        if self.ids.is_empty() && self.slots.len() < INDEX_THRESHOLD {
            if self.slots.iter().any(|slot| slot.id == id) {
                return Err(ConnectionError::InvalidFormat);
            }
        } else {
            if self.ids.is_empty() {
                self.ids.extend(self.slots.iter().map(|slot| slot.id));
            }
            if !self.ids.insert(id) {
                return Err(ConnectionError::InvalidFormat);
            }
        }
        let slot = SlotId(self.slots.len());
        self.slots.push(Slot { id, reply: None });
        Ok(slot)
    }

    fn complete(&mut self, SlotId(idx): SlotId, reply: Reply<R>) -> bool {
        // Only push() creates tokens, and each token is completed once. Pushes
        // append; pops remove only completed suffix slots. This unfinished slot
        // therefore still exists at idx, and idx + 1 <= len <= usize::MAX.
        self.slots[idx].reply = Some(reply);
        idx + 1 == self.slots.len()
    }

    fn is_ready(&self) -> bool {
        self.slots.last().is_some_and(|slot| slot.reply.is_some())
    }

    fn pop_ready(&mut self) -> Option<(MessageId, Reply<R>)> {
        self.slots.last()?.reply.as_ref()?;
        let slot = self.slots.pop()?;
        self.ids.remove(&slot.id);
        Some((slot.id, slot.reply?))
    }
}

pub(super) struct Replies<R> {
    state: Mutex<State<R>>,
    ready: Notify,
}

struct State<R> {
    stack: Stack<R>,
    receiving: bool,
}

impl<R> State<R> {
    fn is_ready(&self) -> bool {
        !self.receiving && self.stack.is_ready()
    }
}

impl<R> Replies<R> {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            state: Mutex::new(State {
                stack: Stack::new(limit),
                receiving: false,
            }),
            ready: Notify::new(),
        }
    }

    fn state(&self) -> MutexGuard<'_, State<R>> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn begin_batch(&self) {
        self.state().receiving = true;
    }

    pub(super) fn end_batch(&self) {
        let mut state = self.state();
        let ready = state.receiving && state.stack.is_ready();
        state.receiving = false;
        drop(state);
        if ready {
            self.ready.notify_one();
        }
    }

    pub(super) fn push(&self, id: MessageId) -> Result<SlotId, ConnectionError> {
        self.state().stack.push(id)
    }

    pub(super) fn complete(&self, slot: SlotId, reply: Reply<R>) {
        let mut state = self.state();
        let ready = state.stack.complete(slot, reply) && !state.receiving;
        drop(state);
        if ready {
            self.ready.notify_one();
        }
    }

    pub(super) async fn wait_ready(&self) {
        loop {
            let ready = self.ready.notified();
            if self.state().is_ready() {
                return;
            }
            ready.await;
        }
    }

    pub(super) fn pop_ready(&self) -> Option<(MessageId, Reply<R>)> {
        let mut state = self.state();
        if state.receiving {
            return None;
        }
        state.stack.pop_ready()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_completions_keep_their_slots_as_newer_replies_are_sent() {
        let replies = Replies::new(usize::MAX);
        let oldest = replies.push(MessageId(10)).unwrap();
        let middle = replies.push(MessageId(11)).unwrap();
        let newest = replies.push(MessageId(12)).unwrap();
        replies.complete(middle, Reply::Value(11));
        assert!(replies.pop_ready().is_none());
        replies.complete(newest, Reply::Value(12));
        assert!(matches!(
            replies.pop_ready(),
            Some((MessageId(12), Reply::Value(12)))
        ));

        let nested = replies.push(MessageId(13)).unwrap();
        replies.complete(oldest, Reply::Value(10));
        assert!(replies.pop_ready().is_none());
        replies.complete(nested, Reply::Value(13));
        for id in [13, 11, 10] {
            assert!(matches!(
                replies.pop_ready(),
                Some((MessageId(actual), Reply::Value(value))) if actual == id && value == id
            ));
        }
        assert!(replies.pop_ready().is_none());
    }

    #[test]
    fn duplicate_ids_remain_registered_until_their_slots_are_popped() {
        for depth in [1, INDEX_THRESHOLD, INDEX_THRESHOLD + 1, 4096] {
            let replies = Replies::new(usize::MAX);
            let depth = u32::try_from(depth).unwrap();
            for id in 0..depth {
                assert_eq!(replies.push(MessageId(id)).unwrap().0, id as usize);
            }

            replies.begin_batch();
            for id in 0..depth {
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::InvalidFormat)
                ));
                replies.complete(SlotId(id as usize), Reply::Value(id));
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::InvalidFormat)
                ));
            }
            assert!(!replies.state().is_ready());
            assert!(replies.pop_ready().is_none());
            replies.end_batch();

            for id in (0..depth).rev() {
                assert!(matches!(
                    replies.pop_ready(),
                    Some((MessageId(actual), Reply::Value(value))) if actual == id && value == id
                ));
                // Reusing the popped ID leaves every older slot registered.
                let idx = replies.push(MessageId(id)).unwrap();
                assert_eq!(idx.0, id as usize);
                replies.complete(idx, Reply::InvalidParams("invalid arguments".into()));
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::InvalidFormat)
                ));
                assert!(matches!(
                    replies.pop_ready(),
                    Some((MessageId(actual), Reply::InvalidParams(error)))
                        if actual == id && error == "invalid arguments"
                ));
            }
            assert!(replies.pop_ready().is_none());
            assert!(replies.pop_ready().is_none());
        }
    }

    #[test]
    fn indexes_deep_stacks_until_drained_and_reuses_storage() {
        let replies = Replies::new(usize::MAX);
        let threshold = u32::try_from(INDEX_THRESHOLD).unwrap();
        for _ in 0..2 {
            for id in 0..threshold {
                replies.push(MessageId(id)).unwrap();
                assert!(replies.state().stack.ids.is_empty());
            }
            replies.push(MessageId(threshold)).unwrap();
            {
                let state = replies.state();
                assert_eq!(state.stack.ids.len(), state.stack.slots.len());
                assert!(
                    state
                        .stack
                        .slots
                        .iter()
                        .all(|slot| state.stack.ids.contains(&slot.id))
                );
            }
            for id in (0..=threshold).rev() {
                replies.complete(SlotId(id as usize), Reply::Value(id));
                assert!(replies.state().stack.ids.contains(&MessageId(id)));
                assert!(replies.pop_ready().is_some());
                assert!(!replies.state().stack.ids.contains(&MessageId(id)));
                assert_eq!(replies.state().stack.ids.len(), id as usize);
            }
            let capacity = replies.state().stack.ids.capacity();
            assert!(capacity > 0);
            let idx = replies.push(MessageId(u32::MAX)).unwrap();
            assert!(replies.state().stack.ids.is_empty());
            replies.complete(idx, Reply::Value(u32::MAX));
            assert!(replies.pop_ready().is_some());
            assert_eq!(replies.state().stack.ids.capacity(), capacity);
        }
    }

    #[test]
    fn pending_reply_limit_precedes_duplicate_rejection() {
        for max_pending in [0, 1, INDEX_THRESHOLD + 1] {
            let replies = Replies::<u32>::new(max_pending);
            for id in 0..u32::try_from(max_pending).unwrap() {
                replies.push(MessageId(id)).unwrap();
            }
            for id in [0, u32::MAX] {
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::Limit("pending RPC replies"))
                ));
            }
            assert_eq!(replies.state().stack.slots.len(), max_pending);
        }
    }

    #[tokio::test]
    async fn readiness_wait_survives_cancellation_and_respects_batches() {
        let replies = Replies::new(1);
        replies.begin_batch();
        let slot = replies.push(MessageId(7)).unwrap();
        replies.complete(slot, Reply::Value(42));

        {
            let ready = replies.wait_ready();
            tokio::pin!(ready);
            tokio::select! {
                biased;
                () = &mut ready => panic!("reply became ready before the batch ended"),
                () = tokio::task::yield_now() => {},
            }
        }
        replies.end_batch();
        tokio::time::timeout(std::time::Duration::from_secs(1), replies.wait_ready())
            .await
            .unwrap();
        assert!(matches!(
            replies.pop_ready(),
            Some((MessageId(7), Reply::Value(42)))
        ));

        let slot = replies.push(MessageId(7)).unwrap();
        replies.complete(slot, Reply::Value(43));
        replies.wait_ready().await;
        assert!(matches!(
            replies.pop_ready(),
            Some((MessageId(7), Reply::Value(43)))
        ));
    }

    #[test]
    #[ignore = "manual release-mode reply-stack benchmark; prints timings"]
    fn benchmark_reply_stacks() {
        use std::hint::black_box;
        use std::time::Instant;

        fn fill(replies: &Replies<u64>, depth: u32) {
            for id in 0..depth {
                black_box(replies.push(MessageId(black_box(id))).unwrap());
            }
        }

        fn drain(replies: &Replies<u64>, depth: u32) {
            for idx in 0..depth {
                replies.complete(SlotId(idx as usize), Reply::Value(u64::from(idx)));
            }
            for _ in 0..depth {
                black_box(replies.pop_ready().unwrap());
            }
        }

        for depth in [1, 4, 16, 32, 64, 128, 4096, 16384] {
            let cycles = 262_144 / depth;
            let reused = Replies::new(usize::MAX);
            fill(&reused, depth);
            drain(&reused, depth);
            let start = Instant::now();
            for _ in 0..cycles {
                fill(&reused, depth);
                drain(&reused, depth);
            }
            eprintln!(
                "reused burst, depth {depth}: {:.2} ns/request",
                start.elapsed().as_nanos() as f64 / f64::from(cycles * depth),
            );

            let start = Instant::now();
            for _ in 0..cycles {
                let fresh = Replies::new(usize::MAX);
                fill(&fresh, depth);
                drain(&fresh, depth);
            }
            eprintln!(
                "fresh burst, depth {depth}: {:.2} ns/request",
                start.elapsed().as_nanos() as f64 / f64::from(cycles * depth),
            );

            fill(&reused, depth);
            let start = Instant::now();
            for _ in 0..262_144 {
                let idx = reused.push(MessageId(black_box(depth))).unwrap();
                reused.complete(idx, Reply::Value(u64::from(depth)));
                black_box(reused.pop_ready().unwrap());
            }
            eprintln!(
                "interleaved, depth {depth}: {:.2} ns/request",
                start.elapsed().as_nanos() as f64 / 262_144.0,
            );
        }
    }
}
