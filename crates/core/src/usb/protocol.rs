use crate::blocks::BlockKind;

pub const EP_OUT: u8 = 0x01;
pub const EP_IN: u8 = 0x81;
pub const INTERFACE: u8 = 0;

pub const TIMEOUT_MS: u64 = 2_000;
pub const DRAIN_TIMEOUT_MS: u64 = 50;

pub const OFFSET_STEP: u32 = 0x0100;
pub const STREAM_INITIAL_OFFSET: u32 = 0x0000_1138;
pub const PAGINATION_INITIAL_SEQ: u8 = 0x08;

/// Size of the IN buffer used while streaming presets.
///
/// Stream chunks have been seen at 272, 468 and 512 bytes; the buffer leaves
/// headroom so a larger chunk is not cut off by the read.
pub const STREAM_BUFFER_SIZE: usize = 2_048;

/// Upper bound on pagination requests for one preset stream.
///
/// A full 128-preset stream needs well under 32 chunks even at the smallest
/// observed chunk size; the cap only stops a device that never completes it.
pub const MAX_STREAM_REQUESTS: usize = 128;

/// Number of leading header bytes to skip before the MessagePack payload.
pub const RESPONSE_HEADER_SIZE: usize = 16;

/// Offset of the little-endian `u32` payload length within a response's inner
/// header.
pub const RESPONSE_LEN_OFFSET: usize = 20;

/// Offset of the MessagePack payload in the first packet of a response.
///
/// A response carries the 16-byte transport header followed by an 8-byte inner
/// header whose last four bytes are the payload length.
pub const RESPONSE_PAYLOAD_OFFSET: usize = 24;

// Common header layout for Line 6 proprietary USB packets:
//   [0..4]   – framing / length prefix (low byte at [0])
//   [4..8]   – protocol routing field; host→device emits `01 10 EF 03`,
//              device→host echoes it byte-pair-swapped as `EF 03 01 10`
//   [8..10]  – sequence number, big-endian u16. The high byte at [8] is
//              always 0 in observed traffic, so [9] is the effective u8
//              seq (see `SEQ_BYTE_OFFSET` in `client.rs`).
//   [10..12] – command, big-endian u16. [10] is always 0; [11] is the
//              effective cmd byte (e.g. 0x04 = open, 0x08 = chunk req).
//   [12..]   – payload

/// Channel handshake (`magic = 0x0028`).
/// Re-synchronises the device's session state.
pub const HANDSHAKE: &[u8] = &[
    0x0C, 0x00, 0x00, 0x28, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x21,
    0x00, 0x10, 0x00, 0x00,
];

/// Open first session resource (`cmd = 0x04`, `seq = 0x02`).
pub const SESSION_OPEN_1: &[u8] = &[
    0x11, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x02, 0x00, 0x04, 0x00, 0x10, 0x00, 0x00,
    0x01, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
];

/// First session chunk request (`cmd = 0x08`, `seq = 0x03`).
pub const SESSION_CHUNK_1: &[u8] = &[
    0x08, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x03, 0x00, 0x08, 0x09, 0x10, 0x00, 0x00,
];

/// Open second session resource (`cmd = 0x04`, `seq = 0x04`).
pub const SESSION_OPEN_2: &[u8] = &[
    0x1A, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x04, 0x00, 0x04, 0x09, 0x10, 0x00, 0x00,
    0x01, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x83, 0x66, 0xCD, 0x03, 0xE8, 0x64, 0xCC, 0xFE,
    0x65, 0x80, 0x00, 0x00,
];

/// Second session chunk request (`cmd = 0x08`, `seq = 0x05`).
pub const SESSION_CHUNK_2: &[u8] = &[
    0x08, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x05, 0x00, 0x08, 0x1A, 0x10, 0x00, 0x00,
];

pub const SESSION_INIT_SEQUENCE: &[&[u8]] = &[
    HANDSHAKE,
    SESSION_OPEN_1,
    SESSION_CHUNK_1,
    SESSION_OPEN_2,
    SESSION_CHUNK_2,
];

/// Open the presets resource (`cmd = 0x04`, `seq = 0x06`).
pub const OPEN_PRESETS: &[u8] = &[
    0x19, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x06, 0x00, 0x04, 0x1A, 0x10, 0x00, 0x00,
    0x01, 0x00, 0x02, 0x00, 0x09, 0x00, 0x00, 0x00, 0x83, 0x66, 0xCD, 0x03, 0xE9, 0x64, 0x00, 0x65,
    0xC0, 0x00, 0x00, 0x00,
];

