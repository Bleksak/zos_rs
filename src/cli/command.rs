use std::{
    fmt::Display,
    fs::{self, read_to_string, File},
};

use crate::{
    fat::{dirent::Flags, FATError},
    units::Unit,
    Application,
};

use super::get;

#[derive(Debug, Clone)]
pub enum CommandError {
    FileNotFound,
    PathNotFound,
    Exist,
    NotEmpty,
    CannotCreateFile,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Self::FileNotFound => "FILE NOT FOUND",
                Self::PathNotFound => "PATH NOT FOUND",
                Self::Exist => "EXIST",
                Self::NotEmpty => "NOT EMPTY",
                Self::CannotCreateFile => "CANNOT CREATE FILE",
            }
        )
    }
}

fn build_path(current_path: &String, given_path: Option<&String>) -> String {
    if let Some(given_path) = given_path {
        if given_path.starts_with('/') {
            given_path[1..].to_string()
        } else {
            let len = if given_path.is_empty() {
                current_path.len() - 1
            } else {
                current_path.len()
            };
            current_path[1..len].to_string() + given_path
        }
    } else {
        current_path[1..].to_string()
    }
}

pub trait CommandHandler {
    type Error;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error>;
}
//     1) Zkopíruje soubor s1 do umístění s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
// cp s1 s2
pub struct CopyFile(String, String);

impl CopyFile {
    pub fn new(source: String, destination: String) -> Self {
        Self(source, destination)
    }
}

impl CommandHandler for CopyFile {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .copy(
                &build_path(&application.current_path, Some(&self.0)),
                &build_path(&application.current_path, Some(&self.1)),
            )
            .map_err(|_| CommandError::FileNotFound)
    }
}
// 2) Přesune soubor s1 do umístění s2, nebo přejmenuje s1 na s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
pub struct MoveFile(String, String);
impl MoveFile {
    pub fn new(source: String, destination: String) -> Self {
        Self(source, destination)
    }
}

impl CommandHandler for MoveFile {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .move_file(
                &build_path(&application.current_path, Some(&self.0)),
                &build_path(&application.current_path, Some(&self.1)),
            )
            .map_err(|e| match e {
                _ => CommandError::FileNotFound,
            })
    }
}
// 3) Smaže soubor s1
// rm s1
// Možný výsledek:
// OK
// FILE NOT FOUND
pub struct RemoveFile(String);
impl RemoveFile {
    pub fn new(file: String) -> Self {
        Self(file)
    }
}

impl CommandHandler for RemoveFile {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .remove_file(&build_path(&application.current_path, Some(&self.0)))
            .map_err(|_| CommandError::FileNotFound)
    }
}
// 4) Vytvoří adresář a1
// mkdir a1
// Možný výsledek:
// OK
// PATH NOT FOUND (neexistuje zadaná cesta)
// EXIST (nelze založit, již existuje)
pub struct MakeDirectory(String);
impl MakeDirectory {
    pub fn new(dirname: String) -> Self {
        Self(dirname)
    }
}

impl CommandHandler for MakeDirectory {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let path = build_path(&application.current_path, Some(&self.0));

        application.file_system.mkdir(&path).map_err(|e| match e {
            FATError::FileExists => CommandError::Exist,
            _ => CommandError::PathNotFound,
        })
    }
}
// 5) Smaže prázdný adresář a1
// rmdir a1
// Možný výsledek:
// OK
// FILE NOT FOUND (neexistující adresář)
// NOT EMPTY (adresář obsahuje podadresáře, nebo soubory)
pub struct RemoveDirectory(String);
impl RemoveDirectory {
    pub fn new(dirname: String) -> Self {
        Self(dirname)
    }
}

impl CommandHandler for RemoveDirectory {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .remove_dir(&build_path(&application.current_path, Some(&self.0)))
            .map_err(|e| match e {
                FATError::DirNotEmpty => CommandError::NotEmpty,
                _ => CommandError::FileNotFound,
            })
    }
}
// 6) Vypíše obsah adresáře a1, bez parametru vypíše obsah aktuálního adresáře
// ls a1
// ls
// Možný výsledek:
// FILE: f1
// DIR: a2
// PATH NOT FOUND (neexistující adresář)
pub struct Listing(Option<String>);
impl Listing {
    pub fn new(dirname: Option<String>) -> Self {
        Self(dirname)
    }
}

impl CommandHandler for Listing {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let mut path = build_path(&application.current_path, self.0.as_ref());

        if path.ends_with("/") || path.is_empty() {
            path.push('.');
        }
        application
            .file_system
            .listings(&path)
            .map_err(|_| CommandError::FileNotFound)
    }
}
// 7) Vypíše obsah souboru s1
// cat s1
// Možný výsledek:
// OBSAH
// FILE NOT FOUND (není zdroj)
pub struct Concatenate(String);
impl Concatenate {
    pub fn new(dirname: String) -> Self {
        Self(dirname)
    }
}

impl CommandHandler for Concatenate {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .cat(
                &build_path(&application.current_path, Some(&self.0)),
                std::io::stdout(),
            )
            .map_err(|e| match e {
                FATError::FileExists => CommandError::Exist,
                _ => CommandError::PathNotFound,
            })
    }
}
// 8) Změní aktuální cestu do adresáře a1
// cd a1
// Možný výsledek:
// OK
// PATH NOT FOUND (neexistující cesta)
pub struct ChangeDirectory(String);
impl ChangeDirectory {
    pub fn new(dirname: String) -> Self {
        Self(dirname)
    }
}
impl CommandHandler for ChangeDirectory {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let path = build_path(&application.current_path, Some(&self.0));

