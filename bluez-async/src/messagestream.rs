use dbus::Message;
use dbus::channel::MatchingReceiver;
use dbus::nonblock::{MsgMatch, SyncConnection};
use futures::Stream;
use futures::channel::mpsc::UnboundedReceiver;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// Wrapper for a stream of D-Bus messages which automatically removes the `MsgMatch` from the D-Bus
/// connection when it is dropped.
pub struct MessageStream {
    msg_match: Option<MsgMatch>,
    /// The match rule, to remove it from the D-Bus daemon when the stream is dropped.
    match_str: String,
    events: UnboundedReceiver<Message>,
    connection: Arc<SyncConnection>,
}

impl MessageStream {
    pub fn new(msg_match: MsgMatch, match_str: String, connection: Arc<SyncConnection>) -> Self {
        let (msg_match, events) = msg_match.msg_stream();
        Self {
            msg_match: Some(msg_match),
            match_str,
            events,
            connection,
        }
    }
}

impl Stream for MessageStream {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.events).poll_next(cx)
    }
}

impl Drop for MessageStream {
    fn drop(&mut self) {
        let msg_match = self.msg_match.take().unwrap();
        // Remove the local filter now, before `events` is dropped. Otherwise a signal arriving in
        // between finds the receiver gone, and dbus-rs removes the filter by itself, so a later
        // `remove_match` would fail before it reached the daemon (#419).
        self.connection.stop_receive(msg_match.token());
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            log::debug!("No Tokio runtime to remove the D-Bus match for a dropped stream");
            return;
        };
        let connection = self.connection.clone();
        let match_str = std::mem::take(&mut self.match_str);
        runtime.spawn(async move {
            // This can fail, for example once the D-Bus connection has been lost. Panicking here
            // would abort applications built with `panic = "abort"`, so log the error instead.
            if let Err(e) = connection.remove_match_no_cb(&match_str).await {
                log::debug!(
                    "Failed to remove the D-Bus match for a dropped stream: {}",
                    e
                );
            }
        });
    }
}