/// Sequence number of the start-paged-stream request in a fresh session.
pub const OPEN_STREAM_SEQ: u8 = 0x07;

/// Builds the start-paged-stream request (`cmd = 0x0C`) for one setlist.
///
/// The payload is a MsgPack structure:
/// `{102: 1002, 100: 1, 101: {107: setlist, 101: 2}}`
///
/// Key 107 selects which setlist the device streams. Single-setlist devices
/// always use `0`, which reproduces the packet captured from an HX Stomp XL.
/// `setlist` must be `< 128` so it encodes as a positive fixint.
#[inline]
pub fn build_open_stream_request(seq: u8, setlist: u8) -> [u8; 40] {
    debug_assert!(setlist < 0x80, "setlist must encode as a positive fixint");
    [
        0x1D, 0x00, 0x00, 0x18, // Magic header with length for 40-byte packet
        0x01, 0x10, 0xEF, 0x03, // Standard Line 6 routing
        0x00, seq, 0x00, 0x0C, // Sequence and Command (0x0C = start paged stream)
        // Inner resource header
        0x38, 0x10, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x0D, 0x00, 0x00,
        0x00, // MsgPack payload length (13 bytes)
        // MsgPack Payload
        0x83, // fixmap(3 items)
        0x66, 0xCD, 0x03, 0xEA, // Key 102 (Transaction ID): 1002
        0x64, 0x01, // Key 100 (Command Type): 1 (Start stream)
        0x65, 0x82, // Key 101 (Args): map(2 items)
        0x6B, setlist, // Key 107 (Setlist Index): setlist
        0x65, 0x02, // Key 101: 2
        // Padding to align to 4-byte boundary
        0x00, 0x00, 0x00,
    ]
}

/// Routing field of host→device packets on the preset channel.
pub const PRESET_ROUTING_OUT: [u8; 4] = [0x01, 0x10, 0xEF, 0x03];

/// Builds a pagination request on the channel with the given host→device
/// `routing` ([`PRESET_ROUTING_OUT`] or [`EDIT_ROUTING_OUT`]) for the given
/// sequence number and stream offset.
#[inline]
pub fn build_pagination_request(routing: [u8; 4], seq: u8, offset: u32) -> [u8; 16] {
    let [r0, r1, r2, r3] = routing;
    let [o0, o1, o2, o3] = offset.to_le_bytes();
    [
        0x08, 0x00, 0x00, 0x18, r0, r1, r2, r3, 0x00, seq, 0x00, 0x08, o0, o1, o2, o3,
    ]
}

/// Builds a preset selection request (`cmd = 0x04`, `seq = 0x06`).
///
/// The payload is a MsgPack structure:
/// `{102: 1010, 100: 20, 101: {107: setlist, 108: preset}}`
///
/// `preset` is the slot within `setlist` (`0..preset_count`). Both values
/// must be `< 128` so they encode as positive fixints.
#[inline]
pub fn build_select_preset_request(seq: u8, setlist: u8, preset: u8) -> [u8; 40] {
    debug_assert!(
        setlist < 0x80 && preset < 0x80,
        "args must encode as positive fixints"
    );
    [
        0x1D, 0x00, 0x00, 0x18, // Magic header with length for 40-byte packet
        0x01, 0x10, 0xEF, 0x03, // Standard Line 6 routing
        0x00, seq, 0x00, 0x04, // Sequence and Command (0x04)
        // Inner resource header
        0x1A, 0x10, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x0D, 0x00, 0x00,
        0x00, // MsgPack payload length (13 bytes)
        // MsgPack Payload
        0x83, // fixmap(3 items)
        0x66, 0xCD, 0x03, 0xF2, // Key 102 (Transaction ID): 1010
        0x64, 0x14, // Key 100 (Command Type): 20 (Select)
        0x65, 0x82, // Key 101 (Args): map(2 items)
        0x6B, setlist, // Key 107 (Setlist Index): setlist
        0x6C, preset, // Key 108 (Preset Index): preset
        // Padding to align to 4-byte boundary
        0x00, 0x00, 0x00,
    ]
}

// Edit channel
//
// The device exposes a second channel next to the preset channel above:
// routing `80 10 ED 03` (host→device) / `ED 03 80 10` (device→host), resource
// 6. It serves the active preset's full contents — its blocks and their
// parameters — as one length-delimited stream, paginated like the preset
// list.

