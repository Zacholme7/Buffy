use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};


pub enum ChannelError {
        ChannelClosed,
        ChannelEmpty,
        RecvBlocked
}

/// Sender part of the channel
pub struct Sender<T> {
        state: Arc<SharedState<T>>
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

/// Receiver part of the channel
pub struct Receiver<T> {
        state: Arc<SharedState<T>>
}

impl<T> Receiver<T> {
        pub fn recv(&self) -> Result<T, ChannelError> {
                let (mut elements, condvar) = (self.state.elements.lock().unwrap(), &self.state.is_empty);
                while elements.is_empty() {
                        if self.state.closed.load(Ordering::Relaxed) {
                                return Err(ChannelError::ChannelClosed);
                        }
                        elements = condvar.wait(elements).unwrap();
                }
                // we have an element, return it
                Ok(elements.pop_front().unwrap())
        }

        pub fn try_recv(&self) -> Result<T, ChannelError> {
                if self.state.closed.load(Ordering::Acquire) {
                    if let Ok(mut guard) = self.state.elements.try_lock() {
                        return if let Some(item) = guard.pop_front() {
                            Ok(item)
                        } else {
                            Err(ChannelError::ChannelClosed)
                        };
                    }
                }
                
                if let Ok(mut guard) = self.state.elements.try_lock() {
                    if let Some(item) = guard.pop_front() {
                        Ok(item)
                    } else {
                        Err(ChannelError::ChannelEmpty)
                    }
                } else {
                    Err(ChannelError::RecvBlocked)
                }
            }
}

/// The shared state between the sender and the receiver
pub struct SharedState<T> {
        elements: Mutex<VecDeque<T>>,
        is_empty: Condvar,
        closed: AtomicBool
}


pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let shared_state = Arc::new(SharedState {
                elements: Mutex::new(VecDeque::new()),
                is_empty: Condvar::new(),
                closed: AtomicBool::new(false)
        });

        let sender = Sender { state: shared_state.clone() };
        let receiver = Receiver { state: shared_state };
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