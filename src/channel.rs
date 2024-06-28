use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};

pub enum ChannelError {
        ChannelEmpty,
        ChannelClosed,
}

/// Sender part of the channel
pub struct Sender<T> {
        sender: Arc<SharedState<T>>
}

impl<T> Sender<T> {
        pub fn send(&self, item: T) -> Result<(), ChannelError> {
                let (mut elements, condvar) = (self.sender.elements.lock().unwrap(), &self.sender.is_empty);
                elements.push_back(item);
                condvar.notify_one();
                Ok(())
        }
}

/// Receiver part of the channel
pub struct Receiver<T> {
        receiver: Arc<SharedState<T>>
}

impl<T> Receiver<T> {
        pub fn recv(&self) -> Result<T, ChannelError> {
                // make sure this is not closed
                if self.receiver.closed {
                        return Err(ChannelError::ChannelClosed);
                }

                // get the lock and condvar
                let (mut elements, condvar) = (self.receiver.elements.lock().unwrap(), &self.receiver.is_empty);

                // if there are no elements in the channel, just wait
                while elements.len() == 0 {
                        elements = condvar.wait(elements).unwrap();
                }
                // we have an element, return it
                Ok(elements.pop_front().unwrap())
        }
}

/// The shared state between the sender and the receiver
pub struct SharedState<T> {
        elements: Mutex<VecDeque<T>>,
        is_empty: Condvar,
        closed: bool
}


pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let shared_state = Mutex::new(VecDeque::new());
        let shared_state = Arc::new(SharedState { elements: shared_state , is_empty: Condvar::new(), closed: false});
        let sender = Sender { sender: shared_state.clone() };
        let receiver = Receiver { receiver: shared_state };
        (sender, receiver)
}
    
#[cfg(test)]
mod tests {
        use super::*;

        #[test]
        fn test_channel_creation() {
            let (tx, rx) = channel::<i32>();
            // Basic test to ensure channel creation doesn't panic
        }

}