use crate::frame::Frames;
use crate::rpc::{Message, MessageId};
use crate::{ConnectionError, ConnectionOptions, FromParams, Handler, Neovim};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Notify;
use tokio::task::{JoinHandle, JoinSet};

use std::collections::{HashSet, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, Mutex, MutexGuard};

/// Starts a connection and its handler tasks on the current Tokio runtime.
///
/// `from_neovim` receives Neovim's RPC messages and `to_neovim` sends messages
/// back. They may be process pipes, files, or socket halves. For a plugin started
/// by Neovim, pass its stdin as `from_neovim` and stdout as `to_neovim`.
/// Pass the underlying writer; [`Neovim`] buffers encoded chunks and flushes
/// every complete request, notification, and reply.
///
/// The returned task owns the receive loop and all handler tasks. Await it to
/// observe EOF, transport errors, invalid notification arguments, or handler
/// panics. EOF or a connection failure stops handler tasks and discards queued
/// callbacks; aborting the task does the same. Closing the connection wakes
/// pending requests. Messages own immutable [`crate::Bytes`]. See [`Handler`]
/// for callback execution and reply ordering. This compatibility API leaves
/// incoming frames and pending-work counts unbounded; use [`crate::connect()`]
/// for explicit bounds and managed task ownership.
pub fn spawn<R, W, H>(
    from_neovim: R,
    to_neovim: W,
    handler: H,
) -> (Neovim<W>, JoinHandle<Result<(), ConnectionError>>)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    H: Handler<W>,
{
    spawn_with_options(
        from_neovim,
        to_neovim,
        handler,
        ConnectionOptions::UNLIMITED,
        false,
    )
}

pub(crate) fn spawn_with_options<R, W, H>(
    from_neovim: R,
    to_neovim: W,
    handler: H,
    options: ConnectionOptions,
    drain_on_eof: bool,
) -> (Neovim<W>, JoinHandle<Result<(), ConnectionError>>)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    H: Handler<W>,
{
    let nvim = Neovim::new(to_neovim, options.max_pending_requests);
    let guard = CloseGuard(nvim.clone());
    let task = tokio::spawn(async move {
        let guard = guard;
        run(from_neovim, guard.0.clone(), handler, options, drain_on_eof).await
    });
    (nvim, task)
}

struct CloseGuard<W>(Neovim<W>);

impl<W> Drop for CloseGuard<W> {
    fn drop(&mut self) {
        self.0.close_with_error(ConnectionError::Closed);
    }
}

enum Reply<R> {
    Value(R),
    InvalidParams(String),
}

struct Slot<R> {
    id: MessageId,
    reply: Option<Reply<R>>,
}

struct Replies<R> {
    state: Mutex<ReplyState<R>>,
    ready: Notify,
    max_pending: usize,
}

struct ReplyState<R> {
    stack: Vec<Slot<R>>,
    // Empty while scanning shallow stacks; otherwise contains every stack ID,
    // including completed slots. Index until drained, retaining capacity for reuse.
    ids: HashSet<MessageId>,
    receiving: bool,
}

// Linear lookup wins on shallow stacks; release-mode burst and interleaved
// measurements put the crossover around 64 pending slots.
const REPLY_INDEX_THRESHOLD: usize = 64;

