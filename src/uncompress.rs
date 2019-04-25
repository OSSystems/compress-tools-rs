// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use failure::ensure;
use pipers::Pipe;
use std::{fs, io::Write, path::Path};

// Uncompress structure works by pipping the file through uncompress commands,
// Uncompress::new is used to start the stream with a `cat` from shell,
// uncompress commands can be called subsequently as needed.
// When the file is done compressing two methods can be used for output,
// Uncompress::file if the expected output is a single file,
// or Uncompress::tar expected output is a directory tree.
pub struct Uncompress(Pipe);

impl Uncompress {
    pub fn new(file: &Path) -> Self {
        Uncompress(Pipe::new(&format!("cat {}", file.display())))
    }

    pub fn gz(self) -> Uncompress {
        Uncompress(self.0.then("gzip -dc"))
    }

    pub fn bz2(self) -> Uncompress {
        Uncompress(self.0.then("bzip2 -dc"))
    }

    pub fn xz(self) -> Uncompress {
        Uncompress(self.0.then("xz -dc"))
    }

    pub fn lzma(self) -> Uncompress {
        Uncompress(self.0.then("lzma -dc"))
    }

    pub fn lz(self) -> Uncompress {
        Uncompress(self.0.then("lzip -dc"))
    }

    pub fn tar(self, dir: &Path) -> Result<(), failure::Error> {
        let out = self
            .0
            .then(&format!("tar -xC {}", dir.display()))
            .finally()?
            .wait_with_output()?;
        ensure!(
            out.status.success(),
            format!("Extract process exited with error:{:#?}", out)
        );
        Ok(())
    }

    pub fn file(self, target: &Path) -> Result<(), failure::Error> {
        let out = self.0.finally()?.wait_with_output()?;
        ensure!(
            out.status.success(),
            format!("Extract process exited with error:{:#?}", out)
        );
        fs::File::create(target)?.write_all(&out.stdout)?;
        Ok(())
    }
}