/// Routing field of host→device packets on the edit channel.
pub const EDIT_ROUTING_OUT: [u8; 4] = [0x80, 0x10, 0xED, 0x03];

/// Routing field the device puts on edit-channel replies.
pub const EDIT_ROUTING_IN: [u8; 4] = [0xED, 0x03, 0x80, 0x10];

/// Edit-channel handshake (`magic = 0x0028`, `seq = 0x00`).
pub const EDIT_HANDSHAKE: &[u8] = &[
    0x0C, 0x00, 0x00, 0x28, 0x80, 0x10, 0xED, 0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x21,
    0x00, 0x10, 0x00, 0x00,
];

/// Opens resource 6 on the edit channel (`cmd = 0x04`, `seq = 0x02`).
pub const EDIT_SESSION_OPEN: &[u8] = &[
    0x11, 0x00, 0x00, 0x18, 0x80, 0x10, 0xED, 0x03, 0x00, 0x02, 0x00, 0x04, 0x00, 0x10, 0x00, 0x00,
    0x01, 0x00, 0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
];

/// Sequence number of the read-active-preset request in a fresh edit session.
pub const READ_ACTIVE_PRESET_SEQ: u8 = 0x03;

/// Builds an edit-channel request without arguments, the first after opening
/// the channel (`seq` [`READ_ACTIVE_PRESET_SEQ`]).
///
/// The payload is a MsgPack structure: `{102: txn, 100: command, 101: nil}`.
const fn edit_request(cmd: u8, txn: u16, command: u8) -> [u8; 36] {
    let [t0, t1] = txn.to_be_bytes();
    let [r0, r1, r2, r3] = EDIT_ROUTING_OUT;
    [
        0x19,
        0x00,
        0x00,
        0x18, // Magic header with length for 36-byte packet
        r0,
        r1,
        r2,
        r3, // Edit-channel routing
        0x00,
        READ_ACTIVE_PRESET_SEQ,
        0x00,
        cmd, // Sequence and Command
        // Inner resource header
        0x09,
        0x10,
        0x00,
        0x00,
        0x01,
        0x00,
        0x06,
        0x00,
        0x09,
        0x00,
        0x00,
        0x00, // MsgPack payload length (9 bytes)
        // MsgPack Payload
        0x83, // fixmap(3 items)
        0x66,
        0xCD,
        t0,
        t1, // Key 102 (Transaction ID): txn
        0x64,
        command, // Key 100 (Command Type)
        0x65,
        0xC0, // Key 101 (Args): nil
        // Padding to align to 4-byte boundary
        0x00,
        0x00,
        0x00,
    ]
}

/// Starts streaming the active preset (`cmd = 0x0C`, `seq = 0x03`).
///
/// The payload is a MsgPack structure: `{102: 1012, 100: 22, 101: nil}`.
pub const READ_ACTIVE_PRESET: &[u8] = &edit_request(0x0C, 1012, 22);

/// Asks which preset is active (`cmd = 0x04`, `seq = 0x03`).
///
/// The payload is a MsgPack structure: `{102: 1028, 100: 23, 101: nil}`. The
/// reply's result is a map starting `{107: setlist, 108: preset, 109: name}`.
/// Like [`READ_ACTIVE_PRESET`], it is the first request after opening the
/// edit channel, so it reuses that sequence number.
pub const READ_ACTIVE_PRESET_INFO: &[u8] = &edit_request(0x04, 1028, 23);

/// Sequence number of the first edit-channel pagination request.
pub const EDIT_PAGINATION_INITIAL_SEQ: u8 = READ_ACTIVE_PRESET_SEQ + 1;

/// Status code (key `103`) a write command's ACK carries on success.
///
/// Captured from an HX Stomp for both preset selection and rename. Read
/// replies differ: the setlist reply reports `0` on success.
pub const ACK_STATUS_OK: u64 = 1;

