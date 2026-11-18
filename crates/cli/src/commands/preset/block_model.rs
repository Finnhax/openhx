//! `preset set-model`, `preset add-block` and `preset remove-block`.

use crate::{
    args::{BlockArg, DeviceArg, ModelArg},
    commands::connect,
};
use openhx_core::{
    Block, BlockCategory, BlockKind, DeviceClient, Model,
    blocks::{find_models, model_info},
    error::HxError,
};
use openhx_i18n::fl;

/// Executes `preset set-model`: replaces the model of an existing block.
pub fn set_model(
    device: Option<DeviceArg>,
    block: BlockArg,
    model: &ModelArg,
) -> Result<(), HxError> {
    let client = connect(device)?;
    let (path, position) = block.index();

    let current = existing_block(client.as_ref(), block)?;
    let mut kind = block_kind(model, current.category())?;
    // A new amp keeps the cab of an amp+cab block unless another is given.
    if let (BlockKind::AmpCab { cab, .. }, BlockKind::Model(amp), None) =
        (&current.kind, &kind, &model.cab)
        && model_info(*amp).is_some_and(|m| m.category == BlockCategory::Amp)
    {
        kind = BlockKind::AmpCab {
            amp: *amp,
            cab: *cab,
        };
    }

    let new = client.set_block_model(path, position, &kind)?;
    println!(
        "{}",
        fl!(
            "cli-model-set",
            path = block.path,
            position = block.position,
            model = new.name()
        )
    );
    println!("{}", fl!("cli-unsaved-hint"));
    Ok(())
}

/// Executes `preset add-block`: adds a block at an empty position.
pub fn add_block(
    device: Option<DeviceArg>,
    block: BlockArg,
    model: &ModelArg,
) -> Result<(), HxError> {
    let client = connect(device)?;
    let (path, position) = block.index();

    if let Some(existing) = client.read_active_preset_blocks()?.block_at(path, position) {
        return Err(HxError::Protocol(fl!(
            "cli-position-taken",
            path = block.path,
            position = block.position,
            block = existing.name()
        )));
    }

    let new = client.add_block(path, position, &block_kind(model, None)?)?;
    println!(
        "{}",
        fl!(
            "cli-block-added",
            path = block.path,
            position = block.position,
            model = new.name()
        )
    );
    println!("{}", fl!("cli-unsaved-hint"));
    Ok(())
}

/// Executes `preset remove-block`.
pub fn remove_block(device: Option<DeviceArg>, block: BlockArg) -> Result<(), HxError> {
    let client = connect(device)?;
    let (path, position) = block.index();

    let removed = existing_block(client.as_ref(), block)?;
    client.remove_block(path, position)?;
    println!(
        "{}",
        fl!(
            "cli-block-removed",
            path = block.path,
            position = block.position,
            model = removed.name()
        )
    );
    println!("{}", fl!("cli-unsaved-hint"));
    Ok(())
}

/// The block at `block`, or an error when the position is empty.
fn existing_block(client: &(dyn DeviceClient + Send), block: BlockArg) -> Result<Block, HxError> {
    let (path, position) = block.index();
    client
        .read_active_preset_blocks()?
        .block_at(path, position)
        .cloned()
        .ok_or_else(|| {
            HxError::Protocol(fl!(
                "cli-block-not-found",
                path = block.path,
                position = block.position
            ))
        })
}

/// The block kind `model` asks for: a single model, or an amp with a cab.
///
/// A name shared by models of several categories (an amp and its preamp)
/// resolves to the one in `prefer`, the category of the block being
/// changed.
fn block_kind(model: &ModelArg, prefer: Option<BlockCategory>) -> Result<BlockKind, HxError> {
    let Some(cab) = &model.cab else {
        return Ok(BlockKind::Model(resolve_model(&model.model, prefer)?.id));
    };

    let main = resolve_model(&model.model, Some(BlockCategory::Amp))?;
    let cab = resolve_model(cab, Some(BlockCategory::Cab))?;
    if main.category != BlockCategory::Amp || cab.category != BlockCategory::Cab {
        return Err(HxError::Protocol(fl!("cli-amp-cab-mismatch")));
    }
    Ok(BlockKind::AmpCab {
        amp: main.id,
        cab: cab.id,
    })
}

/// Finds one model by its ID or name, preferring `prefer` when the name
/// matches models of several categories.
fn resolve_model(query: &str, prefer: Option<BlockCategory>) -> Result<&'static Model, HxError> {
    if let Ok(id) = query.trim().parse::<u16>() {
        return model_info(id).ok_or_else(|| {
            HxError::Protocol(fl!("cli-model-not-found", model = query.to_owned()))
        });
    }

    let mut found = find_models(query);
    // An amp+cab block takes its amp from the amps.
    let prefer = prefer.map(|c| match c {
        BlockCategory::AmpCab => BlockCategory::Amp,
        other => other,
    });
    if found.len() > 1
        && let Some(prefer) = prefer
        && found.iter().any(|m| m.category == prefer)
    {
        found.retain(|m| m.category == prefer);
    }

    match found.as_slice() {
        [model] => Ok(model),
        [] => Err(HxError::Protocol(fl!(
            "cli-model-not-found",
            model = query.to_owned()
        ))),
        candidates => {
            const SHOWN: usize = 8;
            let mut names: Vec<_> = candidates
                .iter()
                .take(SHOWN)
                .map(|m| format!("{} [{}, ID {}]", m.name, m.category, m.id))
                .collect();
            if candidates.len() > SHOWN {
                names.push("…".to_owned());
            }
            Err(HxError::Protocol(fl!(
                "cli-model-ambiguous",
                model = query.to_owned(),
                candidates = names.join(", ")
            )))
        }
    }
}
