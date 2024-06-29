use crate::shared_state::{SharedState, ChannelError};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::thread;



/// Receiver part of the channel
pub struct Receiver<T> {
        pub state: Arc<SharedState<T>>
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