impl<R> Replies<R> {
    fn state(&self) -> MutexGuard<'_, ReplyState<R>> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn begin_batch(&self) {
        self.state().receiving = true;
    }

    fn end_batch(&self) {
        let mut state = self.state();
        let ready = state.receiving && state.stack.last().is_some_and(|slot| slot.reply.is_some());
        state.receiving = false;
        drop(state);
        if ready {
            self.ready.notify_one();
        }
    }

    fn push(&self, id: MessageId) -> Result<usize, ConnectionError> {
        let mut state = self.state();
        if state.stack.len() >= self.max_pending {
            return Err(ConnectionError::Limit("pending RPC replies"));
        }
        let ReplyState { stack, ids, .. } = &mut *state;
        if ids.is_empty() && stack.len() < REPLY_INDEX_THRESHOLD {
            if stack.iter().any(|slot| slot.id == id) {
                return Err(ConnectionError::InvalidFormat);
            }
        } else {
            if ids.is_empty() {
                ids.extend(stack.iter().map(|slot| slot.id));
            }
            if !ids.insert(id) {
                return Err(ConnectionError::InvalidFormat);
            }
        }
        let idx = stack.len();
        stack.push(Slot { id, reply: None });
        Ok(idx)
    }

    fn complete(&self, idx: usize, reply: Reply<R>) {
        let mut state = self.state();
        // Registration returns idx < len. Pushes append; pops remove only
        // completed suffix slots. This unfinished slot therefore still exists
        // at idx, and idx + 1 <= len <= usize::MAX.
        state.stack[idx].reply = Some(reply);
        let ready = !state.receiving && idx + 1 == state.stack.len();
        drop(state);
        if ready {
            self.ready.notify_one();
        }
    }

    fn is_ready(&self) -> bool {
        let state = self.state();
        !state.receiving && state.stack.last().is_some_and(|slot| slot.reply.is_some())
    }

    fn pop_ready(&self) -> Option<(MessageId, Reply<R>)> {
        let mut state = self.state();
        if state.receiving {
            return None;
        }
        state.stack.last()?.reply.as_ref()?;
        let slot = state.stack.pop()?;
        if !state.ids.is_empty() {
            state.ids.remove(&slot.id);
        }
        Some((slot.id, slot.reply?))
    }
}

enum Completion<R> {
    Request(usize, R),
    Notification,
    Writer(Result<(), ConnectionError>),
}

enum Callback {
    Request(usize, crate::rpc::Request),
    Notification(crate::rpc::Notification),
}

async fn write_replies<W, R>(
    nvim: Neovim<W>,
    replies: Arc<Replies<R>>,
) -> Result<(), ConnectionError>
where
    W: AsyncWrite + Unpin + Send + 'static,
    R: crate::IntoResponse,
{
    loop {
        let ready = replies.ready.notified();
        if !replies.is_ready() {
            tokio::select! {
                () = ready => {},
                () = nvim.closed() => return Err(nvim.connection_error()),
            }
            continue;
        }
        let mut writer = nvim.writer().await?;
        let Some((id, reply)) = replies.pop_ready() else {
            continue;
        };
        match reply {
            Reply::Value(value) => nvim.respond(&mut writer, id, value).await?,
            Reply::InvalidParams(error) => {
                nvim.respond(&mut writer, id, Err::<Infallible, _>(error))
                    .await?;
            }
        }
    }
}