/// Builds a preset rename request (`cmd = 0x04`).
///
/// The payload is a MsgPack structure:
/// `{102: 1005, 100: 6, 101: {107: setlist, 108: preset, 109: name}}`
///
/// The device stores names NUL-terminated, so the string carries a trailing
/// `\0`; this mirrors what [`crate::usb::parser`] strips when reading names
/// back. `setlist` and `preset` must be `< 128` so they encode as positive
/// fixints, and `name` must be ASCII and short enough to fit a fixstr
/// (callers validate with [`crate::models::validate_preset_name`]).
///
/// Unlike the fixed-size requests above, the packet length depends on the
/// name, so it is assembled into a `Vec` and zero-padded to a 4-byte
/// boundary like every captured request.
pub fn build_rename_preset_request(seq: u8, setlist: u8, preset: u8, name: &str) -> Vec<u8> {
    debug_assert!(
        setlist < 0x80 && preset < 0x80,
        "args must encode as positive fixints"
    );
    debug_assert!(
        name.is_ascii() && name.len() < 0x1F,
        "name must encode as a fixstr including its NUL terminator"
    );

    // fixstr length covers the name plus its NUL terminator.
    let str_len = name.len() as u8 + 1;

    let mut payload = vec![
        0x83, // fixmap(3 items)
        0x66,
        0xCD,
        0x03,
        0xED, // Key 102 (Transaction ID): 1005
        0x64,
        0x06, // Key 100 (Command Type): 6 (Rename)
        0x65,
        0x83, // Key 101 (Args): map(3 items)
        0x6B,
        setlist, // Key 107 (Setlist Index): setlist
        0x6C,
        preset, // Key 108 (Preset Index): preset
        0x6D,
        0xA0 | str_len, // Key 109 (Preset Name): fixstr
    ];
    payload.extend_from_slice(name.as_bytes());
    payload.push(0x00);

    let payload_len = payload.len() as u32;
    // The framing prefix counts every byte after the routing field: the
    // 16-byte seq/cmd + inner header plus the unpadded payload.
    let frame_len = 16 + payload_len;
    let [f0, f1, _, _] = frame_len.to_le_bytes();
    let [l0, l1, l2, l3] = payload_len.to_le_bytes();

    let mut packet = vec![
        f0, f1, 0x00, 0x18, // Magic header with frame length
        0x01, 0x10, 0xEF, 0x03, // Standard Line 6 routing
        0x00, seq, 0x00, 0x04, // Sequence and Command (0x04)
        // Inner resource header
        0x1A, 0x10, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, l0, l1, l2,
        l3, // MsgPack payload length
    ];
    packet.extend_from_slice(&payload);

    // Padding to align to 4-byte boundary
    packet.resize(packet.len().next_multiple_of(4), 0x00);
    packet
}

// Edit-channel writes
//
// Changing the active preset is done with single requests on the edit
// channel, each the first request after opening it (seq
// [`READ_ACTIVE_PRESET_SEQ`]). The device echoes the arguments (or `nil`)
// with status `0` once it has applied the change, and broadcasts it on its
// notification channel.

/// Transaction ID sent with edit-channel writes; the device echoes it.
pub const EDIT_WRITE_TXN: u16 = 1013;

/// Command type (key `100`) that sets one parameter of a block.
pub const CMD_SET_PARAM: u8 = 30;
/// Command type that switches a block on or off.
pub const CMD_SET_BLOCK_ENABLED: u8 = 41;
/// Command type that stores the active preset, with its edits, in a slot.
pub const CMD_SAVE_PRESET: u8 = 71;
/// Command type that removes a block.
pub const CMD_REMOVE_BLOCK: u8 = 28;
/// Command type that adds a block at an empty position.
pub const CMD_ADD_BLOCK: u8 = 39;
/// Command type that replaces a block's model.
pub const CMD_SET_BLOCK_MODEL: u8 = 40;

/// Argument keys of the edit-channel writes.
const ARG_SLOT: u8 = 98;
const ARG_MODEL: u8 = 26;
const ARG_PARAM: u8 = 28;
const ARG_PARAM_FLAG: u8 = 29;
const ARG_VALUE: u8 = 119;
const ARG_ENABLED: u8 = 59;
const ARG_SETLIST: u8 = 107;
const ARG_PRESET: u8 = 108;
const ARG_NAME: u8 = 109;
const ARG_NEW_BLOCK: u8 = 99;
const ARG_MODELS: u8 = 100;

/// Keys of a block's model map and of a new block's slot, as in the
/// preset data (see [`crate::usb::preset_data`]).
const KEY_AMP_CAB: u8 = 23;
const KEY_MODEL_A: u8 = 25;
const KEY_MODEL_B: u8 = 26;
const KEY_SLOT_TYPE: u8 = 19;
const KEY_SLOT_BODY: u8 = 20;
const KEY_MODELS: u8 = 24;
const KEY_BLOCK_TYPE: u8 = 9;
const KEY_ENABLED: u8 = 10;
const SLOT_TYPE_BLOCK: u8 = 6;

