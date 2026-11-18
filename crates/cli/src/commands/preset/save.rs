use crate::{args::DeviceArg, commands::connect};
use openhx_core::error::HxError;
use openhx_i18n::fl;

/// Executes the `preset save` CLI command: stores the active preset, with
/// its edits, in its own slot under its current name.
pub fn execute(device: Option<DeviceArg>) -> Result<(), HxError> {
    let client = connect(device)?;

    let active = client.read_active_preset()?;
    client.save_preset(active.setlist, active.preset, &active.name)?;

    println!(
        "{}",
        fl!(
            "cli-preset-saved",
            name = active.name,
            setlist = active.setlist,
            preset = format!("{:03}", active.preset)
        )
    );

    Ok(())
}
