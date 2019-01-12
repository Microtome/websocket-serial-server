use std::sync::mpsc::Receiver;

use crate::messages::SubscriptionRequest;

/// Convenience type for a listener
/// that accepts weak refs of Senders of Serial Reponses
/// This is how the manager will communicate
/// results back to the websockets
pub type SubscReceiver = Receiver<SubscriptionRequest>;
