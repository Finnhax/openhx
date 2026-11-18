use crate::{
    args::{BlockArg, DeviceArg},
    commands::connect,
};
use openhx_core::{ParamAddress, ParamDef, error::HxError};
use openhx_i18n::fl;

/// Executes the `preset set-param` CLI command.
pub fn execute(
    device: Option<DeviceArg>,
    block_arg: BlockArg,
    cab: bool,
    param: &str,
    value: &str,
) -> Result<(), HxError> {
    let client = connect(device)?;

    let (path, position) = block_arg.index();
    let chain = client.read_active_preset_blocks()?;
    let block = chain.block_at(path, position).ok_or_else(|| {
        HxError::Protocol(fl!(
            "cli-block-not-found",
            path = block_arg.path,
            position = block_arg.position
        ))
    })?;

    let defs = if cab {
        block.cab_param_defs()
    } else {
        block.param_defs()
    }
    .ok_or_else(|| HxError::Protocol(fl!("cli-params-unknown", block = block.name())))?;

    let (index, def) = find_param(defs, param).ok_or_else(|| {
        HxError::Protocol(fl!(
            "cli-param-not-found",
            param = param.to_owned(),
            block = block.name()
        ))
    })?;
    let new_value = def.parse(value)?;

    let address = ParamAddress {
        path,
        position,
        cab,
        index,
    };
    client.set_block_param(address, &new_value)?;

    println!(
        "{}",
        fl!(
            "cli-param-set",
            param = def.name,
            value = def.format(&new_value)
        )
    );
    println!("{}", fl!("cli-unsaved-hint"));

    Ok(())
}

/// Finds a parameter by its name or ID (case-insensitive, spaces ignored),
/// or by its 1-based number.
fn find_param<'a>(defs: &'a [ParamDef], param: &str) -> Option<(u8, &'a ParamDef)> {
    let normalize = |s: &str| s.replace(' ', "").to_lowercase();
    let wanted = normalize(param);

    let position = match param.parse::<usize>() {
        Ok(number) => number.checked_sub(1).filter(|&i| i < defs.len()),
        Err(_) => defs
            .iter()
            .position(|d| normalize(d.name) == wanted || normalize(d.id) == wanted),
    }?;
    Some((u8::try_from(position).ok()?, &defs[position]))
}
