use std::{
    collections::{hash_map::IntoIter, HashMap},
    mem::size_of,
};

pub struct FATManager {
    fat_sectors: HashMap<u32, [u32; 128]>,
    clusters_per_fat_sector: u32,
}

impl FATManager {
    pub fn new() -> Self {
        Self {
            fat_sectors: HashMap::new(),
            clusters_per_fat_sector: 512 / size_of::<u32>() as u32,
        }
    }

    pub fn contains_cluster(&self, cluster: u32) -> bool {
        let map_index = cluster / self.clusters_per_fat_sector;
        self.fat_sectors.contains_key(&map_index)
    }

    pub fn add_cluster(&mut self, cluster: u32, sector: [u32; 128]) {
        let map_index = cluster / self.clusters_per_fat_sector;
        self.fat_sectors.insert(map_index, sector).map(|_| ());
    }

    pub fn get_cluster_value(&self, cluster: u32) -> Option<u32> {
        let map_index = cluster / self.clusters_per_fat_sector;
        let fat_index = (cluster % self.clusters_per_fat_sector) as usize;
        self.fat_sectors.get(&map_index)?.get(fat_index).cloned()
    }

    pub fn set_cluster_value(&mut self, cluster: u32, value: u32) -> Option<()> {
        let map_index = cluster / self.clusters_per_fat_sector;
        let fat_index = (cluster % self.clusters_per_fat_sector) as usize;
        *self.fat_sectors.get_mut(&map_index)?.get_mut(fat_index)? = value;
        Some(())
    }

    pub fn flush(self) -> IntoIter<u32, [u32; 128]> {
        self.fat_sectors.into_iter()
    }
}
