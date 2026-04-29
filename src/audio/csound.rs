use crate::audio::control::{ControlChannel, ControlMessage, Update};
use crate::ui::types::Channel;
use csound::Csound;
use std::collections::HashMap;

/// Audio engine
pub struct Audio {}

pub struct ReadChannelMap {
    floats: HashMap<Channel, f64>,
}

impl ReadChannelMap {
    pub fn new(channels: &[Channel]) -> ReadChannelMap {
        ReadChannelMap {
            floats: channels.iter().map(|chan| (chan.clone(), 0.0)).collect(),
        }
    }
}

impl Audio {
    pub fn run(
        file_path: String,
        mut control: ControlChannel,
        mut prev_channels_to_read: ReadChannelMap,
        control_channels: Vec<Channel>,
    ) {
        let file = file_path.clone();

        tokio::spawn(async move {
            let csound = Csound::new().expect("Failed to create Csound instance");
            csound.compile_csd(file, 0, 1).unwrap();
            csound.start().expect("Failed to run csound");

            csound.perform_ksmps();
            send_init_control_channels(&control_channels, &csound, &mut control);
            let mut should_run = ShouldRun::default();

            while !csound.perform_ksmps() && should_run.0 {
                control.on_recv(|msg| {
                    should_run = react_on_control_message(&csound, msg);
                });

                let update = get_read_updates(&mut prev_channels_to_read, &csound);
                if !update.is_empty() {
                    control.send(ControlMessage::Updates { updates: update });
                }
            }
        });
    }
}

fn send_init_control_channels(channels: &[Channel], csound: &Csound, control: &mut ControlChannel) {
    let updates: Vec<Update> = channels
        .iter()
        .flat_map(|chan| {
            csound.get_control_channel(&chan.0).map(|value| Update {
                channel: chan.clone(),
                value,
            })
        })
        .collect();
    control.send(ControlMessage::Updates {
        updates: updates.clone(),
    })
}

fn get_read_updates(prev_channels_to_read: &mut ReadChannelMap, csound: &Csound) -> Vec<Update> {
    let mut res = Vec::new();

    for (key, prev_value) in prev_channels_to_read.floats.iter_mut() {
        if let Ok(current_value) = csound.get_control_channel(&key.0) {
            if current_value != *prev_value {
                res.push(Update {
                    channel: key.clone(),
                    value: current_value,
                });
                *prev_value = current_value;
            }
        } else {
            println!("WARN: failed to read channel by name {:?}", key);
        }
    }
    res
}

fn update_channel(csound: &Csound, update: &Update) {
    match csound.set_control_channel(&update.channel.0, update.value) {
        Ok(_) => {}
        Err(_) => {
            println!("WARN: failed to set channel {:?}", update.channel);
        }
    };
}

struct ShouldRun(bool);

impl Default for ShouldRun {
    fn default() -> Self {
        ShouldRun(true)
    }
}

fn react_on_control_message(csound: &Csound, msg: &ControlMessage) -> ShouldRun {
    println!("UI to Csound update: {:?}", msg);
    match msg {
        ControlMessage::Updates { updates } => {
            updates.iter().for_each(|update| {
                update_channel(csound, update);
            });
            ShouldRun(true)
        }
        ControlMessage::ExitAudio => ShouldRun(false),
    }
}
