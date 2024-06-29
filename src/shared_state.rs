use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::thread;
use crate::sender::Sender;
use crate::receiver::Receiver;

#[derive(Debug)]
pub enum ChannelError {
        ChannelClosed,
        ChannelEmpty,
        RecvBlocked
}


/// The shared state between the sender and the receiver
pub struct SharedState<T> {
        pub elements: Mutex<VecDeque<T>>,
        pub is_empty: Condvar,
        pub closed: AtomicBool,
        pub num_senders: AtomicUsize,
}


pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let shared_state = Arc::new(SharedState {
                elements: Mutex::new(VecDeque::new()),
                is_empty: Condvar::new(),
                closed: AtomicBool::new(false),
                num_senders: AtomicUsize::new(1)
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