//! This module provides an implementation of streams as overeager receivers of messages.
//! Here 'overeager' means that one message is always received in advance.

use super::Stream;

use crossbeam::channel::{bounded, unbounded};
use crossbeam::channel::{Receiver, Sender};

/// [`OvereagerReceiver<X>`] abstracts receivers of messages of type `X` which always buffer one message.
pub struct OvereagerReceiver<X> {
    /// overeagerly received message
    message: X,
    /// receiver of messages
    receiver: Receiver<X>,
}

impl<X> OvereagerReceiver<X> {
    /// Create a channel with an overeager receiver instead of a normal one.
    /// - `cap` is the number of messages the channel can hold where `0` means it can hold any number of messages.
    /// - `message` is an initial placeholder for what the overeager receiver overeagerly receives.
    ///
    /// # Examples
    ///
    /// Creating a stream with head `true` and tail whatever is passed by `tx`:
    ///
    /// ```
    /// let (tx, stream) = rspl::streams::overeager_receivers::OvereagerReceiver::channel(0, true);
    /// ```
    pub fn channel(cap: usize, message: X) -> (Sender<X>, Self) {
        let (tx, receiver) = if cap > 0 { bounded(cap) } else { unbounded() };
        (tx, Self { message, receiver })
    }
}

impl<X> Stream<X> for OvereagerReceiver<X> {
    /// Make the message buffer of `self` the head.
    fn head(&self) -> &X {
        &self.message
    }

    /// Blocks the current thread until it can make `self` with an updated message buffer the tail.
    ///
    /// # Panics
    ///
    /// A panic is caused if the channel becomes disconnected.
    fn tail(mut self) -> Self {
        self.message = self.receiver.recv().unwrap();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::unbounded as channel;

    use crate::assert_head_eq;
    use crate::assert_tail_starts_with;

    #[macro_export]
    macro_rules! enqueue {
        ($tx:expr, $xs:expr) => {
            for x in $xs {
                $tx.send(x).unwrap();
            }
        };
    }

    #[test]
    fn test_overeager_channel() {
        let (tx, mut stream) = OvereagerReceiver::channel(1, false);
        enqueue!(tx, [true]);
        assert_head_eq!(stream, false);
        assert_tail_starts_with!(stream, [true]);
    }

    #[test]
    fn test_head() {
        let (_, rx) = channel();
        let stream = OvereagerReceiver {
            message: true,
            receiver: rx,
        };
        assert!(stream.head());
    }

    #[test]
    fn test_tail() {
        let (tx, rx) = channel();
        let stream = OvereagerReceiver {
            message: false,
            receiver: rx,
        };
        enqueue!(tx, [true]);
        assert!(stream.tail().head());
    }
}
