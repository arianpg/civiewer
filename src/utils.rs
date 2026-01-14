use std::path::Path;

pub const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

pub fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
        SUPPORTED_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

pub fn is_apng(path: &Path) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::{Read, Seek};
        let mut header = [0u8; 8];
        if file.read_exact(&mut header).is_err() { return false; }
        // PNG Signature: 89 50 4E 47 0D 0A 1A 0A
        if header != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] { return false; }

        loop {
            let mut length_bytes = [0u8; 4];
            if file.read_exact(&mut length_bytes).is_err() { break; }
            let length = u32::from_be_bytes(length_bytes) as u64;

            let mut type_bytes = [0u8; 4];
            if file.read_exact(&mut type_bytes).is_err() { break; }
            
            // Chunk Type
            if &type_bytes == b"acTL" { return true; }
            if &type_bytes == b"IDAT" { return false; } // acTL must come before IDAT

            // Skip data + crc (4 bytes)
            if file.seek(std::io::SeekFrom::Current((length + 4) as i64)).is_err() { break; }
        }
    }
    false
}

pub fn is_apng_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 8 || &bytes[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return false;
    }
    
    let mut offset = 8;
    while offset + 8 <= bytes.len() {
        let length = u32::from_be_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        let type_bytes = &bytes[offset+4..offset+8];
        
        if type_bytes == b"acTL" { return true; }
        if type_bytes == b"IDAT" { return false; }

        offset += 8 + length + 4; // Length + Type + Data + CRC
    }
    false
}
