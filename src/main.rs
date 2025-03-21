use powerbutton::PowerButton;
use steam::PressType;

mod steam;
mod powerbutton;

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .expect("Failed to build runtime");
        
    let channel = async_channel::bounded::<PressType>(1);
    let buttons = runtime.block_on(async { PowerButton::auto_detect().expect("Failed to auto detect power buttons") });
    if buttons.is_empty() {
        eprintln!("No power buttons detected");
        std::process::exit(1);
    }
    
    for mut button in buttons {
        let chan = channel.0.clone();
        runtime.spawn(async move {
            println!("Detected power button: {}", button);
            button.listen(chan).await.expect("Failed to listen to power button");
        });
    }
    runtime.block_on(async {
        let steam = steam::SteamInstance::fetch().expect("Failed to fetch steam instance");
        steam.listen(channel.1).await.expect("Failed to listen to steam instance");
    })
}