        if application
            .file_system
            .find_file(&path, |entry| {
                entry.flags() & (Flags::Occupied as u32 | Flags::Directory as u32)
                    == Flags::Occupied as u32 | Flags::Directory as u32
            })
            .is_err()
        {
            return Err(CommandError::PathNotFound);
        }

        let mut v = vec![];

        let mut it = path.split('/').peekable();
        while let Some(item) = it.next() {
            if let Some(next) = it.peek() {
                if *next == ".." {
                    continue;
                }
            }

            if item == ".." || item == "." {
                continue;
            }

            v.push(item.to_string() + "/");
        }

        let path = v.join("");

        application.current_path = "/".to_string() + &path;

        Ok(())
    }
}
// 9) Vypíše aktuální cestu
// pwd
// Možný výsledek:
// PATH
pub struct PrintWorkingDirectory;
impl PrintWorkingDirectory {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for PrintWorkingDirectory {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        println!("{}", application.current_path);
        Ok(())
    }
}
// 10) Vypíše informace o souboru/adresáři s1/a1 (v jakých clusterech se nachází)
// info a1/s1
// Možný výsledek:
// S1 2,3,4,7,10
// FILE NOT FOUND (není zdroj)
pub struct PrintInfo(String);
impl PrintInfo {
    pub fn new(file: String) -> Self {
        Self(file)
    }
}

impl CommandHandler for PrintInfo {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .info(&build_path(&application.current_path, Some(&self.0)))
            .map_err(|_| CommandError::FileNotFound)
    }
}
// 11) Nahraje soubor s1 z pevného disku do umístění s2 ve vašem FS
// incp s1 s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
pub struct CopyIn(String, String);
impl CopyIn {
    pub fn new(source: String, destination: String) -> Self {
        Self(source, destination)
    }
}

impl CommandHandler for CopyIn {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let file = fs::File::open(&self.0).map_err(|_| CommandError::FileNotFound)?;

        application
            .file_system
            .new_file(&build_path(&application.current_path, Some(&self.1)), file)
            .map_err(|e| match e {
                _ => CommandError::PathNotFound,
            })
    }
}
// 12) Nahraje soubor s1 z vašeho FS do umístění s2 na pevném disku
// outcp s1 s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
pub struct CopyOut(String, String);
impl CopyOut {
    pub fn new(source: String, destination: String) -> Self {
        Self(source, destination)
    }
}

impl CommandHandler for CopyOut {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let file = File::options()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&self.1)
            .map_err(|_| CommandError::FileNotFound)?;

        application
            .file_system
            .cat(&build_path(&application.current_path, Some(&self.0)), file)
            .map_err(|e| match e {
                _ => CommandError::PathNotFound,
            })
    }
}
// 13) Načte soubor z pevného disku, ve kterém budou jednotlivé příkazy, a začne je sekvenčně
// vykonávat. Formát je 1 příkaz/1řádek
// load s1
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
pub struct LoadCommands(String);
impl LoadCommands {
    pub fn new(file: String) -> Self {
        Self(file)
    }
}

impl CommandHandler for LoadCommands {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let string = read_to_string(&self.0).map_err(|_| CommandError::FileNotFound)?;
        for line in string.lines() {
            if let Some(cmd) = get(line) {
                println!("{line}");
                match cmd.handle(application) {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("{e}"),
                }
            } else {
                println!("invalid command: {line}");
            }
        }

        Ok(())
    }
}
// 14) Příkaz provede formát souboru, který byl zadán jako parametr při spuštění programu na
// souborový systém dané velikosti. Pokud už soubor nějaká data obsahoval, budou přemazána.
// Pokud soubor neexistoval, bude vytvořen.
// format 600MB
// Možný výsledek:
// OK
// CANNOT CREATE FILE
pub struct Format(String);
impl Format {
    pub fn new(size: String) -> Self {
        Self(size)
    }
}

impl CommandHandler for Format {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        let units = self.0.trim_start_matches(|c: char| c.is_digit(10));
        let count = self
            .0
            .trim_end_matches(|c: char| c.is_alphabetic())
            .parse::<usize>()
            .map_err(|_| CommandError::CannotCreateFile)?;

        let capacity = Unit::from_str(count, units).ok_or(CommandError::CannotCreateFile)?;
        application
            .file_system
            .format(capacity)
            .map_err(|_| CommandError::CannotCreateFile)
    }
}

pub struct Bug(String);
impl Bug {
    pub fn new(file: String) -> Self {
        Self(file)
    }
}

impl CommandHandler for Bug {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .bug(&build_path(&application.current_path, Some(&self.0)))
            .map_err(|_| CommandError::FileNotFound)
    }
}

pub struct Check;
impl Check {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for Check {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application
            .file_system
            .check()
            .map_err(|_| CommandError::FileNotFound)
    }
}

pub struct Exit;
impl Exit {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for Exit {
    type Error = CommandError;

    fn handle(&self, application: &mut Application) -> Result<(), Self::Error> {
        application.quit();
        Ok(())
    }
}
