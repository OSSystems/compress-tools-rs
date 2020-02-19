// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use pipers::Pipe;
use std::{fs, io::Write, path::Path};

use derive_more::{Display, From};

// Uncompress structure works by pipping the file through uncompress commands,
// Uncompress::new is used to start the stream with a `cat` from shell,
// uncompress commands can be called subsequently as needed.
// When the file is done compressing two methods can be used for output,
// Uncompress::file if the expected output is a single file,
// or Uncompress::tar expected output is a directory tree.
pub struct Uncompress(Pipe);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display, From)]
pub enum Error {
    #[display(fmt = "Io error: {}", _0)]
    Io(std::io::Error),
    #[from(ignore)]
    #[display(fmt = "Command error: {}", _0)]
    CommandError(String),
}

impl From<std::process::Output> for Error {
    fn from(out: std::process::Output) -> Self {
        Error::CommandError(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

impl Uncompress {
    pub fn new(file: &Path) -> Self {
        Uncompress(Pipe::new(&format!("cat {}", file.display())))
    }

    pub fn gz(self) -> Uncompress {
        Uncompress(self.0.then("zcat"))
    }

    pub fn bz2(self) -> Uncompress {
        Uncompress(self.0.then("bzcat"))
    }

    pub fn xz(self) -> Uncompress {
        Uncompress(self.0.then("xzcat"))
    }

    pub fn lzma(self) -> Uncompress {
        Uncompress(self.0.then("lzcat"))
    }

    pub fn lzip(self) -> Uncompress {
        Uncompress(self.0.then("lzip -dc"))
    }

    pub fn tar(self, dir: &Path) -> Result<()> {
        let out = self
            .0
            .then(&format!(
                "tar --same-owner --preserve-permissions --xattrs -xC {}",
                dir.display()
            ))
            .finally()?
            .wait_with_output()?;

        if !out.status.success() {
            Err(Error::from(out))
        } else {
            Ok(())
        }
    }

    pub fn unzip(self, dir: &Path) -> Result<()> {
        let out = self
            .0
            .then(&format!("unzip - -o -d {}", dir.display()))
            .finally()?
            .wait_with_output()?;

        if !out.status.success() {
            Err(Error::from(out))
        } else {
            Ok(())
        }
    }

    pub fn file(self, target: &Path) -> Result<()> {
        let out = self.0.finally()?.wait_with_output()?;

        fs::File::create(target)?.write_all(&out.stdout)?;

        if !out.status.success() {
            Err(Error::from(out))
        } else {
            Ok(())
        }
    }
}
