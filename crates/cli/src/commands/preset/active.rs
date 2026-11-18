use crate::{args::DeviceArg, commands::connect};
use openhx_core::error::HxError;
use openhx_i18n::fl;

/// Executes the `preset active` CLI command.
pub fn execute(device: Option<DeviceArg>) -> Result<(), HxError> {
    let client = connect(device)?;

    let active = client.read_active_preset()?;

    println!(
        "{}",
        fl!(
            "cli-active-preset",
            setlist = active.setlist,
            preset = format!("{:03}", active.preset),
            name = active.name
        )
    );

    Ok(())
}
