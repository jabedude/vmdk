/// "VMDK"
const EXTENT_MAGIC: u32 = 0x564d444b;
const EXTENT_VERSION: u32 = 1;
const SECTOR_SIZE: u64 = 512;

use std::convert::TryInto;
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
pub struct SectorType(u64);

#[derive(Debug)]
pub struct ExtentHeader {
    /// The header signature "KDMV"
    pub magic_number: u32,
    /// Version (1, 2, or 3)
    pub version: u32,
    /// Flags
    pub flags: u32,
    /// Maximum data sectors
    pub capacity: SectorType,
    /// Grain number of sectors (power of 2 and >8)
    pub grain_size: SectorType,
    /// The sector number of the embedded descriptor file. Offset from 
    /// beginning of file
    pub desc_offset: SectorType,
    /// Number of sectors of the embedded descriptor file
    pub desc_size: SectorType,
    /// Number of grain table entries
    pub gtes_per_gt: u32,
    /// Secondary grain directory sector offset
    pub rgd_offset: SectorType,
    /// Grain directory sector number
    pub gd_offset: SectorType,
    /// Metadata overhead of sectors
    pub overhead: SectorType,
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
        let capacity = SectorType(capacity);

        let grain_size = reader.read_u64::<LittleEndian>()?;
        info!("Grain Size: 0x{:x}", grain_size);
        let grain_size = SectorType(grain_size);

        let desc_offset = reader.read_u64::<LittleEndian>()?;
        info!("Descriptor offset: 0x{:x}", desc_offset);
        let desc_offset = SectorType(desc_offset);

        let desc_size = reader.read_u64::<LittleEndian>()?;
        info!("Descriptor size: 0x{:x}", desc_size);
        let desc_size = SectorType(desc_size);

        let gte_per_gt = reader.read_u32::<LittleEndian>()?;
        info!("GTE's per GT: 0x{:x}", gte_per_gt);

        let rgd_offset = reader.read_u64::<LittleEndian>()?;
        info!("Redundant GD offset: 0x{:x}", rgd_offset);
        let rgd_offset = SectorType(rgd_offset);

        let gd_offset = reader.read_u64::<LittleEndian>()?;
        info!("GD Offset: 0x{:x}", gd_offset);
        let gd_offset = SectorType(gd_offset);

        let meta_overhead = reader.read_u64::<LittleEndian>()?;
        info!("Metadata overhead: 0x{:x}", meta_overhead);
        let meta_overhead = SectorType(meta_overhead);

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
    pub extent_header: Option<ExtentHeader>,
    pub descriptor: Option<String>,
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
        //let mut buf = [0u8; 0x2a0];
        let desc_size_in_bytes = extent_header.desc_size.0 * SECTOR_SIZE;
        let mut buf: Vec<u8> = vec![0u8; desc_size_in_bytes.try_into()?];
        file.read_exact(&mut buf)?;
        let descriptor = std::str::from_utf8(&buf)?.to_owned();
        let descriptor = descriptor.trim_matches(char::from(0)).to_owned();
        eprintln!("Descriptor string: {}", descriptor);
        eprintln!("Descriptor string len: {}", descriptor.len());

        Ok(Vmdk {
            extent_header: Some(extent_header),
            descriptor: Some(descriptor),
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

    #[test]
    fn test_descriptor() {
        let vmdk = Vmdk::new("/home/josh/VirtualBox VMs/OMS CS6250 Course \
                             VM/OMS CS6250 Course VM-disk1.vmdk").unwrap();
        let descriptor = r#"# Disk DescriptorFile
version=1
CID=def0d352
parentCID=ffffffff
createType="monolithicSparse"

# Extent description
RW 41943040 SPARSE "OMS CS6250 Course VM-disk1.vmdk"

# The disk Data Base 
#DDB

ddb.virtualHWVersion = "4"
ddb.adapterType="ide"
ddb.geometry.cylinders="16383"
ddb.geometry.heads="16"
ddb.geometry.sectors="63"
ddb.geometry.biosCylinders="1024"
ddb.geometry.biosHeads="255"
ddb.geometry.biosSectors="63"
ddb.uuid.image="2ebfd8e9-9868-4688-8f3f-97e3f9def370"
ddb.uuid.parent="00000000-0000-0000-0000-000000000000"
ddb.uuid.modification="e2b662bc-16ff-478e-8b7f-3323b027087e"
ddb.uuid.parentmodification="00000000-0000-0000-0000-000000000000"
ddb.comment=""
"#;
        eprintln!("Descriptor: {}", descriptor);
        assert_eq!(vmdk.descriptor.unwrap(), descriptor);
    }
}
