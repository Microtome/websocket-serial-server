use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

/// Create a bidirectional channel
/// Returns two sender-receiver channel pairs
/// One pair should be given to one thread
/// and the other pair to another thread
pub fn bichannel<T, U>() -> ((Sender<T>, Receiver<U>), (Sender<U>, Receiver<T>)) {
  let side1 = channel::<T>();
  let side2 = channel::<U>();
  return ((side1.0, side2.1), (side2.0, side1.1));
}
