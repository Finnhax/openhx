use crate::{
    args::{BlockArg, DeviceArg, OnOff},
    commands::connect,
};
use openhx_core::error::HxError;
use openhx_i18n::fl;

/// Executes the `preset block-state` CLI command.
pub fn execute(device: Option<DeviceArg>, block: BlockArg, state: OnOff) -> Result<(), HxError> {
    let client = connect(device)?;

    let (path, position) = block.index();
    // The device accepts the command for an empty position too, without
    // effect; refuse it instead.
    if client
        .read_active_preset_blocks()?
        .block_at(path, position)
        .is_none()
    {
        return Err(HxError::Protocol(fl!(
            "cli-block-not-found",
            path = block.path,
            position = block.position
        )));
    }

    let enabled = matches!(state, OnOff::On);
    client.set_block_enabled(path, position, enabled)?;

    let state = if enabled {
        fl!("block-on")
    } else {
        fl!("block-off")
    };
    println!(
        "{}",
        fl!(
            "cli-block-state-set",
            path = block.path,
            position = block.position,
            state = state
        )
    );
    println!("{}", fl!("cli-unsaved-hint"));

    Ok(())
}
