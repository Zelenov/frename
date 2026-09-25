//! Finding the XMP packet in a MOV/MP4 file by walking box headers.
//!
//! Adobe's XMP Toolkit reads XMP by parsing the whole movie header, sample tables included,
//! which took about 55 ms per clip: a folder scan of 1254 videos ran over a minute. The
//! packet itself sits in one known place, so this reads the box headers on the way there
//! and the packet, a handful of small reads. Writing still goes through the toolkit.
//!
//! Where the toolkit puts XMP: QuickTime files (`qt  ` brand) in `moov/udta/XMP_`, other
//! ISO media files in a top-level `uuid` box with the XMP UUID.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// UUID of the top-level box holding XMP in ISO media files (XMP Specification, part 3).
const XMP_UUID: [u8; 16] = [
    0xBE, 0x7A, 0xCF, 0xCB, 0x97, 0xA9, 0x42, 0xE8, 0x9C, 0x71, 0x99, 0x94, 0x91, 0xE3, 0xAF, 0xAC,
];

/// A packet larger than this is not XMP frename wrote; leave such files to the toolkit.
const MAX_PACKET: u64 = 16 * 1024 * 1024;

/// Top-level box types a MOV/MP4 file can start with. Anything else is not one, whatever
/// its extension says.
const FIRST_BOXES: [&[u8; 4]; 7] = [
    b"ftyp", b"moov", b"mdat", b"wide", b"free", b"skip", b"pnot",
];

/// The XMP packet of a MOV/MP4 file: `Some(None)` when the file has none, `None` when this
/// reader cannot tell (not a MOV/MP4, or a layout it does not understand), in which case
/// the caller asks the toolkit.
pub(super) fn find_xmp_packet(path: &Path) -> Option<Option<String>> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(ext.as_str(), "mov" | "mp4" | "m4v") {
        return None;
    }
    scan(path).ok().flatten()
}

fn scan(path: &Path) -> io::Result<Option<Option<String>>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let Some(first) = read_header(&mut file, 0, len)? else {
        return Ok(None);
    };
    if !FIRST_BOXES.contains(&&first.kind) {
        return Ok(None);
    }

    let mut quicktime = false;
    let mut in_uuid = None;
    let mut in_udta = None;
    let mut pos = 0;
    while let Some(b) = read_header(&mut file, pos, len)? {
        match &b.kind {
            b"ftyp" if b.len() >= 4 => {
                quicktime = &read_bytes(&mut file, b.payload, 4)?[..] == b"qt  "
            }
            b"uuid" if b.len() >= 16 => {
                if read_bytes(&mut file, b.payload, 16)? == XMP_UUID {
                    in_uuid = Some(read_packet(&mut file, b.payload + 16, b.end)?);
                }
            }
            b"moov" => in_udta = find_in_moov(&mut file, &b)?.or(in_udta),
            _ => {}
        }
        pos = b.end;
    }
    let packet = if quicktime {
        in_udta.or(in_uuid)
    } else {
        in_uuid.or(in_udta)
    };
    Ok(Some(packet))
}

/// `moov/udta/XMP_`, if present.
fn find_in_moov(file: &mut File, moov: &BoxHeader) -> io::Result<Option<String>> {
    let mut pos = moov.payload;
    while let Some(child) = read_header(file, pos, moov.end)? {
        if &child.kind == b"udta" {
            let mut inner = child.payload;
            while let Some(entry) = read_header(file, inner, child.end)? {
                if &entry.kind == b"XMP_" {
                    return read_packet(file, entry.payload, entry.end).map(Some);
                }
                inner = entry.end;
            }
        }
        pos = child.end;
    }
    Ok(None)
}

/// A box: its type, where its payload starts and where it ends (absolute offsets).
struct BoxHeader {
    kind: [u8; 4],
    payload: u64,
    end: u64,
}

impl BoxHeader {
    fn len(&self) -> u64 {
        self.end - self.payload
    }
}

/// The box header at `pos`, or `None` once fewer than 8 bytes are left before `limit`.
/// A header that points outside its parent is invalid data.
fn read_header(file: &mut File, pos: u64, limit: u64) -> io::Result<Option<BoxHeader>> {
    if pos.saturating_add(8) > limit {
        return Ok(None);
    }
    let head = read_bytes(file, pos, 8)?;
    let size = u32::from_be_bytes([head[0], head[1], head[2], head[3]]);
    let kind = [head[4], head[5], head[6], head[7]];
    let (payload, end) = match size {
        0 => (pos + 8, limit),
        1 => {
            let large = read_bytes(file, pos + 8, 8)?;
            let large = u64::from_be_bytes(large.try_into().map_err(|_| invalid())?);
            (pos + 16, pos.checked_add(large).ok_or_else(invalid)?)
        }
        n => (pos + 8, pos + u64::from(n)),
    };
    if end < payload || end > limit {
        return Err(invalid());
    }
    Ok(Some(BoxHeader { kind, payload, end }))
}

fn read_packet(file: &mut File, start: u64, end: u64) -> io::Result<String> {
    let len = end - start;
    if len > MAX_PACKET {
        return Err(invalid());
    }
    let bytes = read_bytes(file, start, len as usize)?;
    // Packets are padded with spaces or NULs so they can grow in place.
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.trim_end_matches(['\0', ' ', '\n', '\r']).to_string())
}

fn read_bytes(file: &mut File, pos: u64, len: usize) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(pos))?;
    let mut buf = vec![0; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "malformed MOV/MP4 box")
}
