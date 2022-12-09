use std::{fs::File, io::{self, Read, Seek}, mem::size_of};

use self::header::Header;

pub mod header;

pub struct FAT {
    header: Option<Header>,
    file: File
}

impl FAT {
    pub fn new(filename: String) -> io::Result<Self> {
        let mut file =  File::options().read(true).write(true).append(true).create(true).open(filename)?;
        let filesize = file.metadata().unwrap().len() as usize;
        
        let header = if filesize < 5 * size_of::<u32>() {
            None
        } else {
            let mut buffer = [0; 5*size_of::<u32>()];
            file.read(&mut buffer)?;
            Header::from_raw_bytes(&buffer).ok()
        };
        
        Ok(Self { header, file })
    }
}