use crate::units::Unit;
use std::{cmp::Ordering, fmt::Display};

#[derive(Debug, Clone)]
pub struct Header {
    bytes_per_sector: u32,
    sectors_per_cluster: u32,
    sector_count: u32,
    fat_count: u32,
    checksum: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum HeaderError {
    BadCapacity,
    BadChecksum,
    BadBytes,
    CannotFormat,
}

const BYTES_PER_SECTOR: u32 = 512;
const SECTORS_PER_CLUSTER: u32 = 8;

impl Header {
    fn capacity_to_sector_count(capacity: usize) -> u32 {
        capacity as u32 / BYTES_PER_SECTOR
    }

    fn update_checksum(&mut self) {
        self.checksum = u32::MAX
            - (self.bytes_per_sector
                + self.sectors_per_cluster
                + self.sector_count
                + self.fat_count)
            + 1;
    }

    pub fn new(capacity: Unit) -> Result<Self, HeaderError> {
        let capacity = capacity.to_bytes();
        if capacity % 512 != 0 {
            return Err(HeaderError::BadCapacity);
        }

        let sector_count = Self::capacity_to_sector_count(capacity);
        
        let mut fat = Self {
            bytes_per_sector: BYTES_PER_SECTOR,
            sectors_per_cluster: SECTORS_PER_CLUSTER,
            sector_count,
            fat_count: 2,
            checksum: 0,
        };

        fat.update_checksum();
        Ok(fat)
    }

    fn check_checksum(&self) -> Result<(), HeaderError> {
        let sum = 
        self
            .checksum
            .wrapping_add(self.bytes_per_sector)
            .wrapping_add(self.sectors_per_cluster)
            .wrapping_add(self.sector_count)
            .wrapping_add(self.fat_count);
        if sum == 0 { Ok(()) } else { Err(HeaderError::BadChecksum) }
    }

    pub fn from_raw_bytes(bytes: &[u8]) -> Result<Self, HeaderError> {
        use std::mem::size_of;

        let u32_size = size_of::<u32>();

        if bytes.len().cmp(&(5 * u32_size)) != Ordering::Equal {
            return Err(HeaderError::BadBytes);
        }
        let bytes_per_sector = u32::from_le_bytes(bytes[0..u32_size].try_into().unwrap());
        let sectors_per_cluster= u32::from_le_bytes(bytes[u32_size..2 * u32_size].try_into().unwrap());
        let sector_count = u32::from_le_bytes(bytes[2 * u32_size..3 * u32_size].try_into().unwrap());
        let fat_count = u32::from_le_bytes(bytes[3 * u32_size..4 * u32_size].try_into().unwrap());
        let checksum = u32::from_le_bytes(bytes[4 * u32_size..5 * u32_size].try_into().unwrap());
        
        let fat = Self {
            bytes_per_sector,
            sectors_per_cluster,
            sector_count,
            fat_count,
            checksum,
        };

        fat.check_checksum()?;
        Ok(fat)
    }

    pub fn bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector
    }

    pub fn sectors_per_cluster(&self) -> u32 {
        self.sectors_per_cluster
    }

    pub fn sector_count(&self) -> u32 {
        self.sector_count
    }

    pub fn fat_count(&self) -> u32 {
        self.fat_count
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FAT Info:\nBytes per sector: {}\nSectors per cluster: {}\nSector count: {}\nNumber of FATs: {}\n", self.bytes_per_sector, self.sectors_per_cluster, self.sector_count, self.fat_count)
    }
}