async fn run<R, W, H>(
    reader: R,
    nvim: Neovim<W>,
    handler: H,
    options: ConnectionOptions,
    drain_on_eof: bool,
) -> Result<(), ConnectionError>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    H: Handler<W>,
{
    let mut frames = Frames::new(reader, options.max_rpc_bytes);
    let handler = Arc::new(handler);
    let replies = Arc::new(Replies {
        state: Mutex::new(ReplyState {
            stack: Vec::new(),
            ids: HashSet::new(),
            receiving: false,
        }),
        ready: Notify::new(),
        max_pending: options.max_pending_replies,
    });
    let mut tasks = JoinSet::new();
    let mut callbacks = VecDeque::new();
    let mut notifying = false;
    let mut eof = false;
    let reply_writer = write_replies(nvim.clone(), Arc::clone(&replies));
    tasks.spawn(async move { Completion::Writer(reply_writer.await) });

    let result = 'receive: loop {
        if nvim.is_close_requested() {
            break Err(nvim.connection_error());
        }
        while !notifying {
            let Some(callback) = callbacks.pop_front() else {
                break;
            };
            tokio::task::consume_budget().await;
            if (!eof && nvim.is_closed()) || nvim.is_close_requested() {
                break 'receive Err(nvim.connection_error());
            }
            match callback {
                Callback::Request(idx, request) => {
                    let params = match H::Request::from_params(request.params) {
                        Ok(params) => params,
                        Err(error) => {
                            replies.complete(idx, Reply::InvalidParams(error.to_string()));
                            continue;
                        }
                    };
                    let handler = Arc::clone(&handler);
                    let nvim = nvim.clone();
                    tasks.spawn(async move {
                        let value = handler
                            .on_request(request.method.as_slice(), params, nvim)
                            .await;
                        Completion::Request(idx, value)
                    });
                }
                Callback::Notification(notification) => {
                    let params = match H::Notification::from_params(notification.params) {
                        Ok(params) => params,
                        Err(error) => break 'receive Err(ConnectionError::Decode(error)),
                    };
                    let handler = Arc::clone(&handler);
                    let nvim = nvim.clone();
                    tasks.spawn(async move {
                        handler
                            .on_notification(notification.method.as_slice(), params, nvim)
                            .await;
                        Completion::Notification
                    });
                    notifying = true;
                }
            }
        }
        if eof {
            let completed = tokio::select! {
                () = nvim.close_requested() => break Err(nvim.connection_error()),
                completed = tasks.join_next() => completed,
            };
            match completed {
                Some(Ok(Completion::Request(_, _))) | Some(Ok(Completion::Writer(Ok(())))) => {}
                Some(Ok(Completion::Notification)) => notifying = false,
                Some(Ok(Completion::Writer(Err(ConnectionError::Closed)))) => {}
                Some(Ok(Completion::Writer(Err(e)))) => break Err(e),
                Some(Err(e)) => break Err(ConnectionError::Task(Arc::new(e))),
                None => break Ok(()),
            }
            continue;
        }
        let frame = if let Some(frame) = frames.buffered() {
            Ok(Some(frame))
        } else {
            replies.end_batch();
            tokio::task::consume_budget().await;
            let frame = tokio::select! {
                () = nvim.closed() => break Err(nvim.connection_error()),
                frame = frames.next() => frame,
                completed = tasks.join_next() => {
                    let Some(completed) = completed else { break Err(ConnectionError::Closed); };
                    match completed {
                        Ok(Completion::Request(idx, value)) => replies.complete(idx, Reply::Value(value)),
                        Ok(Completion::Notification) => notifying = false,
                        Ok(Completion::Writer(result)) => break result,
                        Err(error) => break Err(ConnectionError::Task(Arc::new(error))),
                    }
                    continue;
                }
            };
            // Register the requests already read before admitting their replies.
            replies.begin_batch();
            frame
        };
        let frame = match frame {
            Ok(Some(frame)) => frame,
            Ok(None) if drain_on_eof => {
                eof = true;
                nvim.close_with_error(ConnectionError::Closed);
                continue;
            }
            Ok(None) => break Ok(()),
            Err(error) => break Err(error),
        };
        // Frames returns exactly one complete value bounded by SeekState,
        // including every nested element. Header reads and lazy array/map
        // iteration therefore have all required bytes in this owned frame.
        let message: Message = crate::mp::decode::Object::from_buf_unchecked(frame).into();
        match message {
            Message::Response(response) => {
                if let Err(error) = nvim.route(response) {
                    break Err(error);
                }
            }
            Message::Request(request) => {
                if callbacks.len() >= options.max_pending_callbacks {
                    break Err(ConnectionError::Limit("pending RPC callbacks"));
                }
                let idx = match replies.push(request.msgid) {
                    Ok(idx) => idx,
                    Err(error) => break Err(error),
                };
                callbacks.push_back(Callback::Request(idx, request));
            }
            Message::Notification(notification) => {
                if callbacks.len() >= options.max_pending_callbacks {
                    break Err(ConnectionError::Limit("pending RPC callbacks"));
                }
                callbacks.push_back(Callback::Notification(notification));
            }
            Message::InvalidFormat => break Err(ConnectionError::InvalidFormat),
        }
    };
    nvim.close_with_error(
        result
            .as_ref()
            .err()
            .cloned()
            .unwrap_or(ConnectionError::Closed),
    );
    tasks.shutdown().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replies<R>(max_pending: usize) -> Replies<R> {
        Replies {
            state: Mutex::new(ReplyState {
                stack: Vec::new(),
                ids: HashSet::new(),
                receiving: false,
            }),
            ready: Notify::new(),
            max_pending,
        }
    }

    #[test]
    fn nested_completions_keep_their_slots_as_newer_replies_are_sent() {
        let replies = replies(usize::MAX);
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
        for depth in [1, REPLY_INDEX_THRESHOLD, REPLY_INDEX_THRESHOLD + 1, 4096] {
            let replies = replies(usize::MAX);
            let depth = u32::try_from(depth).unwrap();
            for id in 0..depth {
                assert_eq!(replies.push(MessageId(id)).unwrap(), id as usize);
            }

            replies.begin_batch();
            for id in 0..depth {
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::InvalidFormat)
                ));
                replies.complete(id as usize, Reply::Value(id));
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::InvalidFormat)
                ));
            }
            assert!(!replies.is_ready());
            assert!(replies.pop_ready().is_none());
            replies.end_batch();

            for id in (0..depth).rev() {
                assert!(matches!(
                    replies.pop_ready(),
                    Some((MessageId(actual), Reply::Value(value))) if actual == id && value == id
                ));
                // Reusing the popped ID leaves every older slot registered.
                let idx = replies.push(MessageId(id)).unwrap();
                assert_eq!(idx, id as usize);
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
        let replies = replies(usize::MAX);
        let threshold = u32::try_from(REPLY_INDEX_THRESHOLD).unwrap();
        for _ in 0..2 {
            for id in 0..threshold {
                replies.push(MessageId(id)).unwrap();
                assert!(replies.state().ids.is_empty());
            }
            replies.push(MessageId(threshold)).unwrap();
            {
                let state = replies.state();
                assert_eq!(state.ids.len(), state.stack.len());
                assert!(state.stack.iter().all(|slot| state.ids.contains(&slot.id)));
            }
            for id in (0..=threshold).rev() {
                replies.complete(id as usize, Reply::Value(id));
                assert!(replies.state().ids.contains(&MessageId(id)));
                assert!(replies.pop_ready().is_some());
                assert!(!replies.state().ids.contains(&MessageId(id)));
                assert_eq!(replies.state().ids.len(), id as usize);
            }
            let capacity = replies.state().ids.capacity();
            assert!(capacity > 0);
            let idx = replies.push(MessageId(u32::MAX)).unwrap();
            assert!(replies.state().ids.is_empty());
            replies.complete(idx, Reply::Value(u32::MAX));
            assert!(replies.pop_ready().is_some());
            assert_eq!(replies.state().ids.capacity(), capacity);
        }
    }

    #[test]
    fn pending_reply_limit_precedes_duplicate_rejection() {
        for max_pending in [0, 1, REPLY_INDEX_THRESHOLD + 1] {
            let replies = replies::<u32>(max_pending);
            for id in 0..u32::try_from(max_pending).unwrap() {
                replies.push(MessageId(id)).unwrap();
            }
            for id in [0, u32::MAX] {
                assert!(matches!(
                    replies.push(MessageId(id)),
                    Err(ConnectionError::Limit("pending RPC replies"))
                ));
            }
            assert_eq!(replies.state().stack.len(), max_pending);
        }
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
                replies.complete(idx as usize, Reply::Value(u64::from(idx)));
            }
            for _ in 0..depth {
                black_box(replies.pop_ready().unwrap());
            }
        }

        for depth in [1, 4, 16, 32, 64, 128, 4096, 16384] {
            let cycles = 262_144 / depth;
            let reused = replies(usize::MAX);
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
                let fresh = replies(usize::MAX);
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
