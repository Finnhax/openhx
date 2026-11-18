use crate::{message::Message, state::AppState, update, view, worker};
use iced::{Subscription, Task};
use openhx_core::{
    ActivePreset, Block, BlockCategory, BlockKind, ParamAddress, ParamValue, Preset, PresetChain,
    Setlist,
};

pub struct App {
    pub state: AppState,
    pub device_name: String,
    /// The connected device's setlists, named as on the device.
    pub setlists: Vec<Setlist>,
    /// Setlist whose presets are currently shown.
    pub setlist: u8,
    pub presets: Vec<Preset>,
    pub error_log: Option<String>,
    pub selected_preset: Option<u8>,
    /// Name being edited for the selected preset.
    pub rename_input: String,
    /// `true` while a rename is in flight; blocks a second submit.
    pub rename_pending: bool,
    /// Why the last rename failed, shown under the name field.
    pub rename_error: Option<String>,
    /// The active preset's blocks and the block view's state.
    pub blocks: BlocksState,
}

/// The active preset's signal chain as shown in the block view.
#[derive(Debug, Default)]
pub struct BlocksState {
    /// The device's active preset.
    pub chain: PresetChain,
    /// `true` while the active preset's blocks are being read.
    pub loading: bool,
    /// Why reading the blocks failed, if it did.
    pub error: Option<String>,
    /// Block whose settings are shown, as `(path, position)`.
    pub selected: Option<(u8, u8)>,
    /// Device type picked in the type list; `None` follows the selected
    /// block's own type.
    pub browse_category: Option<BlockCategory>,
    /// The preset the chain belongs to; saving stores it here.
    pub active: Option<ActivePreset>,
    /// A slider value not sent to the device yet, waiting for its release.
    pub pending: Option<(ParamAddress, ParamValue)>,
    /// `true` once the chain was changed since it was read or saved.
    pub edited: bool,
    /// `true` while the active preset is being saved.
    pub saving: bool,
    /// Why the last edit or save failed, if it did.
    pub edit_error: Option<String>,
}

impl BlocksState {
    /// The block whose settings are shown, if it still exists.
    pub fn selected_block(&self) -> Option<&Block> {
        let (path, position) = self.selected?;
        self.chain.block_at(path, position)
    }

    /// The block at `position` of `path`, to change it.
    pub fn block_mut(&mut self, path: u8, position: u8) -> Option<&mut Block> {
        self.chain
            .blocks
            .iter_mut()
            .find(|b| b.path == path && b.position == position)
    }

    /// The device type whose models the module list shows: the one picked
    /// in the type list, else `block`'s own.
    pub fn browsed_category(&self, block: Option<&Block>) -> Option<BlockCategory> {
        self.browse_category
            .or_else(|| block.and_then(Block::category))
    }

    /// The block a click on `model` in the module list asks for, given the
    /// selected `block` (`None` for an empty position).
    ///
    /// Browsing amp+cab replaces the amp of an amp+cab block and browsing
    /// cabs its cab; any other pick replaces the whole block. `None` when an
    /// amp+cab block would need a cab that isn't known.
    pub fn chosen_kind(&self, block: Option<&Block>, model: u16) -> Option<BlockKind> {
        let current = block.map(|b| &b.kind);
        match (self.browsed_category(block), current) {
            (Some(BlockCategory::AmpCab), Some(BlockKind::AmpCab { cab, .. })) => {
                Some(BlockKind::AmpCab {
                    amp: model,
                    cab: *cab,
                })
            }
            (Some(BlockCategory::AmpCab), _) => None,
            (Some(BlockCategory::Cab), Some(BlockKind::AmpCab { amp, .. })) => {
                Some(BlockKind::AmpCab {
                    amp: *amp,
                    cab: model,
                })
            }
            _ => Some(BlockKind::Model(model)),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Waiting,
            device_name: String::new(),
            setlists: Vec::new(),
            setlist: 0,
            presets: Vec::new(),
            error_log: None,
            selected_preset: None,
            rename_input: String::new(),
            rename_pending: false,
            rename_error: None,
            blocks: BlocksState::default(),
        }
    }
}

impl App {
    /// Clears the selection together with any rename in progress.
    pub fn clear_selection(&mut self) {
        self.selected_preset = None;
        self.rename_input.clear();
        self.rename_pending = false;
        self.rename_error = None;
    }

    /// Selects preset slot `index` of the shown setlist and offers its name
    /// for renaming.
    pub fn select_preset(&mut self, index: u8) {
        self.clear_selection();
        self.selected_preset = Some(index);
        if let Some(preset) = self.presets.iter().find(|p| p.index == index) {
            self.rename_input = preset.name.clone();
        }
    }

    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::handle_message(self, message)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            AppState::Waiting => Subscription::run(worker::usb_poll),
            AppState::Connected => Subscription::run(worker::usb_check_disconnect),
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        view::view(self)
    }
}
