/// "VMDK"
const EXTENT_MAGIC: u32 = 0x564d444b;
const EXTENT_VERSION: u32 = 1;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Error, Fail};

#[derive(Debug, Fail)]
pub enum VmdkError {
    #[fail(display = "Parsing error")]
    ParseError,
}

pub struct ExtentHeader {
    /// The header signature "KDMV"
    pub magicNumber: u32,
    /// Version (1, 2, or 3)
    pub version: u32,
    /// Flags
    pub flags: u32,
    /// Maximum data sectors
    pub capacity: u64,
    /// Grain number of sectors (power of 2 and >8)
    pub grainSize: u64,
    /// The sector number of the embedded descriptor file. Offset from 
    /// beginning of file
    pub descriptorOffset: u64,
    /// Number of sectors of the embedded descriptor file
    pub descriptorSize: u64,
    /// Number of grain table entries
    pub numGTEsPerGT: u32,
    /// Secondary grain directory sector offset
    pub rgdOffset: u64,
    /// Grain directory sector number
    pub gdOffset: u64,
    /// Metadata overhead of sectors
    pub overHead: u64,
    /// Determines if extent data was cleanly closed
    pub uncleanShutdown: u8,
    /// Single EOL character
    pub singleEndLineChar: u8,
    /// Non EOL character
    pub nonEndLineChar: u8,
    /// Second double EOL character
    pub doubleEndLineChar: u8,
    /// Compression method
    pub compressAlgorithm: u16,
    /// Padding
    pub pad: [u8; 433],
}

pub struct Vmdk {
    extent_header: Option<ExtentHeader>,
    file: File,
}

impl Vmdk {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(path)?;

        // Extent Header
        file.seek(SeekFrom::Start(0))?;
        let magic = file.read_u32::<LittleEndian>()?;
        eprintln!("Magic: 0x{:x}", magic);
        if magic != EXTENT_MAGIC {
            return Err(VmdkError::ParseError.into());
        }

        let version = file.read_u32::<LittleEndian>()?;
        eprintln!("Version: 0x{:x}", version);
        if version != EXTENT_VERSION {
            return Err(VmdkError::ParseError.into());
        }

        let flags = file.read_u32::<LittleEndian>()?;
        eprintln!("Flags: 0x{:x}", flags);

        let capacity = file.read_u64::<LittleEndian>()?;
        eprintln!("Capacity: 0x{:x}", capacity);

        let grain_size = file.read_u64::<LittleEndian>()?;
        eprintln!("Grain Size: 0x{:x}", grain_size);

        let desc_offset = file.read_u64::<LittleEndian>()?;
        eprintln!("Descriptor offset: 0x{:x}", desc_offset);

        let desc_size = file.read_u64::<LittleEndian>()?;
        eprintln!("Descriptor size: 0x{:x}", desc_size);

        let gte_per_gt = file.read_u32::<LittleEndian>()?;
        eprintln!("GTE's per GT: 0x{:x}", gte_per_gt);

        let rgd_offset = file.read_u64::<LittleEndian>()?;
        eprintln!("Redundant GD offset: 0x{:x}", rgd_offset);

        let gd_offset = file.read_u64::<LittleEndian>()?;
        eprintln!("GD Offset: 0x{:x}", gd_offset);

        let meta_overhead = file.read_u64::<LittleEndian>()?;
        eprintln!("Metadata overhead: 0x{:x}", meta_overhead);

        let dirty_shutdown = file.read_u8()?;
        eprintln!("Dirty shutdown: 0x{:x}", dirty_shutdown);

        let eol_char = file.read_u8()?;
        eprintln!("Single EOL Character: 0x{:x}", eol_char);

        let non_eol_char = file.read_u8()?;
        eprintln!("Non EOL Char: 0x{:x}", non_eol_char);

        let dbl_eol_char = file.read_u8()?;
        eprintln!("Double EOL Char: 0x{:x}", dbl_eol_char);

        let compress_method = file.read_u16::<LittleEndian>()?;
        eprintln!("Compression Algo: 0x{:x}", compress_method);

        // Embedded Descriptor
        file.seek(SeekFrom::Start(512))?;
        let mut buf = [0u8; 0x2a0];
        file.read_exact(&mut buf)?;
        let string = std::str::from_utf8(&buf)?;
        eprintln!("String: {}", string);

        Ok(Vmdk {
            extent_header: None,
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
