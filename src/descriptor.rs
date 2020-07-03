use std::str::FromStr;
use std::default::Default;
use std::path::PathBuf;
use crate::VmdkError;
use failure::Error;

const CID_NOPARENT: u32 = 0xffffffff;


#[derive(Debug, Default)]
pub struct Descriptor {
    /// Version
    version: u32,
    /// Unique 32-bit integer
    cid: u32,
    /// Parent ID. If ~0x0, no parent
    parent_cid: Option<u32>,
    /// Disk type
    create_type: DiskType,
    /// Extent descriptors
    extent_descriptors: Vec<ExtentDescriptor>,
    // The disk database
    // TODO
    //ddb: DiskDatabase,
}

impl Descriptor {
    pub fn new(text: &str) -> Result<Self, Error> {
        //eprintln!("Text: {}", text);
        let mut descriptor = Descriptor::default();

        for chunk in text.split("# ").filter(|x| x.len() != 0) {
            //eprintln!("Chunk: {:x?}", chunk.as_bytes());
            if chunk.starts_with("Disk DescriptorFile") {
                for line in chunk.split("\n") {
                    if line.starts_with("version=") {
                        let version = line.trim_start_matches("version=").parse::<u32>()?;
                        descriptor.version = version;
                        eprintln!("Header version: {}", version);
                    } else if line.starts_with("CID=") {
                        let cid = u32::from_str_radix(line.trim_start_matches("CID="), 16)?;
                        descriptor.cid = cid;
                        eprintln!("CID: {}", cid);
                    } else if line.starts_with("parentCID=") {
                        descriptor.parent_cid = match u32::from_str_radix(line.trim_start_matches("parentCID="), 16)? {
                            CID_NOPARENT => None,
                            c => Some(c),
                        };
                        eprintln!("Parent CID: {:?}", descriptor.parent_cid);
                    } else if line.starts_with("createType=") {
                        let disk_type = DiskType::from_str(line.trim_start_matches("createType="))?;
                        eprintln!("Disk Type: {:?}", disk_type);
                        descriptor.create_type = disk_type;
                    }
                }
            } else if chunk.starts_with("Extent description") {
                eprintln!("ExtentDescriptors: {}", chunk);
                for line in chunk.split("\n").skip(1).filter(|x| x.len() != 0) {
                    let extent_descriptor = ExtentDescriptor::from_str(line)?;
                    eprintln!("ExtentDescriptor: {:?}", extent_descriptor);
                    descriptor.extent_descriptors.push(extent_descriptor);
                }
            } else if chunk.starts_with("The disk Data Base") {
                eprintln!("Disk database: {}", chunk);
            } else {
                panic!(format!("Unsupported chunk in descriptor file: {}", chunk));
            }
        }
        //for line in text.split("\n") {
        //    if line.starts_with("version=") {
        //        eprintln!("Header version: {}", line.trim_start_matches("version="));
        //    } else if line.starts_with("CID=") {
        //        eprintln!("CID: {}", line.trim_start_matches("CID="));
        //    } else if line.starts_with("parentCID=") {
        //        eprintln!("Parent CID: {}", line.trim_start_matches("parentCID="));
        //    } else if line.starts_with("createType=") {
        //        eprintln!("Disk Type: {}", line.trim_start_matches("createType="));
        //    }
        //}
        Ok(descriptor)
    }
}

#[derive(Debug)]
pub struct ExtentDescriptor {
    extent_type: ExtentType,
    path: PathBuf,
}

impl FromStr for ExtentDescriptor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = s.split(" ").collect();
        let perms = items.get(0).ok_or_else(|| VmdkError::ParseError)?;
        let sectors = items.get(1).ok_or_else(|| VmdkError::ParseError)?.parse::<u64>()?;
        let extent_type = ExtentType::from_str(items.get(2).ok_or_else(|| VmdkError::ParseError)?)?;
        let path = PathBuf::from_str(items.get(3).ok_or_else(|| VmdkError::ParseError)?)?;
        Ok(ExtentDescriptor {
            extent_type: extent_type,
            path: path,
        })
    }
}

#[derive(Debug)]
pub enum ExtentType {
    Flat,
    Sparse,
    Zero,
    Vmfs,
    VmfsSparse,
    VmfsRdm,
    VmfsRaw,
}

impl FromStr for ExtentType {
    type Err = VmdkError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SPARSE" => Ok(Self::Sparse),
            // TODO: the rest of the types:
            // https://github.com/libyal/libvmdk/blob/master/documentation/VMWare%20Virtual%20Disk%20Format%20(VMDK).asciidoc#212-disk-type
            _ => Err(Self::Err::InvalidDisk),
        }
    }
}

#[derive(Debug)]
pub struct DiskDatabase;

#[derive(Debug, PartialEq, Eq)]
pub enum DiskType {
    MonolithicSparse,
    VmfsSparse,
    MonolithicFlat,
    Vmfs,
    TwoGbMaxExtentSparse,
    TwoGbMaxExtentFlat,
    FullDevice,
    VmfsRaw,
    PartitionedDevice,
    VmfsRawDeviceMap,
    VmfsPassthroughRawDeviceMap,
    StreamOptimized,
}

impl FromStr for DiskType {
    type Err = VmdkError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        eprintln!("Str: {}", s);
        match s {
            "\"monolithicSparse\"" => Ok(Self::MonolithicSparse),
            // TODO: the rest of the types:
            // https://github.com/libyal/libvmdk/blob/master/documentation/VMWare%20Virtual%20Disk%20Format%20(VMDK).asciidoc#212-disk-type
            _ => Err(Self::Err::InvalidDisk),
        }
    }
}

impl Default for DiskType {
    fn default() -> Self { DiskType::MonolithicSparse  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_new() {
        let descriptor_text = r#"# Disk DescriptorFile
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
        let desc = Descriptor::new(descriptor_text).expect("Failed to make a descriptor");
        assert_eq!(desc.create_type, DiskType::MonolithicSparse);
        assert_eq!(desc.cid, 0xdef0d352);
        assert_eq!(desc.parent_cid, None);
        // TODO: add rest of member tests.
    }
}
