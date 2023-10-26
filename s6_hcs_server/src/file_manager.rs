use s6_hcs_lib_transfer::aux::FileList;

use path_macro::path;
use rand::random;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::{
    error::Error,
    fs::{self, File},
    io,
    io::{BufReader, Read, Write},
    path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    name: String,
    key: u128,
}

pub struct FileManager {
    dir: PathBuf,
}

impl FileManager {
    pub fn new(dir: &str) -> Result<Self, Box<dyn Error>> {
        if !(Path::new(&dir).exists()) {
            fs::create_dir_all(&dir)?;
        }
        let new = Self {
            dir: PathBuf::from_str(&dir)?,
        };
        new.init_cleanup()?;
        Ok(new)
    }

    pub fn save_file(&self, name: String, key: u128, contents: Vec<u128>) -> io::Result<()> {
        let id: u128 = random();
        let path = path!(self.dir / format!("{id}"));
        fs::create_dir_all(&path)?;

        let mut file = File::create(path!(path / "file"))?;
        for block in contents {
            file.write_all(&block.to_be_bytes())?;
        }
        file.flush()?;

        let metadata = Metadata { name, key };
        let metadata_json = serde_json::to_string(&metadata)?;
        fs::write(path!(path / "metadata.json"), metadata_json)?;

        Ok(())
    }

    fn init_cleanup(&self) -> Result<(), Box<dyn Error>> {
        for path in {
            fs::read_dir(&self.dir)?
                .map(|p| p.unwrap().path())
                .filter(|p| p.is_dir())
        } {
            if let Ok(_) = fs::read_to_string(path!(path / "metadata.json")) {
                fs::remove_file(path!(path / "lock")).unwrap_or_default()
            }
        }
        Ok(())
    }

    pub fn get_file_list(&self) -> Result<FileList, Box<dyn Error>> {
        let mut file_list: FileList = Vec::new();
        for path in {
            fs::read_dir(&self.dir)?
                .map(|p| p.unwrap().path())
                .filter(|p| p.is_dir())
        } {
            if let Ok(metadata) = fs::read_to_string(path!(path / "metadata.json")) {
                file_list.push((
                    path.file_name().unwrap().to_str().unwrap().parse()?,
                    fs::metadata(path!(path / "file"))?.len() as usize,
                    serde_json::from_str::<Metadata>(&metadata)?.name,
                ));
            }
        }

        Ok(file_list)
    }

    pub fn get_file(&self, id: u128) -> io::Result<(Vec<u128>, u128)> {
        let path = path!(self.dir / format!("{id}"));
        fs::write(path!(path / "lock"), "")?;
        let file = File::open(&path!(path / "file"))?;
        let metadata = fs::read_to_string(path!(path / "metadata.json"))?;
        let metadata: Metadata = serde_json::from_str(&metadata)?;

        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 16];
        let mut contents = vec![];
        while reader.read_exact(&mut buffer).is_ok() {
            contents.push(u128::from_be_bytes(buffer));
        }
        fs::remove_file(path!(path / "lock"))?;
        Ok((contents, metadata.key))
    }

    pub fn delete_file(&self, id: u128) -> io::Result<()> {
        let path = path!(self.dir / format!("{id}"));
        if !path!(path / "lock").exists() {
            fs::remove_dir_all(path).unwrap_or_default();
        }
        Ok(())
    }
}
