use crate::audio::control::ControlChannel;
use csound::Csound;

/// Audio engine
pub struct Audio {}

impl Audio {
    pub fn run(file_path: String, mut control: ControlChannel) {
        let file = file_path.clone();
        println!("CSD 0");
        tokio::spawn(async move {
            println!("CSD 1");
            let csound = Csound::new().expect("Failed to create Csound instance");
            csound.compile_csd(file, 0, 1).unwrap();
            csound.start().expect("Failed to run csound");
            println!("CSD 2");
            while !csound.perform_ksmps() {
                control.on_recv(|msg| {
                    println!("UI to Csound update: {:?}", msg);
                    msg.update.iter().for_each(|update| {
                        match csound.set_control_channel(&update.channel.0, update.value as f64) {
                            Ok(_) => {}
                            Err(_) => {
                                println!("WARN: failed to set channel {:?}", update.channel.0);
                            }
                        };
                    });
                });
            }
        });
    }
}