/// Builds an edit-channel request `{102: txn, 100: command, 101: args}`.
///
/// The packet length depends on the arguments, so it is assembled into a
/// `Vec` and zero-padded to a 4-byte boundary like every captured request.
pub fn build_edit_request(command: u8, args: Vec<(rmpv::Value, rmpv::Value)>) -> Vec<u8> {
    let message = rmpv::Value::Map(vec![
        (102.into(), EDIT_WRITE_TXN.into()),
        (100.into(), command.into()),
        (101.into(), rmpv::Value::Map(args)),
    ]);
    let mut payload = Vec::new();
    rmpv::encode::write_value(&mut payload, &message).expect("writing to a Vec cannot fail");

    let payload_len = payload.len() as u32;
    // The framing prefix counts every byte after the routing field: the
    // 16-byte seq/cmd + inner header plus the unpadded payload.
    let [f0, f1, _, _] = (16 + payload_len).to_le_bytes();
    let [r0, r1, r2, r3] = EDIT_ROUTING_OUT;
    let [l0, l1, l2, l3] = payload_len.to_le_bytes();

    let mut packet = vec![
        f0,
        f1,
        0x00,
        0x18, // Magic header with frame length
        r0,
        r1,
        r2,
        r3, // Edit-channel routing
        0x00,
        READ_ACTIVE_PRESET_SEQ,
        0x00,
        0x04, // Sequence and Command (0x04)
        // Inner resource header
        0x09,
        0x10,
        0x00,
        0x00,
        0x01,
        0x00,
        0x06,
        0x00,
        l0,
        l1,
        l2,
        l3, // MsgPack payload length
    ];
    packet.extend_from_slice(&payload);
    packet.resize(packet.len().next_multiple_of(4), 0x00);
    packet
}

/// Builds a request setting parameter `param` of the block in preset slot
/// `slot` (see [`crate::blocks::slot_index`]) to `value`.
///
/// `model` picks the block's model: `0` for a single model or the amp of an
/// amp+cab block, `1` for its cab.
pub fn build_set_param_request(slot: u8, model: u8, param: u8, value: rmpv::Value) -> Vec<u8> {
    build_edit_request(
        CMD_SET_PARAM,
        vec![
            (ARG_SLOT.into(), slot.into()),
            // Always `true` in captured traffic, for any block and model.
            (ARG_PARAM_FLAG.into(), true.into()),
            (ARG_MODEL.into(), model.into()),
            (ARG_PARAM.into(), param.into()),
            (ARG_VALUE.into(), value),
        ],
    )
}

/// Builds a request switching the block in preset slot `slot` on or off.
pub fn build_set_block_enabled_request(slot: u8, enabled: bool) -> Vec<u8> {
    build_edit_request(
        CMD_SET_BLOCK_ENABLED,
        vec![
            (ARG_SLOT.into(), slot.into()),
            (ARG_ENABLED.into(), enabled.into()),
        ],
    )
}

/// A block's model map: `{23: amp+cab, 25: model, 26: cab or -1}`.
fn block_models(kind: &BlockKind) -> rmpv::Value {
    let (amp_cab, model, cab) = match *kind {
        BlockKind::Model(model) => (false, model, -1),
        BlockKind::AmpCab { amp, cab } => (true, amp, i32::from(cab)),
    };
    rmpv::Value::Map(vec![
        (KEY_AMP_CAB.into(), amp_cab.into()),
        (KEY_MODEL_A.into(), model.into()),
        (KEY_MODEL_B.into(), cab.into()),
    ])
}

/// Builds a request replacing the model of the block in preset slot
/// `slot` with `kind`. The device answers with the new block, its
/// parameters at their defaults.
pub fn build_set_block_model_request(slot: u8, kind: &BlockKind) -> Vec<u8> {
    build_edit_request(
        CMD_SET_BLOCK_MODEL,
        vec![
            (ARG_SLOT.into(), slot.into()),
            (ARG_MODELS.into(), block_models(kind)),
        ],
    )
}

/// Builds a request adding a block of `kind` at the empty preset slot
/// `slot`, switched on. `block_type` is the block's type (see
/// [`BlockCategory::block_type`](crate::blocks::BlockCategory::block_type)). The device answers with
/// the new block, its parameters at their defaults.
pub fn build_add_block_request(slot: u8, kind: &BlockKind, block_type: u8) -> Vec<u8> {
    let block = rmpv::Value::Map(vec![
        (KEY_SLOT_TYPE.into(), SLOT_TYPE_BLOCK.into()),
        (
            KEY_SLOT_BODY.into(),
            rmpv::Value::Map(vec![
                (KEY_MODELS.into(), block_models(kind)),
                (KEY_BLOCK_TYPE.into(), block_type.into()),
                (KEY_ENABLED.into(), true.into()),
            ]),
        ),
    ]);
    build_edit_request(
        CMD_ADD_BLOCK,
        vec![
            (ARG_SLOT.into(), slot.into()),
            (ARG_NEW_BLOCK.into(), block),
        ],
    )
}

