use std::{error::Error, fmt::Display, time::{Duration, Instant}};

use async_channel::Sender;
use evdev::{Device, EventStream, EventSummary, KeyCode};

use crate::steam::PressType;

pub struct PowerButton(EventStream);

impl PowerButton {
    pub fn new(device: &str) -> Result<PowerButton, Box<dyn Error>> {
        Ok(PowerButton(Device::open(device)?.into_event_stream()?))
    }

    pub fn auto_detect() -> Result<Vec<PowerButton>, Box<dyn Error>> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("input")?;
        enumerator.match_sysname("event*")?;
        enumerator.match_property("STEAMOS_POWER_BUTTON", "1")?;
        let mut reuslt = Vec::new();
        for dev in enumerator.scan_devices()? {
            if let Some(devnode) = dev.devnode() {
                match PowerButton::new(devnode.as_os_str().to_str().unwrap()) {
                    Ok(button) => { 
                        reuslt.push(button) 
                    },
                    Err(err) => eprintln!("Failed to open power button: {}", err),
                }
            }
        }
        Ok(reuslt)
    }

    pub async fn listen(&mut self, sender: Sender<PressType>) -> Result<(), Box<dyn Error>> {
        let mut pressed_time = None;
        loop {
            if let Ok(event) = self.0.next_event().await {
                if let EventSummary::Key(_, KeyCode::KEY_POWER, _) = event.destructure() {
                    if event.value() == 1 {
                        pressed_time = Some(Instant::now());
                    } else if let Some(time) = pressed_time {
                        let duration = time.elapsed();
                        if duration >= Duration::from_secs(1) {
                            println!("Button {} triggered press: {:?}", self, PressType::LongPress);
                            let _ = sender.send(PressType::LongPress).await;
                        } else {
                            println!("Button {} triggered press: {:?}", self, PressType::ShortPress);
                            let _ = sender.send(PressType::ShortPress).await;
                        }
                        pressed_time = None;
                    }
                }
            }
        }
    }
}

impl Display for PowerButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.device().physical_path().map(|path| write!(f, "{}", path)).unwrap_or_else(|| write!(f, "Unknown"))
    }
}