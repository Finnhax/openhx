use clap::{Parser, Subcommand};
use openhx_core::device::KnownDevice;
use openhx_i18n::fl;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = fl!("cli-preset-category-about"))]
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },

    #[command(about = fl!("cli-setlist-category-about"))]
    Setlist {
        #[command(subcommand)]
        action: SetlistAction,
    },
}

#[derive(Subcommand)]
pub enum SetlistAction {
    #[command(
        about = fl!("cli-list-setlists-about"),
        long_about = fl!("cli-list-setlists-long")
    )]
    List {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,
    },
}

#[derive(Subcommand)]
pub enum PresetAction {
    #[command(
        about = fl!("cli-list-presets-about"),
        long_about = fl!("cli-list-presets-long")
    )]
    List {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[arg(
            short,
            long,
            default_value_t = 0,
            value_name = "SETLIST",
            help = fl!("cli-setlist-help")
        )]
        setlist: u8,
    },

    #[command(
        about = fl!("cli-select-preset-about"),
        long_about = fl!("cli-select-preset-long")
    )]
    Select {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[arg(
            short,
            long,
            alias = "bank",
            default_value_t = 0,
            value_name = "SETLIST",
            help = fl!("cli-setlist-help")
        )]
        setlist: u8,

        #[arg(short, long)]
        preset: u8,
    },

    #[command(
        about = fl!("cli-rename-preset-about"),
        long_about = fl!("cli-rename-preset-long")
    )]
    Rename {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[arg(
            short,
            long,
            default_value_t = 0,
            value_name = "SETLIST",
            help = fl!("cli-setlist-help")
        )]
        setlist: u8,

        #[arg(short, long)]
        preset: u8,

        #[arg(short, long, value_name = "NAME", help = fl!("cli-name-help"))]
        name: String,
    },

    #[command(
        about = fl!("cli-preset-active-about"),
        long_about = fl!("cli-preset-active-long")
    )]
    Active {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,
    },

    #[command(
        about = fl!("cli-preset-blocks-about"),
        long_about = fl!("cli-preset-blocks-long")
    )]
    Blocks {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,
    },

    #[command(
        about = fl!("cli-set-param-about"),
        long_about = fl!("cli-set-param-long")
    )]
    SetParam {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[command(flatten)]
        block: BlockArg,

        #[arg(long, help = fl!("cli-cab-help"))]
        cab: bool,

        #[arg(long, value_name = "PARAM", help = fl!("cli-param-help"))]
        param: String,

        #[arg(long, value_name = "VALUE", allow_hyphen_values = true, help = fl!("cli-value-help"))]
        value: String,
    },

    #[command(
        about = fl!("cli-block-state-about"),
        long_about = fl!("cli-block-state-long")
    )]
    BlockState {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[command(flatten)]
        block: BlockArg,

        #[arg(long, value_enum, help = fl!("cli-state-help"))]
        state: OnOff,
    },

    #[command(
        about = fl!("cli-set-model-about"),
        long_about = fl!("cli-set-model-long")
    )]
    SetModel {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[command(flatten)]
        block: BlockArg,

        #[command(flatten)]
        model: ModelArg,
    },

    #[command(
        about = fl!("cli-add-block-about"),
        long_about = fl!("cli-add-block-long")
    )]
    AddBlock {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[command(flatten)]
        block: BlockArg,

        #[command(flatten)]
        model: ModelArg,
    },

    #[command(
        about = fl!("cli-remove-block-about"),
        long_about = fl!("cli-remove-block-long")
    )]
    RemoveBlock {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,

        #[command(flatten)]
        block: BlockArg,
    },

    #[command(
        about = fl!("cli-save-preset-about"),
        long_about = fl!("cli-save-preset-long")
    )]
    Save {
        #[arg(long, value_name = "DEVICE", help = fl!("cli-device-help"))]
        device: Option<DeviceArg>,
    },
}

/// A block of the active preset, as `preset blocks` numbers it (`P1.3` is
/// path 1, position 3).
#[derive(clap::Args, Clone, Copy)]
pub struct BlockArg {
    #[arg(long, value_name = "PATH", value_parser = clap::value_parser!(u8).range(1..=2), help = fl!("cli-path-help"))]
    pub path: u8,

    #[arg(long, value_name = "POSITION", value_parser = clap::value_parser!(u8).range(1..=8), help = fl!("cli-position-help"))]
    pub position: u8,
}

impl BlockArg {
    /// The 0-based `(path, position)` the core API uses.
    pub fn index(self) -> (u8, u8) {
        (self.path - 1, self.position - 1)
    }
}

/// The model of a block, by name or ID, with the cab of an amp+cab block.
#[derive(clap::Args, Clone)]
pub struct ModelArg {
    #[arg(long, value_name = "MODEL", help = fl!("cli-model-help"))]
    pub model: String,

    #[arg(long = "with-cab", value_name = "CAB", help = fl!("cli-with-cab-help"))]
    pub cab: Option<String>,
}

/// A block's state.
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

/// CLI-friendly wrapper around [`KnownDevice`] that implements [`clap::ValueEnum`].
#[derive(Clone, clap::ValueEnum)]
pub enum DeviceArg {
    HxStomp,
    HxStompXl,
    HelixFloor,
}

impl From<DeviceArg> for KnownDevice {
    fn from(arg: DeviceArg) -> Self {
        match arg {
            DeviceArg::HxStomp => KnownDevice::HxStomp,
            DeviceArg::HxStompXl => KnownDevice::HxStompXl,
            DeviceArg::HelixFloor => KnownDevice::HelixFloor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_structure() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_device_arg_conversion() {
        let dev: KnownDevice = DeviceArg::HxStomp.into();
        assert_eq!(dev, KnownDevice::HxStomp);

        let dev_xl: KnownDevice = DeviceArg::HxStompXl.into();
        assert_eq!(dev_xl, KnownDevice::HxStompXl);

        let dev_helix: KnownDevice = DeviceArg::HelixFloor.into();
        assert_eq!(dev_helix, KnownDevice::HelixFloor);
    }
}
