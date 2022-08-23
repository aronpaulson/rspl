//! This module provides an implementation of streams as overeager receivers of messages.
//! Here 'overeager' means that always one message is received in advance.

use super::Stream;

use crossbeam::channel::{bounded, unbounded};
use crossbeam::channel::{Receiver, Sender};

/// [`OvereagerReceiver<X>`] abstracts receivers of message of type `X` which can buffer one message.
pub struct OvereagerReceiver<X> {
    /// overeagerly received message
    message: X,
    /// receiver of messages
    receiver: Receiver<X>,
}

impl<X> Stream<X> for OvereagerReceiver<X>
where
    X: Copy,
{
    /// Make the overeagerly received part of `self` the head.
    fn head(&self) -> X {
        self.message
    }

    /// Updated the message-buffer of `self` with the next `self` can get and consider it the tail.
    ///
    /// A panic is caused if receiving fails (probably due to a disconnected channel).
    fn tail(mut self) -> Self {
        let x = self.receiver.recv().unwrap();
        self.message = x;
        self
    }
}

impl<X> OvereagerReceiver<X> {
    /// Create a channel with an overeager receiver instead of a normal one.
    /// - `cap` is the number of messages the channel can hold where `0` means it can hold any number of messages.
    /// - `message` is an initial placeholder for what the overeagerly receiver overeagerly receives.
    ///
    /// # Examples
    ///
    /// Creating a stream with head `true` and tail whatever is passed by `tx`:
    ///
    /// ```
    /// let (tx, stream) = rspl::OvereagerReceiver::channel(0, true);
    /// ```
    pub fn channel(cap: usize, message: X) -> (Sender<X>, OvereagerReceiver<X>) {
        let (tx, receiver) = if cap > 0 { bounded(cap) } else { unbounded() };
        (tx, OvereagerReceiver { message, receiver })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::unbounded as channel;

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
        tx.send(true).unwrap();
        assert!(stream.tail().head());
    }

    #[test]
    fn test_overeager_channel() {
        let (tx, stream) = OvereagerReceiver::channel(1, false);
        tx.send(true).unwrap();
        assert!(stream.tail().head());
    }
}
