use std::error::Error;

use async_channel::Receiver;

#[derive(Debug)]
pub enum PressType {
    LongPress,
    ShortPress,
}

pub struct SteamInstance {
    steam_path: String,
}

impl SteamInstance {
    pub fn fetch() -> Result<SteamInstance, Box<dyn Error>> {
        let home = std::env::var("HOME")?;
        let steam_path = format!("{}/.steam/root/ubuntu12_32/steam", home);
        Ok(SteamInstance { steam_path })
    }

    pub async fn listen(&self, channel: Receiver<PressType>) -> Result<(), Box<dyn Error>> {
        loop {
            while let Ok(press_type) = channel.recv().await {
                self.do_press(press_type)?;
            }
        }
    }

    pub fn do_press(&self, press_type: PressType) -> Result<(), Box<dyn Error>> {
        let command = match press_type {
            PressType::LongPress => "long",
            PressType::ShortPress => "short",
        };

        let mut handle = std::process::Command::new("steam-run")
            .args([self.steam_path.as_str(), "-ifrunning", format!("steam://{}powerpress", command).as_str()])
            .spawn()?;

        handle.wait()?;

        Ok(())
    }
}