use crate::shared_state::{SharedState, ChannelError};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::thread;
/// Sender part of the channel
pub struct Sender<T> {
        pub state: Arc<SharedState<T>>
}

impl<T> Sender<T> {
        /// Send an item on the channel
        pub fn send(&self, item: T) -> Result<(), ChannelError> {
                let mut elements = self.state.elements.lock().unwrap();

                if self.state.closed.load(Ordering::Relaxed) {
                        return Err(ChannelError::ChannelClosed);
                }

                elements.push_back(item);
                self.state.is_empty.notify_one();
                Ok(())
        }

        /// Close the channel
        pub fn close(&mut self) -> Result<(), ChannelError> {
                self.state.closed.store(true, Ordering::Relaxed);
                Ok(())
        }
}

impl<T: Clone> Clone for Sender<T> {
        fn clone(&self) -> Self {
                // increment the counter
                self.state.num_senders.fetch_add(1, Ordering::Relaxed);

                // return a new sender
                Sender {
                        state:  self.state.clone()
                }
        }
}

impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
                if self.state.num_senders.load(Ordering::Relaxed) == 1 {
                        self.state.closed.store(true, Ordering::Relaxed);
                }
                self.state.num_senders.fetch_sub(1, Ordering::Relaxed);
        }
}
