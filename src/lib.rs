/// "VMDK"
const EXTENT_MAGIC: u32 = 0x564d444b;
const EXTENT_VERSION: u32 = 1;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};
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

pub struct Vmdk(File);

impl Vmdk {
    pub fn new() -> Result<Self, Error> {
        let mut file = File::open("/home/josh/VirtualBox VMs/\
                                  OMS CS6250 Course VM/OMS CS6250 Course VM-disk1.vmdk")?;

        // Extent Header
        file.seek(SeekFrom::Start(0))?;
        let magic = file.read_u32::<LittleEndian>()?;
        eprintln!("Magic: 0x{:x}", magic);
        if magic != EXTENT_MAGIC {
            return Err(VmdkError::ParseError.into());
        }

        // Embedded Descriptor
        file.seek(SeekFrom::Start(512))?;
        let mut buf = [0u8; 0x2a0];
        file.read_exact(&mut buf)?;
        let string = std::str::from_utf8(&buf)?;
        eprintln!("String: {}", string);
        Ok(Vmdk(file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vmdk() {
        let vmdk = Vmdk::new().unwrap();
    }
}
