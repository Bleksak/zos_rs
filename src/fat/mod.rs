use std::{
    collections::HashSet,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use crate::{fat::dirent::Flags, units::Unit};

use self::{
    dirent::Entry,
    fatmanager::FATManager,
    header::{Header, HeaderError},
};

pub mod dirent;
mod fatmanager;
pub mod header;

pub struct FAT {
    header: Option<Header>,
    file: File,
}

static EMPTY_CLUSTER: [u8; 8192] = [0; 8192];
static FAT_READ_DONE: u32 = 0xFFFFFFFF;
static FAT_BAD_CLUSTER: u32 = 0xFFFFFFFE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FATError {
    FilenameTooLong,
    FileNotFound,
    CannotRead,
    CannotWrite,
    NotEnoughSpace,
    FileExists,
    DirNotEmpty,
}

impl FAT {
    pub fn new(filename: String) -> io::Result<Self> {
        let mut file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;
        let filesize = file.metadata().unwrap().len() as usize;

        let header = if filesize < 5 * size_of::<u32>() {
            None
        } else {
            let mut buffer = [0; 5 * size_of::<u32>()];
            file.read_exact(&mut buffer)?;
            Header::from_raw_bytes(&buffer).ok()
        };

        Ok(Self { header, file })
    }

    fn dealloc_clusters(&mut self, mut cluster: u32) -> Option<()> {
        let mut manager = FATManager::new();

        while cluster != Self::mark_read_done() {
            if !manager.contains_cluster(cluster) {
                manager.add_cluster(cluster, self.read_fat(cluster)?);
            }

            manager.set_cluster_value(cluster, 0);

            cluster = self.next_cluster(cluster)?;
            if cluster == Self::mark_bad_cluster() {
                return None;
            }
        }

        for (cluster, value) in manager.flush() {
            self.write_fat(cluster * (512 / size_of::<u32>() as u32), value)?;
        }

        Some(())
    }

    fn allocate_clusters(&mut self, mut count: u32) -> Result<u32, FATError> {
        let mut begin_cluster = 0;
        let header = self.header.as_ref().expect("Filesystem is not formatted!");

        let cluster_count = header.sector_count() / header.sectors_per_cluster();

        let mut manager = FATManager::new();

        let mut prev_cluster = 0;
        let mut current_cluster = 0;

        loop {
            if !manager.contains_cluster(current_cluster) {
                manager.add_cluster(
                    current_cluster,
                    self.read_fat(current_cluster).ok_or(FATError::CannotRead)?,
                );
            }

            let current_cluster_value = manager.get_cluster_value(current_cluster).unwrap();

            if current_cluster_value == 0 {
                if begin_cluster == 0 {
                    begin_cluster = current_cluster;
                }
                if prev_cluster == 0 {
                    prev_cluster = current_cluster;
                } else {
                    if !manager.contains_cluster(prev_cluster) {
                        manager.add_cluster(
                            prev_cluster,
                            self.read_fat(prev_cluster).ok_or(FATError::CannotRead)?,
                        );
                    }

                    manager.set_cluster_value(prev_cluster, current_cluster);
                    prev_cluster = current_cluster;
                    count -= 1;
                }

                if count == 1 {
                    manager.set_cluster_value(current_cluster, Self::mark_read_done());
                    for (cluster, value) in manager.flush() {
                        self.write_fat(cluster * (512 / size_of::<u32>() as u32), value)
                            .ok_or(FATError::CannotWrite)?;
                    }

                    return Ok(begin_cluster);
                }
            }

            current_cluster += 1;
            if current_cluster == cluster_count {
                return Err(FATError::NotEnoughSpace);
            }
        }
        // Ok(0)
    }

    fn empty_cluster() -> &'static [u8; 8192] {
        &EMPTY_CLUSTER
    }

    fn mark_read_done() -> u32 {
        FAT_READ_DONE
    }

    fn mark_bad_cluster() -> u32 {
        FAT_BAD_CLUSTER
    }

    fn sector_to_byte(&self, sector: u64) -> u64 {
        sector
            * self
                .header
                .as_ref()
                .expect("Image is not formatted!")
                .bytes_per_sector() as u64
    }

    fn first_data_sector(&self) -> u64 {
        let header = self.header.as_ref().expect("Image is not formatted!");
        1 + (header.fat_count() * (header.sector_count() / header.sectors_per_cluster())
            / (header.bytes_per_sector() / size_of::<u32>() as u32)) as u64
    }

    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let header = self.header.as_ref().expect("Image is not formatted!");
        self.first_data_sector() + ((cluster - 1) * header.sectors_per_cluster()) as u64
    }

    fn read_sector(&mut self, sector: u64) -> Option<[u8; 512]> {
        let mut buf = [0; 512];
        self.file
            .seek(SeekFrom::Start(self.sector_to_byte(sector)))
            .ok()?;
        self.file.read(&mut buf).ok()?;
        Some(buf)
    }

    fn write_sector(&mut self, sector: u64, bytes: [u8; 512]) -> Option<()> {
        self.file
            .seek(SeekFrom::Start(self.sector_to_byte(sector)))
            .ok()?;
        self.file.write(&bytes).ok()?;
        Some(())
    }

    fn read_cluster(&mut self, cluster: u32) -> Option<[u8; 4096]> {
        let mut buf = [0; 4096];
        self.file
            .seek(SeekFrom::Start(
                self.sector_to_byte(self.cluster_to_sector(cluster)),
            ))
            .ok()?;
        self.file.read(&mut buf).ok()?;
        Some(buf)
    }

    fn write_cluster(&mut self, cluster: u32, bytes: [u8; 4096]) -> Option<()> {
        self.file
            .seek(SeekFrom::Start(
                self.sector_to_byte(self.cluster_to_sector(cluster)),
            ))
            .ok()?;
        self.file.write(&bytes).ok()?;
        Some(())
    }

    fn read_cluster_entries(&mut self, cluster: u32) -> Option<Vec<Entry>> {
        let bytes = self.read_cluster(cluster)?;
        let mut v = vec![];

        for i in (0..4096).step_by(32) {
            v.push(Entry::from_bytes(&bytes[i..i + 32]).unwrap());
        }

        Some(v)
    }

    fn read_fat(&mut self, cluster: u32) -> Option<[u32; 512 / size_of::<u32>()]> {
        let sector = 1 + cluster / (512 / size_of::<u32>() as u32);
        let sector = self.read_sector(sector as u64)?;

        let mut fat: [u32; 512 / size_of::<u32>()] = [0; 512 / size_of::<u32>()];

        for (data, res) in std::iter::zip(sector.chunks(4), fat.iter_mut()) {
            *res = u32::from_le_bytes(data.try_into().unwrap());
        }

        Some(fat)
    }

    fn write_fat(&mut self, cluster: u32, fat: [u32; 512 / size_of::<u32>()]) -> Option<()> {
        let sector = 1 + cluster / (512 / size_of::<u32>() as u32);

        let mut bytes: [u8; 512] = [0; 512];

        for (data, res) in std::iter::zip(fat.iter(), bytes.chunks_mut(4)) {
            res.clone_from_slice(&u32::to_le_bytes(*data));
        }

        self.write_sector(sector as u64, bytes)
    }

    fn next_cluster(&mut self, cluster: u32) -> Option<u32> {
        let fat = self.read_fat(cluster)?;
        Some(fat[(cluster as usize % (512 / size_of::<u32>()))])
    }

    fn write_cluster_entries(&mut self, cluster: u32, entries: &Vec<Entry>) -> Option<()> {
        let mut bytes = [0; 4096];

        for i in (0..4096).step_by(32) {
            bytes[i..i + 32].clone_from_slice(&entries[i / 32].as_bytes());
        }

        self.write_cluster(cluster, bytes)
    }

    pub fn update_file_in_dir<F: Fn(&Entry) -> bool, U: Fn(&mut Entry)>(
        &mut self,
        dir: &Entry,
        filter: F,
        update: U,
    ) -> Result<Entry, FATError> {
        let mut cluster = dir.cluster();

        loop {
            let mut entries = self
                .read_cluster_entries(cluster)
                .ok_or(FATError::CannotRead)?;

            for entry in entries.iter_mut() {
                if filter(entry) {
                    let cloned = entry.clone();
                    update(entry);
                    self.write_cluster_entries(cluster, &entries);
                    return Ok(cloned);
                }
            }

            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }
    }

    pub fn find_file(&mut self, path: &str, filter: fn(&Entry) -> bool) -> Result<Entry, FATError> {
        let mut it = path.split('/').peekable();
        let mut current_cluster = 1;

        'outer: while let Some(item) = it.next() {
            let len = item.len();

            if len > 12 {
                return Err(FATError::FilenameTooLong);
            }

            loop {
                let mut entries = self
                    .read_cluster_entries(current_cluster)
                    .ok_or(FATError::CannotRead)?;
                for entry in entries.iter_mut() {
                    if entry.name() == item {
                        if it.peek().is_none() {
                            if filter(&entry) {
                                return Ok(entry.clone());
                            }
                        } else if entry.flags() & (Flags::Occupied as u32 | Flags::Directory as u32)
                            == Flags::Occupied as u32 | Flags::Directory as u32
                        {
                            current_cluster = entry.cluster();
                            continue 'outer;
                        }
                    }
                }

                current_cluster = self
                    .next_cluster(current_cluster)
                    .ok_or(FATError::CannotRead)?;
                if current_cluster == Self::mark_read_done() {
                    return Err(FATError::FileNotFound);
                }

                if current_cluster == Self::mark_bad_cluster() {
                    return Err(FATError::CannotRead);
                }
            }
        }

        Err(FATError::FileNotFound)
    }

    pub fn filter_ls(entry: &Entry) -> bool {
        entry.flags() & (Flags::Occupied as u32 | Flags::Directory as u32)
            == Flags::Occupied as u32 | Flags::Directory as u32
    }

    pub fn listings(&mut self, path: &str) -> Result<(), FATError> {
        let dir = self.find_file(&path, FAT::filter_ls)?;

        let mut current_cluster = dir.cluster();

        while current_cluster != Self::mark_read_done() {
            let entries = self
                .read_cluster_entries(current_cluster)
                .ok_or(FATError::CannotRead)?;
            for entry in entries {
                if entry.flags() & Flags::Occupied as u32 == Flags::Occupied as u32 {
                    let spec = if entry.flags() & Flags::Directory as u32 == Flags::Directory as u32
                    {
                        "DIR"
                    } else {
                        "FILE"
                    };
                    println!("{spec}: {}", entry.name());
                }
            }

            current_cluster = self
                .next_cluster(current_cluster)
                .ok_or(FATError::CannotRead)?;

            if current_cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Ok(())
    }

    pub fn filter_mkdir(entry: &Entry) -> bool {
        entry.flags() & (Flags::Occupied as u32 | Flags::Directory as u32)
            == Flags::Occupied as u32 | Flags::Directory as u32
    }

    pub fn filter_find(entry: &Entry) -> bool {
        entry.flags() & Flags::Occupied as u32 == Flags::Occupied as u32
    }

    pub fn filter_find_file(entry: &Entry) -> bool {
        entry.flags() & (Flags::Occupied as u32 | Flags::Directory as u32) == Flags::Occupied as u32
    }

    fn split_path(path: &str) -> (&str, &str) {
        path.rsplit_once('/').unwrap_or((".", path))
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), FATError> {
        let (dir, filename) = Self::split_path(path);

        if self.find_file(path, Self::filter_find).is_ok() {
            return Err(FATError::FileExists);
        }

        let entry = self.find_file(dir, Self::filter_mkdir)?;

        let mut new_entry = Entry::new(
            filename,
            0,
            0,
            Flags::Occupied as u32 | Flags::Directory as u32,
        )
        .ok_or(FATError::FilenameTooLong)?;

        let mut current_cluster = entry.cluster();

        while current_cluster != Self::mark_read_done() {
            let mut dirents = self
                .read_cluster_entries(current_cluster)
                .ok_or(FATError::CannotRead)?;
            for dirent in dirents.iter_mut() {
                if dirent.flags() & Flags::Occupied as u32 == 0 {
                    let cluster = self.allocate_clusters(1)?;
                    new_entry.set_cluster(cluster);

                    self.write_cluster(cluster, FAT::empty_cluster()[0..4096].try_into().unwrap())
                        .ok_or(FATError::CannotWrite)?;
                    let mut entries = self
                        .read_cluster_entries(cluster)
                        .ok_or(FATError::CannotRead)?;

                    entries[0] = Entry::new(
                        ".",
                        0,
                        new_entry.cluster(),
                        Flags::Occupied as u32 | Flags::Directory as u32 | Flags::System as u32,
                    )
                    .unwrap();
                    entries[1] = Entry::new(
                        "..",
                        0,
                        entry.cluster(),
                        Flags::Occupied as u32 | Flags::Directory as u32 | Flags::System as u32,
                    )
                    .unwrap();

                    self.write_cluster_entries(cluster, &entries)
                        .ok_or(FATError::CannotWrite)?;

                    *dirent = new_entry;
                    self.write_cluster_entries(current_cluster, &dirents)
                        .ok_or(FATError::CannotWrite)?;
                    return Ok(());
                }
            }

            current_cluster = self
                .next_cluster(current_cluster)
                .ok_or(FATError::CannotRead)?;

            if current_cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Err(FATError::NotEnoughSpace)
    }

    pub fn new_file<T: Read + Seek>(&mut self, path: &str, mut infile: T) -> Result<(), FATError> {
        let file_size = infile
            .seek(SeekFrom::End(0))
            .map_err(|_| FATError::CannotRead)?;
        infile.rewind().map_err(|_| FATError::CannotRead)?;

        let (dir, filename) = Self::split_path(path);

        if self.find_file(path, Self::filter_find).is_ok() {
            return Err(FATError::FileExists);
        }

        let dir = self.find_file(dir, Self::filter_mkdir)?;
        let mut new_entry = Entry::new(filename, file_size as u32, 0, Flags::Occupied as u32)
            .ok_or(FATError::FilenameTooLong)?;

        let mut current_cluster = dir.cluster();

        while current_cluster != Self::mark_read_done() {
            let mut dirents = self
                .read_cluster_entries(current_cluster)
                .ok_or(FATError::CannotRead)?;
            for dirent in dirents.iter_mut() {
                if dirent.flags() & Flags::Occupied as u32 == 0 {
                    let cluster_size = (self.header.as_ref().unwrap().sectors_per_cluster()
                        * self.header.as_ref().unwrap().bytes_per_sector())
                        as u64;
                    let rem = file_size % cluster_size;
                    let cluster_count = file_size / cluster_size + if rem == 0 { 0 } else { 1 };
                    let mut cluster = self.allocate_clusters(cluster_count as u32)?;
                    new_entry.set_cluster(cluster);

                    loop {
                        let mut buffer = vec![0; cluster_size as usize];
                        let n = infile.read(&mut buffer).map_err(|_| FATError::CannotRead)?;

                        if n == 0 {
                            *dirent = new_entry;
                            self.write_cluster_entries(current_cluster, &dirents)
                                .ok_or(FATError::CannotWrite)?;
                            return Ok(());
                        }

                        self.write_cluster(cluster, buffer[..].try_into().unwrap())
                            .ok_or(FATError::CannotWrite)?;
                        cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
                    }
                }
            }

            current_cluster = self
                .next_cluster(current_cluster)
                .ok_or(FATError::CannotRead)?;

            if current_cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Err(FATError::NotEnoughSpace)
    }

    pub fn cat<T: Write>(&mut self, path: &str, mut outfile: T) -> Result<(), FATError> {
        let entry = self.find_file(path, Self::filter_find_file)?;

        let mut size = entry.size();
        let mut cluster = entry.cluster();

        while cluster != Self::mark_read_done() {
            let limit = size.min(4096);
            let bytes = self.read_cluster(cluster).ok_or(FATError::CannotRead)?;
            outfile
                .write(&bytes[0..limit as usize])
                .map_err(|_| FATError::CannotWrite)?;

            size -= limit;

            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Ok(())
    }

    pub fn info(&mut self, path: &str) -> Result<(), FATError> {
        let entry = self.find_file(path, Self::filter_find)?;

        let mut cluster = entry.cluster();
        let mut clusters = vec![];

        while cluster != Self::mark_read_done() {
            clusters.push(cluster);
            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        println!(
            "{} {}",
            entry.name(),
            clusters
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(())
    }

    fn is_empty(&mut self, entry: &Entry) -> Result<bool, FATError> {
        let mut cluster = entry.cluster();
        while cluster != Self::mark_read_done() {
            let mut entries = self
                .read_cluster_entries(cluster)
                .ok_or(FATError::CannotRead)?;

            for entry in entries.iter_mut() {
                if entry.name() == "." || entry.name() == ".." {
                    continue;
                }

                if entry.flags() & Flags::Occupied as u32 == Flags::Occupied as u32 {
                    return Ok(false);
                }
            }

            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Ok(true)
    }

    fn remove(&mut self, path: &str, flags: u32) -> Result<(), FATError> {
        let (dir, filename) = Self::split_path(path);
        let dir = self.find_file(dir, Self::filter_mkdir)?;

        let mut current_cluster = dir.cluster();

        while current_cluster != Self::mark_read_done() {
            let mut entries = self
                .read_cluster_entries(current_cluster)
                .ok_or(FATError::CannotRead)?;

            for entry in entries.iter_mut() {
                if entry.name() == filename && entry.flags() == flags {
                    if flags & Flags::Directory as u32 == Flags::Directory as u32
                        && !self.is_empty(entry)?
                    {
                        return Err(FATError::DirNotEmpty);
                    }

                    entry.set_flags(0);
                    self.dealloc_clusters(entry.cluster());
                    self.write_cluster_entries(current_cluster, &entries);
                    return Ok(());
                }
            }

            current_cluster = self
                .next_cluster(current_cluster)
                .ok_or(FATError::CannotRead)?;
            if current_cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Err(FATError::FileNotFound)
    }

    pub fn remove_file(&mut self, path: &str) -> Result<(), FATError> {
        self.remove(path, Flags::Occupied as u32)
    }

    pub fn remove_dir(&mut self, path: &str) -> Result<(), FATError> {
        self.remove(path, Flags::Occupied as u32 | Flags::Directory as u32)
    }

    pub fn move_file(&mut self, source: &str, dest: &str) -> Result<(), FATError> {
        if self.find_file(dest, Self::filter_find).is_ok() {
            return Err(FATError::FileExists);
        }

        if self.find_file(source, Self::filter_find_file).is_err() {
            return Err(FATError::FileNotFound);
        }

        let (dir1, file1) = Self::split_path(source);
        let (dir2, file2) = Self::split_path(dest);

        let dir_src = self.find_file(dir1, Self::filter_mkdir)?;
        let dir_dest = self.find_file(dir2, Self::filter_mkdir)?;

        let mut entry = self.update_file_in_dir(
            &dir_src,
            |entry| entry.name() == file1 && entry.flags() == Flags::Occupied as u32,
            |entry| entry.set_flags(0),
        )?;
        entry.set_name(file2).ok_or(FATError::FilenameTooLong)?;
        self.update_file_in_dir(
            &dir_dest,
            |entry| entry.flags() & Flags::Occupied as u32 == 0,
            |update| *update = entry.clone(),
        )?;

        Ok(())
    }

    pub fn copy(&mut self, source: &str, dest: &str) -> Result<(), FATError> {
        if self.find_file(dest, Self::filter_find).is_ok() {
            return Err(FATError::FileExists);
        }

        let entry = self.find_file(source, Self::filter_find_file)?;

        let cluster_size = self.header.as_ref().unwrap().sectors_per_cluster()
            * self.header.as_ref().unwrap().bytes_per_sector();
        let rem = entry.size() % cluster_size;

        let cluster_count = entry.size() / cluster_size + if rem == 0 { 0 } else { 1 };

        let (dir, filename) = Self::split_path(dest);

        let new_file_dir_entry = self.find_file(dir, Self::filter_mkdir)?;

        let mut new_entry = Entry::new(filename, entry.size(), 0, Flags::Occupied as u32)
            .ok_or(FATError::FilenameTooLong)?;
        let mut cluster = new_file_dir_entry.cluster();

        while cluster != Self::mark_read_done() {
            let mut entries = self
                .read_cluster_entries(cluster)
                .ok_or(FATError::CannotRead)?;
            for dirent in entries.iter_mut() {
                if dirent.flags() & Flags::Occupied as u32 == 0 {
                    let alloc = self
                        .allocate_clusters(cluster_count)
                        .map_err(|_| FATError::CannotRead)?;
                    new_entry.set_cluster(alloc);
                    *dirent = new_entry;

                    let mut cluster_a = alloc;
                    let mut cluster_b = entry.cluster();

                    while cluster_a != Self::mark_read_done() || cluster_b != Self::mark_read_done()
                    {
                        let cluster = self.read_cluster(cluster_b).ok_or(FATError::CannotRead)?;
                        self.write_cluster(cluster_a, cluster)
                            .ok_or(FATError::CannotWrite)?;

                        cluster_a = self.next_cluster(cluster_a).ok_or(FATError::CannotRead)?;
                        cluster_b = self.next_cluster(cluster_b).ok_or(FATError::CannotRead)?;
                    }

                    self.write_cluster_entries(cluster, &entries)
                        .ok_or(FATError::CannotRead)?;

                    return Ok(());
                }
            }

            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
        }

        Err(FATError::FileNotFound)
    }

    pub fn set_cluster_value(&mut self, cluster: u32, value: u32) -> Option<()> {
        let mut fat = self.read_fat(cluster)?;
        let index = cluster as usize % (512 / size_of::<u32>());
        fat[index] = value;
        self.write_fat(cluster, fat)
    }

    pub fn bug(&mut self, path: &str) -> Result<(), FATError> {
        let file = self.find_file(path, Self::filter_find_file)?;

        let mut cluster = file.cluster();
        let last_cluster;

        loop {
            let next_cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;
            if next_cluster == Self::mark_read_done() {
                last_cluster = cluster;
                break;
            }

            if next_cluster == Self::mark_bad_cluster() {
                return Err(FATError::CannotRead);
            }
            cluster = next_cluster;
        }

        self.set_cluster_value(last_cluster, file.cluster());
        Ok(())
    }

    fn check_entry(&mut self, entry: &Entry, tabs: usize) -> Result<(), FATError> {
        let mut cluster = entry.cluster();
        let tabs_str = (0..tabs).map(|_| "\t").collect::<Vec<_>>().join("");
        println!("{tabs_str}{}", entry.name());
        if entry.flags() & Flags::Directory as u32 == Flags::Directory as u32 && entry.size() != 0 {
            println!("{tabs_str} is a directory with size != 0");
        }

        let mut visited = HashSet::new();

        while cluster != Self::mark_read_done() {
            if visited.contains(&cluster) {
                println!("{tabs_str} FAT contains a cycle! Cannot continue.");
                return Ok(());
            }

            visited.insert(cluster);

            if entry.flags() & Flags::Directory as u32 == Flags::Directory as u32 {
                let entries = self
                    .read_cluster_entries(cluster)
                    .ok_or(FATError::CannotRead)?;
                for dirent in entries {
                    if dirent.flags() & Flags::Occupied as u32 == Flags::Occupied as u32
                        && dirent.name() != "."
                        && dirent.name() != ".."
                    {
                        self.check_entry(&dirent, tabs + 1)?;
                    }
                }
            }

            cluster = self.next_cluster(cluster).ok_or(FATError::CannotRead)?;

            if cluster == Self::mark_bad_cluster() {
                println!("{tabs_str}  FAT contains bad sector(s)! Cannot continue.");
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn check(&mut self) -> Result<(), FATError> {
        let entry = Entry::new("/", 0, 1, Flags::Directory as u32).unwrap();
        self.check_entry(&entry, 0)
    }

    fn write_header(&mut self) -> Option<()> {
        self.file.rewind().ok()?;

        let header = self.header.as_ref().unwrap();

        self.file.write(&header.bytes_per_sector().to_le_bytes()).ok()?;
        self.file.write(&header.sectors_per_cluster().to_le_bytes()).ok()?;
        self.file.write(&header.sector_count().to_le_bytes()).ok()?;
        self.file.write(&header.fat_count().to_le_bytes()).ok()?;
        self.file.write(&header.checksum().to_le_bytes()).ok()?;

        let cluster_count = header.sector_count() / header.sectors_per_cluster();

        let fat_sectors = 1 + size_of::<u32>() as u32 * cluster_count / header.bytes_per_sector();

        self.file
            .seek(SeekFrom::Start(header.bytes_per_sector() as u64))
            .ok()?;
        for _ in 0..header.sector_count() - 1 {
            self.file
                .write(&FAT::empty_cluster()[0..header.bytes_per_sector() as usize])
                .ok()?;
        }

        self.file
            .seek(SeekFrom::Start(header.bytes_per_sector() as u64))
            .ok()?;
        self.file
            .write(&FAT::mark_bad_cluster().to_le_bytes())
            .ok()?;
        self.file.write(&FAT::mark_read_done().to_le_bytes()).ok()?;

        self.file
            .seek(SeekFrom::Start(
                ((1 + fat_sectors) * header.bytes_per_sector()) as u64,
            ))
            .ok()?;
        self.file
            .write(&FAT::mark_bad_cluster().to_le_bytes())
            .ok()?;
        self.file.write(&FAT::mark_read_done().to_le_bytes()).ok()?;

        let mut entries = self.read_cluster_entries(1)?;
        entries[0] = Entry::new(
            ".",
            0,
            1,
            Flags::Occupied as u32 | Flags::Directory as u32 | Flags::System as u32,
        )
        .unwrap();
        entries[1] = Entry::new(
            "..",
            0,
            1,
            Flags::Occupied as u32 | Flags::Directory as u32 | Flags::System as u32,
        )
        .unwrap();
        self.write_cluster_entries(1, &entries)?;

        self.file.flush().ok()
    }

    pub fn format(&mut self, capacity: Unit) -> Result<(), HeaderError> {
        let header = Header::new(capacity)?;
        self.header = Some(header);
        self.write_header().ok_or(HeaderError::CannotFormat)?;
        Ok(())
    }
}
