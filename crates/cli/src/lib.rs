pub mod args;
pub mod commands;

use args::{Cli, Commands, PresetAction, SetlistAction};
use clap::Parser;
use openhx_core::error::HxError;

pub fn run() -> Result<(), HxError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setlist { action } => match action {
            SetlistAction::List { device } => commands::setlist::list::execute(device),
        },
        Commands::Preset { action } => match action {
            PresetAction::List { device, setlist } => {
                commands::preset::list::execute(device, setlist)
            }
            PresetAction::Select {
                device,
                setlist,
                preset,
            } => commands::preset::select::execute(device, setlist, preset),
            PresetAction::Rename {
                device,
                setlist,
                preset,
                name,
            } => commands::preset::rename::execute(device, setlist, preset, &name),
            PresetAction::Blocks { device } => commands::preset::blocks::execute(device),
            PresetAction::Active { device } => commands::preset::active::execute(device),
            PresetAction::SetParam {
                device,
                block,
                cab,
                param,
                value,
            } => commands::preset::set_param::execute(device, block, cab, &param, &value),
            PresetAction::BlockState {
                device,
                block,
                state,
            } => commands::preset::block_state::execute(device, block, state),
            PresetAction::Save { device } => commands::preset::save::execute(device),
        },
    }
}
