pub use firmware_updater::FirmwareUpdater;
use silabs::MGM210P22A;

mod firmware_updater;
mod silabs;
mod silabs2;

fn main() {
    let zigbee_chip = MGM210P22A::new("/dev/ttymxc3".into());

    match zigbee_chip.current_version() {
        Ok(version) => {
            println!("Current version: {version}");
        }
        Err(error) => {
            eprintln!("Error retrieving version: {error}");
        }
    }

    println!("Available versions: {:?}", zigbee_chip.available_versions());

    if let Some(latest_version) = zigbee_chip.latest_version() {
        println!("Latest version: {latest_version}");
    } else {
        eprintln!("No latest version available.");
    }
}
