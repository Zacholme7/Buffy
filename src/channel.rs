use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;


#[derive(Debug)]
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

        #[test]
        fn test_send_and_recv() {
                let (tx, rx) = channel::<usize>();
                tx.send(10).unwrap();
                assert_eq!(rx.recv().unwrap(), 10);
        }

        #[test]
        fn test_multiple_send_and_recv() {
                let (tx, rx) = channel::<usize>();
                tx.send(10).unwrap();
                tx.send(11).unwrap();
                tx.send(12).unwrap();
                tx.send(13).unwrap();
                assert_eq!(rx.recv().unwrap(), 10);
                assert_eq!(rx.recv().unwrap(), 11);
                assert_eq!(rx.recv().unwrap(), 12);
                assert_eq!(rx.recv().unwrap(), 13);
        }

        #[test]
        fn test_send_and_try_recv() {
            let (tx, rx) = channel();
            tx.send(42).unwrap();
            assert_eq!(rx.try_recv().unwrap(), 42);
            assert!(matches!(rx.try_recv(), Err(ChannelError::ChannelEmpty)));
        }
    
        #[test]
        fn test_send_after_close() {
            let (mut tx, rx) = channel::<i32>();
            tx.close().unwrap();
            assert!(matches!(tx.send(42), Err(ChannelError::ChannelClosed)));
            assert!(matches!(rx.recv(), Err(ChannelError::ChannelClosed)));
        }
    
        #[test]
        fn test_send_and_recv_multiple_threads() {
            let (tx, rx) = channel();
            let tx_thread = thread::spawn(move || {
                for i in 0..100 {
                    tx.send(i).unwrap();
                }
            });
            let rx_thread = thread::spawn(move || {
                let mut sum = 0;
                for _ in 0..100 {
                    sum += rx.recv().unwrap();
                }
                sum
            });
            tx_thread.join().unwrap();
            assert_eq!(rx_thread.join().unwrap(), 4950); // sum of 0..99
        }

}