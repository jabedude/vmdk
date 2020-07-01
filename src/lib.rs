/// "VMDK"
const EXTENT_MAGIC: u32 = 0x564d444b;
const EXTENT_VERSION: u32 = 1;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Error, Fail};
use log::info;

#[derive(Debug, Fail)]
pub enum VmdkError {
    #[fail(display = "Parsing error")]
    ParseError,
}

#[derive(Debug)]
pub struct ExtentHeader {
    /// The header signature "KDMV"
    pub magic_number: u32,
    /// Version (1, 2, or 3)
    pub version: u32,
    /// Flags
    pub flags: u32,
    /// Maximum data sectors
    pub capacity: u64,
    /// Grain number of sectors (power of 2 and >8)
    pub grain_size: u64,
    /// The sector number of the embedded descriptor file. Offset from 
    /// beginning of file
    pub desc_offset: u64,
    /// Number of sectors of the embedded descriptor file
    pub desc_size: u64,
    /// Number of grain table entries
    pub gtes_per_gt: u32,
    /// Secondary grain directory sector offset
    pub rgd_offset: u64,
    /// Grain directory sector number
    pub gd_offset: u64,
    /// Metadata overhead of sectors
    pub overhead: u64,
    /// Determines if extent data was cleanly closed
    pub dirty_shutdown: u8,
    /// Single EOL character
    pub single_eol_char: u8,
    /// Non EOL character
    pub non_eol_char: u8,
    /// Second double EOL character
    pub dbl_eol_char: u8,
    /// Compression method
    pub compress_method: u16,
}

impl ExtentHeader {
    pub fn new<R: Read>(mut reader: R) -> Result<Self, Error> {
        let magic = reader.read_u32::<LittleEndian>()?;
        info!("Magic: 0x{:x}", magic);
        if magic != EXTENT_MAGIC {
            return Err(VmdkError::ParseError.into());
        }

        let version = reader.read_u32::<LittleEndian>()?;
        info!("Version: 0x{:x}", version);
        if version != EXTENT_VERSION {
            return Err(VmdkError::ParseError.into());
        }

        let flags = reader.read_u32::<LittleEndian>()?;
        info!("Flags: 0x{:x}", flags);

        let capacity = reader.read_u64::<LittleEndian>()?;
        info!("Capacity: 0x{:x}", capacity);

        let grain_size = reader.read_u64::<LittleEndian>()?;
        info!("Grain Size: 0x{:x}", grain_size);

        let desc_offset = reader.read_u64::<LittleEndian>()?;
        info!("Descriptor offset: 0x{:x}", desc_offset);

        let desc_size = reader.read_u64::<LittleEndian>()?;
        info!("Descriptor size: 0x{:x}", desc_size);

        let gte_per_gt = reader.read_u32::<LittleEndian>()?;
        info!("GTE's per GT: 0x{:x}", gte_per_gt);

        let rgd_offset = reader.read_u64::<LittleEndian>()?;
        info!("Redundant GD offset: 0x{:x}", rgd_offset);

        let gd_offset = reader.read_u64::<LittleEndian>()?;
        info!("GD Offset: 0x{:x}", gd_offset);

        let meta_overhead = reader.read_u64::<LittleEndian>()?;
        info!("Metadata overhead: 0x{:x}", meta_overhead);

        let dirty_shutdown = reader.read_u8()?;
        info!("Dirty shutdown: 0x{:x}", dirty_shutdown);

        let eol_char = reader.read_u8()?;
        info!("Single EOL Character: 0x{:x}", eol_char);

        let non_eol_char = reader.read_u8()?;
        info!("Non EOL Char: 0x{:x}", non_eol_char);

        let dbl_eol_char = reader.read_u8()?;
        info!("Double EOL Char: 0x{:x}", dbl_eol_char);

        let compress_method = reader.read_u16::<LittleEndian>()?;
        info!("Compression Algo: 0x{:x}", compress_method);

        let ext = ExtentHeader {
            magic_number: magic,
            version: version,
            flags: flags,
            capacity: capacity,
            grain_size: grain_size,
            desc_offset: desc_offset,
            desc_size: desc_size,
            gtes_per_gt: gte_per_gt,
            rgd_offset: rgd_offset,
            gd_offset: gd_offset,
            overhead: meta_overhead,
            dirty_shutdown: dirty_shutdown,
            single_eol_char: eol_char,
            non_eol_char: non_eol_char,
            dbl_eol_char: dbl_eol_char,
            compress_method: compress_method,
        };

        Ok(ext)
    }
}

pub struct Vmdk {
    extent_header: Option<ExtentHeader>,
    file: File,
}

impl Vmdk {
    // TODO: make the input generic over R: Read
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(path)?;

        // Extent Header
        file.seek(SeekFrom::Start(0))?;
        let extent_header = ExtentHeader::new(&mut file)?;
        eprintln!("Ext: {:?}", extent_header);

        // Embedded Descriptor
        file.seek(SeekFrom::Start(512))?;
        let mut buf = [0u8; 0x2a0];
        file.read_exact(&mut buf)?;
        let string = std::str::from_utf8(&buf)?;
        eprintln!("String: {}", string);

        Ok(Vmdk {
            extent_header: Some(extent_header),
            file: file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vmdk() {
        let vmdk = Vmdk::new("/home/josh/VirtualBox VMs/OMS CS6250 Course \
                             VM/OMS CS6250 Course VM-disk1.vmdk").unwrap();
    }
}