/// Builds a request removing the block in preset slot `slot`.
pub fn build_remove_block_request(slot: u8) -> Vec<u8> {
    build_edit_request(CMD_REMOVE_BLOCK, vec![(ARG_SLOT.into(), slot.into())])
}

/// Builds a request storing the active preset, with its edits, as `name`
/// in `preset` (a slot within `setlist`).
///
/// Like a rename, the name is stored NUL-terminated.
pub fn build_save_preset_request(setlist: u8, preset: u8, name: &str) -> Vec<u8> {
    build_edit_request(
        CMD_SAVE_PRESET,
        vec![
            (ARG_SETLIST.into(), setlist.into()),
            (ARG_PRESET.into(), preset.into()),
            (ARG_NAME.into(), format!("{name}\0").into()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Byte-offset constants
    const SEQ_OFFSET: usize = 9;
    const CMD_OFFSET: usize = 11;

    // Builders

    #[test]
    fn build_pagination_request_formats_correctly() {
        let pkt = build_pagination_request(PRESET_ROUTING_OUT, 0x08, STREAM_INITIAL_OFFSET);

        assert_eq!(pkt.len(), 16);
        assert_eq!(pkt[SEQ_OFFSET], 0x08);
        assert_eq!(pkt[CMD_OFFSET], 0x08); // command: chunk request

        // Offset should be Little-Endian (0x0000_1138 -> 38 11 00 00)
        assert_eq!(&pkt[12..16], &[0x38, 0x11, 0x00, 0x00]);
    }

    /// Edit-channel requests as captured from an HX Stomp.
    #[rustfmt::skip]
    const READ_ACTIVE_PRESET_CAPTURED: &[u8] = &[
        0x19, 0x00, 0x00, 0x18, 0x80, 0x10, 0xED, 0x03, 0x00, 0x03, 0x00, 0x0C,
        0x09, 0x10, 0x00, 0x00, 0x01, 0x00, 0x06, 0x00, 0x09, 0x00, 0x00, 0x00,
        0x83, 0x66, 0xCD, 0x03, 0xF4, 0x64, 0x16, 0x65, 0xC0, 0x00, 0x00, 0x00,
    ];
    #[rustfmt::skip]
    const READ_ACTIVE_PRESET_INFO_CAPTURED: &[u8] = &[
        0x19, 0x00, 0x00, 0x18, 0x80, 0x10, 0xED, 0x03, 0x00, 0x03, 0x00, 0x04,
        0x09, 0x10, 0x00, 0x00, 0x01, 0x00, 0x06, 0x00, 0x09, 0x00, 0x00, 0x00,
        0x83, 0x66, 0xCD, 0x04, 0x04, 0x64, 0x17, 0x65, 0xC0, 0x00, 0x00, 0x00,
    ];

    /// MsgPack bodies of edit-channel writes captured from HX Edit, from the
    /// command type on (the transaction IDs differ).
    const SET_DRIVE_CAPTURED: &[u8] = &[
        0x64, 0x1E, 0x65, 0x85, 0x62, 0x01, 0x1D, 0xC3, 0x1A, 0x00, 0x1C, 0x00, 0x77, 0xCA, 0x3E,
        0xB3, 0x33, 0x33,
    ];
    const SET_CAB_DISTANCE_CAPTURED: &[u8] = &[
        0x64, 0x1E, 0x65, 0x85, 0x62, 0x03, 0x1D, 0xC3, 0x1A, 0x01, 0x1C, 0x00, 0x77, 0xCA, 0x3F,
        0xA0, 0x00, 0x00,
    ];
    const BYPASS_CAPTURED: &[u8] = &[0x64, 0x29, 0x65, 0x82, 0x62, 0x01, 0x3B, 0xC2];
    const SAVE_CAPTURED: &[u8] = &[
        0x64, 0x47, 0x65, 0x83, 0x6B, 0x00, 0x6C, 0x01, 0x6D, 0xA9, b'P', b'o', b'p', b'-', b'P',
        b'u', b'n', b'k', 0x00,
    ];

    /// The MsgPack body of `packet` after `{102: txn,`.
    fn body_after_txn(packet: &[u8]) -> &[u8] {
        let len = u32::from_le_bytes(packet[20..24].try_into().unwrap()) as usize;
        // fixmap(3), key 102, uint16 txn
        &packet[24 + 5..24 + len]
    }

    const SET_MODEL_CAPTURED: &[u8] = &[
        0x64, 0x28, 0x65, 0x82, 0x62, 0x01, 0x64, 0x83, 0x17, 0xC2, 0x19, 0x64, 0x1A, 0xFF,
    ];
    const ADD_TILT_CAPTURED: &[u8] = &[
        0x64, 0x27, 0x65, 0x82, 0x62, 0x02, 0x63, 0x82, 0x13, 0x06, 0x14, 0x83, 0x18, 0x83, 0x17,
        0xC2, 0x19, 0xCD, 0x02, 0x11, 0x1A, 0xFF, 0x09, 0x01, 0x0A, 0xC3,
    ];
    const REMOVE_CAPTURED: &[u8] = &[0x64, 0x1C, 0x65, 0x81, 0x62, 0x02];

    #[test]
    fn block_writes_match_captures() {
        // Minotaur (100) in place of the block in slot 1.
        let set_model = build_set_block_model_request(1, &BlockKind::Model(100));
        assert_eq!(body_after_txn(&set_model), SET_MODEL_CAPTURED);

        // Tilt (529) added at slot 2.
        let add = build_add_block_request(2, &BlockKind::Model(529), 1);
        assert_eq!(body_after_txn(&add), ADD_TILT_CAPTURED);

        assert_eq!(
            body_after_txn(&build_remove_block_request(2)),
            REMOVE_CAPTURED
        );
    }

    #[test]
    fn edit_writes_match_captures() {
        let drive = build_set_param_request(1, 0, 0, rmpv::Value::F32(0.35));
        assert_eq!(&drive[4..8], &EDIT_ROUTING_OUT);
        assert_eq!(drive[SEQ_OFFSET], READ_ACTIVE_PRESET_SEQ);
        assert_eq!(drive.len() % 4, 0);
        assert_eq!(usize::from(drive[0]), 24 + 23 - 8);
        assert_eq!(body_after_txn(&drive), SET_DRIVE_CAPTURED);

        let distance = build_set_param_request(3, 1, 0, rmpv::Value::F32(1.25));
        assert_eq!(body_after_txn(&distance), SET_CAB_DISTANCE_CAPTURED);

        let bypass = build_set_block_enabled_request(1, false);
        assert_eq!(body_after_txn(&bypass), BYPASS_CAPTURED);

        let save = build_save_preset_request(0, 1, "Pop-Punk");
        assert_eq!(body_after_txn(&save), SAVE_CAPTURED);
    }

    #[test]
    fn edit_requests_match_captures() {
        assert_eq!(READ_ACTIVE_PRESET, READ_ACTIVE_PRESET_CAPTURED);
        assert_eq!(READ_ACTIVE_PRESET_INFO, READ_ACTIVE_PRESET_INFO_CAPTURED);

        let page = build_pagination_request(EDIT_ROUTING_OUT, 0x04, STREAM_INITIAL_OFFSET);
        assert_eq!(&page[4..8], &EDIT_ROUTING_OUT);
    }

    /// Start-paged-stream packet as captured from an HX Stomp XL
    /// (`cmd = 0x0C`, `seq = 0x07`, setlist 0).
    const OPEN_STREAM_CAPTURED: &[u8] = &[
        0x1D, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03, 0x00, 0x07, 0x00, 0x0C, 0x38, 0x10, 0x00,
        0x00, 0x01, 0x00, 0x02, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x83, 0x66, 0xCD, 0x03, 0xEA, 0x64,
        0x01, 0x65, 0x82, 0x6B, 0x00, 0x65, 0x02, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn build_open_stream_request_for_setlist_zero_matches_captured_packet() {
        let pkt = build_open_stream_request(OPEN_STREAM_SEQ, 0);
        assert_eq!(&pkt[..], OPEN_STREAM_CAPTURED);
    }

    #[test]
    fn build_open_stream_request_places_setlist_correctly() {
        let pkt = build_open_stream_request(OPEN_STREAM_SEQ, 5);

        assert_eq!(pkt.len(), 40);
        assert_eq!(pkt[SEQ_OFFSET], OPEN_STREAM_SEQ);
        assert_eq!(pkt[CMD_OFFSET], 0x0C); // command: start paged stream
        assert_eq!(pkt[34], 5, "setlist index mismatch");
    }

    #[test]
    fn build_select_preset_request_places_args_correctly() {
        let setlist_idx = 3;
        let preset_idx = 15;
        let pkt = build_select_preset_request(0x06, setlist_idx, preset_idx);

        assert_eq!(pkt.len(), 40);
        assert_eq!(pkt[SEQ_OFFSET], 0x06);
        assert_eq!(pkt[CMD_OFFSET], 0x04); // command: open

        // Validate dynamic payload injection points
        assert_eq!(pkt[34], setlist_idx, "setlist index mismatch");
        assert_eq!(pkt[36], preset_idx, "preset index mismatch");
    }

    #[test]
    fn build_rename_preset_request_encodes_name_and_args() {
        let pkt = build_rename_preset_request(0x06, 0, 12, "Clean");

        #[rustfmt::skip]
        let expected: &[u8] = &[
            0x25, 0x00, 0x00, 0x18, 0x01, 0x10, 0xEF, 0x03,
            0x00, 0x06, 0x00, 0x04, 0x1A, 0x10, 0x00, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x15, 0x00, 0x00, 0x00,
            // {102: 1005, 100: 6, 101: {107: 0, 108: 12, 109: "Clean\0"}}
            0x83, 0x66, 0xCD, 0x03, 0xED, 0x64, 0x06, 0x65,
            0x83, 0x6B, 0x00, 0x6C, 0x0C, 0x6D, 0xA6, b'C',
            b'l', b'e', b'a', b'n', 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(&pkt[..], expected);
    }

    #[test]
    fn build_rename_preset_request_payload_decodes() {
        let pkt = build_rename_preset_request(0x06, 3, 127, "Lead Tone");
        let len = u32::from_le_bytes(pkt[20..24].try_into().unwrap()) as usize;
        let mut cursor = std::io::Cursor::new(&pkt[24..24 + len]);
        let value = rmpv::decode::read_value(&mut cursor).unwrap();

        let args = value
            .as_map()
            .unwrap()
            .iter()
            .find(|(k, _)| k.as_u64() == Some(101))
            .map(|(_, v)| v.as_map().unwrap())
            .unwrap();
        let arg = |key: u64| {
            args.iter()
                .find(|(k, _)| k.as_u64() == Some(key))
                .map(|(_, v)| v.clone())
                .unwrap()
        };

        assert_eq!(arg(107).as_u64(), Some(3));
        assert_eq!(arg(108).as_u64(), Some(127));
        assert_eq!(arg(109).as_str(), Some("Lead Tone\0"));
    }

    #[test]
    fn build_rename_preset_request_is_padded_and_framed() {
        for name in ["A", "AB", "ABC", "ABCD", "Sixteen Chars!!!"] {
            let pkt = build_rename_preset_request(0x06, 0, 0, name);
            let payload_len = u32::from_le_bytes(pkt[20..24].try_into().unwrap()) as usize;

            assert_eq!(pkt.len() % 4, 0, "{name:?}: not 4-byte aligned");
            assert_eq!(pkt[0] as usize, 16 + payload_len, "{name:?}: bad frame len");
            assert!(pkt.len() >= 24 + payload_len, "{name:?}: truncated");
        }
    }

    // Static Packets & Sequences

    #[test]
    fn session_init_sequence_is_ordered_correctly() {
        let expected_seqs = [0x00u8, 0x02, 0x03, 0x04, 0x05];

        assert_eq!(SESSION_INIT_SEQUENCE.len(), 5);
        for (packet, &expected) in SESSION_INIT_SEQUENCE.iter().zip(&expected_seqs) {
            assert_eq!(
                packet[SEQ_OFFSET], expected,
                "unexpected seq in session init packet"
            );
        }
    }

    #[test]
    fn all_packets_use_standard_routing_field() {
        let all_packets: &[&[u8]] = &[
            HANDSHAKE,
            SESSION_OPEN_1,
            SESSION_CHUNK_1,
            SESSION_OPEN_2,
            SESSION_CHUNK_2,
            OPEN_PRESETS,
            &build_open_stream_request(OPEN_STREAM_SEQ, 0),
            &build_select_preset_request(0x06, 0, 0),
            &build_rename_preset_request(0x06, 0, 0, "Name"),
        ];

        for pkt in all_packets {
            assert_eq!(
                &pkt[4..8],
                &[0x01, 0x10, 0xEF, 0x03],
                "packet does not carry standard routing field"
            );
        }
    }
}
