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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
