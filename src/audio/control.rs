//! Send control messages between audio and UI threads

use crate::ui::types::Channel;
use tokio_bichannel::channel;

static CONTROL_CHANNEL_MESSAGE_SIZE: usize = 20;

#[derive(Debug, Clone)]
pub enum ControlMessage {
    Updates { updates: Vec<Update> },
    ExitAudio,
}

#[derive(Debug, Clone)]
pub struct Update {
    pub channel: Channel,
    pub value: f64,
}

pub struct ControlChannel(pub tokio_bichannel::Channel<ControlMessage, ControlMessage>);

impl ControlChannel {
    // Non-blocking callback call if we have received a message
    pub fn on_recv<F: FnOnce(&ControlMessage) -> ()>(&mut self, f: F) {
        match self.0.try_recv() {
            Ok(msg) => f(&msg),
            Err(_e) => {}
        }
    }

    pub fn send(&mut self, msg: ControlMessage) {
        let res = self.0.try_send(msg);
        match res {
            Ok(_) => println!("OK sent"),
            Err(e) => println!("Err sent: {:?}", e.to_string()),
        }
    }
}

pub fn audio_control_channel() -> (ControlChannel, ControlChannel) {
    let (chan1, chan2) = channel(CONTROL_CHANNEL_MESSAGE_SIZE);
    (ControlChannel(chan1), ControlChannel(chan2))
}
