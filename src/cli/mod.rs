use self::command::*;

mod command;

pub fn get(line: &str) -> Option<Box<dyn CommandHandler<Error = CommandError>>> {
    if line.len() == 0 {
        return None;
    }

    let words: Vec<&str> = line.split_whitespace().collect();

    match *words.get(0)? {
        "cp" => Some(Box::new(CopyFile::new(
            words.get(1)?.to_string(),
            words.get(2)?.to_string(),
        ))),
        "mv" => Some(Box::new(MoveFile::new(
            words.get(1)?.to_string(),
            words.get(2)?.to_string(),
        ))),
        "rm" => Some(Box::new(RemoveFile::new(words.get(1)?.to_string()))),
        "mkdir" => Some(Box::new(MakeDirectory::new(words.get(1)?.to_string()))),
        "rmdir" => Some(Box::new(RemoveDirectory::new(words.get(1)?.to_string()))),
        "ls" => Some(Box::new(Listing::new(words.get(1).map(|s| s.to_string())))),
        "cat" => Some(Box::new(Concatenate::new(words.get(1)?.to_string()))),
        "cd" => Some(Box::new(ChangeDirectory::new(words.get(1)?.to_string()))),
        "pwd" => Some(Box::new(PrintWorkingDirectory::new())),
        "info" => Some(Box::new(PrintInfo::new(words.get(1)?.to_string()))),
        "incp" => Some(Box::new(CopyIn::new(
            words.get(1)?.to_string(),
            words.get(2)?.to_string(),
        ))),
        "outcp" => Some(Box::new(CopyOut::new(
            words.get(1)?.to_string(),
            words.get(2)?.to_string(),
        ))),
        "load" => Some(Box::new(LoadCommands::new(words.get(1)?.to_string()))),
        "format" => Some(Box::new(Format::new(words.get(1)?.to_string()))),
        "bug" => Some(Box::new(Bug::new(words.get(1)?.to_string()))),
        "check" => Some(Box::new(Check::new())),
        "exit" => Some(Box::new(Exit::new())),
        _ => None,
    }
}
