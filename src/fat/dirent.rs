use std::mem::size_of;
use std::str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flags {
    Occupied = 1 << 0,
    Directory = 1 << 1,
    System = 1 << 2,
}

#[derive(Debug, Clone)]
pub struct Entry {
    name: String,
    size: u32,
    cluster: u32,
    flags: u32,
}

impl Entry {
    pub fn new(name: &str, size: u32, cluster: u32, flags: u32) -> Option<Self> {
        let len = name.len();

        if len > 12 {
            return None;
        }

        let mut name_bytes = [0; 12];
        name_bytes[0..len].clone_from_slice(name.as_bytes());

        Some(Self {
            name: name.to_string(),
            size,
            cluster,
            flags,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            name: str::from_utf8(
                &bytes
                    .get(0..12)?
                    .iter()
                    .filter(|c| **c != 0)
                    .cloned()
                    .collect::<Vec<u8>>(),
            )
            .ok()?
            .to_string(),
            size: u32::from_le_bytes(bytes.get(12..12 + size_of::<u32>())?.try_into().ok()?),
            cluster: u32::from_le_bytes(
                bytes
                    .get(12 + size_of::<u32>()..12 + 2 * size_of::<u32>())?
                    .try_into()
                    .ok()?,
            ),
            flags: u32::from_le_bytes(
                bytes
                    .get(12 + 2 * size_of::<u32>()..12 + 3 * size_of::<u32>())?
                    .try_into()
                    .ok()?,
            ),
        })
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn name(&self) -> &str {
        // unwrap should never fail
        &self.name
    }

    pub fn cluster(&self) -> u32 {
        self.cluster
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn set_name(&mut self, name: &str) -> Option<()> {
        let len = name.len();
        if len > 12 {
            return None;
        }

        self.name = name.to_string();

        Some(())
    }

    pub fn set_cluster(&mut self, cluster: u32) {
        self.cluster = cluster;
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        let mut v = [0; 32];

        let name_len = self.name.len();

        v[0..name_len].clone_from_slice(&self.name.as_bytes());
        v[12..12 + size_of::<u32>()].clone_from_slice(&u32::to_le_bytes(self.size));
        v[12 + size_of::<u32>()..12 + 2 * size_of::<u32>()]
            .clone_from_slice(&u32::to_le_bytes(self.cluster));
        v[12 + 2 * size_of::<u32>()..12 + 3 * size_of::<u32>()]
            .clone_from_slice(&u32::to_le_bytes(self.flags));

        v
    }
}
