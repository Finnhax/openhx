use openhx_core::{
    ActivePreset, Block, BlockCategory, ParamAddress, ParamValue, Preset, PresetChain, Setlist,
};

#[derive(Debug, Clone)]
pub enum Message {
    DeviceDetected {
        name: String,
        setlists: Vec<Setlist>,
        presets: Vec<Preset>,
    },
    DeviceDisconnected,
    ConnectionError(String),
    /// The user picked another setlist in the sidebar.
    SetlistChosen(u8),
    /// A setlist's presets finished loading.
    SetlistLoaded(u8, Vec<Preset>),
    /// Loading a setlist failed; the previous list stays on screen.
    SetlistLoadFailed(u8, String),
    PresetSelected(u8),
    /// The user edited the name field of the selected preset.
    RenameInputChanged(String),
    /// The user asked to store the edited name on the device.
    RenameSubmitted,
    /// The device stored a new preset name.
    PresetRenamed {
        setlist: u8,
        preset: u8,
        name: String,
    },
    /// Storing a new preset name failed.
    RenameFailed(String),
    /// The user asked to re-read the active preset's blocks.
    BlocksRequested,
    /// The device reported its active preset and that preset's chain.
    ActivePresetLoaded {
        active: ActivePreset,
        chain: PresetChain,
    },
    /// Reading the active preset's blocks failed.
    BlocksLoadFailed(String),
    /// The user picked the block at `(path, position)` in the signal chain.
    BlockSelected(u8, u8),
    /// The user picked a device type to browse its models.
    DeviceTypeChosen(BlockCategory),
    /// The user moved a parameter's control. A slider sends its value once
    /// it is released; a switch (`commit`) at once.
    ParamChanged {
        address: ParamAddress,
        value: ParamValue,
        commit: bool,
    },
    /// The user released a parameter's slider.
    ParamReleased,
    /// The user switched the block at `(path, position)` on or off.
    BlockEnabledToggled(u8, u8, bool),
    /// The device applied an edit, or refused it.
    EditWritten(Result<(), String>),
    /// The user picked a model in the module list: the selected block gets
    /// it, or an empty selected position gets a new block with it.
    ModelChosen(u16),
    /// The user asked to remove the selected block.
    RemoveBlockRequested,
    /// The device created or changed a block, or refused to.
    BlockReplaced(Result<Block, String>),
    /// The device removed the block at `(path, position)`, or refused to.
    BlockRemoved(u8, u8, Result<(), String>),
    /// The user asked to save the active preset with its edits.
    SaveRequested,
    /// The device saved the active preset, or refused to.
    PresetSaved(Result<(), String>),
}
