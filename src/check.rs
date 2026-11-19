use std::fs::File;
use std::path::Path;

use crate::extfs::{
    Superblock, Volume, COMPAT_HAS_JOURNAL, INCOMPAT_64BIT, INCOMPAT_EXTENTS, RO_COMPAT_SPARSE_SUPER2,
};

pub struct DetectedType {
    pub fs_type: &'static str,
    pub has_journal: bool,
    pub has_extents: bool,
    pub is_64bit: bool,
    pub sparse_super2: bool,
}

pub fn detect_type(sb: &Superblock) -> DetectedType {
    let has_journal = (sb.s_feature_compat & COMPAT_HAS_JOURNAL) != 0;
    let has_extents = (sb.s_feature_incompat & INCOMPAT_EXTENTS) != 0;
    let is_64bit = (sb.s_feature_incompat & INCOMPAT_64BIT) != 0;
    let sparse_super2 = (sb.s_feature_ro_compat & RO_COMPAT_SPARSE_SUPER2) != 0;

    let fs_type = if sb.s_inode_size >= 256 {
        if has_extents {
            "EXT4"
        } else {
            "EXT4 (Legacy)"
        }
    } else if has_journal {
        "EXT3"
    } else {
        "EXT2"
    };

    DetectedType {
        fs_type,
        has_journal,
        has_extents,
        is_64bit,
        sparse_super2,
    }
}

pub fn check_ext(img_path: &Path) {
    if !img_path.exists() {
        println!("File not found!");
        return;
    }

    let file = match File::open(img_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error reading image");
            println!("Detail: {}", e);
            return;
        }
    };

    let vol = match Volume::open(file) {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Invalid magic value") {
                println!("Not EXT4 (Magic mismatch)");
            } else {
                println!("Error reading image");
                println!("Detail: {}", msg);
            }
            return;
        }
    };

    let sb = &vol.superblock;
    let det = detect_type(sb);

    println!();
    println!("RennsAndroidKitchen CLI - EXT CHECKER");
    println!("=================================");
    println!(
        "Valid EXT filesystem: {}",
        if sb.s_magic == 0xEF53 { "YES" } else { "NO" }
    );
    println!("Magic number       : {:#x}", sb.s_magic);
    println!("Detected type      : {}", det.fs_type);
    println!(
        "Journal support    : {}",
        if det.has_journal { "YES" } else { "NO" }
    );
    println!(
        "EXTENTS (EXT4)     : {}",
        if det.has_extents { "YES" } else { "NO" }
    );
    println!(
        "64-bit mode        : {}",
        if det.is_64bit { "YES" } else { "NO" }
    );
    println!(
        "Sparse Super2      : {}",
        if det.sparse_super2 { "YES" } else { "NO" }
    );
    println!("---------------------------------");
    println!("Block size         : {}", vol.block_size);
    println!("Block count        : {}", sb.s_blocks_count);
    println!("Inode size         : {}", sb.s_inode_size);
    println!("Volume name        : {}", sb.volume_name());
    println!("Mounted path       : {}", sb.last_mounted());
    println!("UUID               : {}", sb.uuid_string());
    println!("=================================");
}
