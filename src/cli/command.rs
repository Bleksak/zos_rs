use crate::Application;

pub trait CommandHandler {
    fn handle(&self, application: &mut Application) -> Option<()>;
}
//     1) Zkopíruje soubor s1 do umístění s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
// cp s1 s2
pub struct CopyFile(String, String);

impl CopyFile {
    pub fn new(source: String, destination: String) -> Self { Self(source, destination) }
}

impl CommandHandler for CopyFile {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 2) Přesune soubor s1 do umístění s2, nebo přejmenuje s1 na s2
// Možný výsledek:
// OK
// FILE NOT FOUND (není zdroj)
// PATH NOT FOUND (neexistuje cílová cesta)
pub struct MoveFile(String, String);
impl MoveFile {
    pub fn new(source: String, destination: String) -> Self { Self(source, destination) }
}

impl CommandHandler for MoveFile {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 3) Smaže soubor s1
// rm s1
// Možný výsledek:
// OK
// FILE NOT FOUND
pub struct RemoveFile(String);
impl RemoveFile {
    pub fn new(file: String) -> Self { Self(file) }
}

impl CommandHandler for RemoveFile {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(dirname: String) -> Self { Self(dirname) }
}

impl CommandHandler for MakeDirectory {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(dirname: String) -> Self { Self(dirname) }
}

impl CommandHandler for RemoveDirectory {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(dirname: Option<String>) -> Self { Self(dirname) }
}

impl CommandHandler for Listing {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 7) Vypíše obsah souboru s1
// cat s1
// Možný výsledek:
// OBSAH
// FILE NOT FOUND (není zdroj)
pub struct Concatenate(String);
impl Concatenate {
    pub fn new(dirname: String) -> Self { Self(dirname) }
}

impl CommandHandler for Concatenate {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 8) Změní aktuální cestu do adresáře a1
// cd a1
// Možný výsledek:
// OK
// PATH NOT FOUND (neexistující cesta)
pub struct ChangeDirectory(String);
impl ChangeDirectory {
    pub fn new(dirname: String) -> Self { Self(dirname) }
}
impl CommandHandler for ChangeDirectory {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 9) Vypíše aktuální cestu
// pwd
// Možný výsledek:
// PATH
pub struct PrintWorkingDirectory;
impl PrintWorkingDirectory{
    pub fn new() -> Self { Self }
}

impl CommandHandler for PrintWorkingDirectory {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 10) Vypíše informace o souboru/adresáři s1/a1 (v jakých clusterech se nachází)
// info a1/s1
// Možný výsledek:
// S1 2,3,4,7,10
// FILE NOT FOUND (není zdroj)
pub struct PrintInfo(String);
impl PrintInfo {
    pub fn new(file: String) -> Self { Self(file) }
}

impl CommandHandler for PrintInfo {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(source: String, destination: String) -> Self { Self(source, destination) }
}

impl CommandHandler for CopyIn {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(source: String, destination: String) -> Self { Self(source, destination) }
}

impl CommandHandler for CopyOut {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
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
    pub fn new(file: String) -> Self { Self(file) }
}

impl CommandHandler for LoadCommands {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
// 14) Příkaz provede formát souboru, který byl zadán jako parametr při spuštění programu na
// souborový systém dané velikosti. Pokud už soubor nějaká data obsahoval, budou přemazána.
// Pokud soubor neexistoval, bude vytvořen.
// format 600MB
// Možný výsledek:
// OK
// CANNOT CREATE FILE
pub struct Format(usize);
impl Format {
    pub fn new(size: usize) -> Self { Self(size) }
}

impl CommandHandler for Format {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}

pub struct Bug(String);
impl Bug {
    pub fn new(file: String) -> Self { Self(file) }
}

impl CommandHandler for Bug {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}

pub struct Check;
impl Check {
    pub fn new() -> Self { Self }
}

impl CommandHandler for Check {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}

pub struct Exit;
impl Exit {
    pub fn new() -> Self { Self }
}

impl CommandHandler for Exit {
    fn handle(&self, application: &mut Application) -> Option<()> {
        None
    }
}
