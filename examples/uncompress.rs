// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use argh::FromArgs;
use compress_tools::*;
use std::path::Path;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: CmdLine,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum CmdLine {
    UncompressFile(SubCommandUncompressFile),
    UncompressArchiveFile(SubCommandUncompressArchiveFile),
    UncompressArchive(SubCommandUncompressArchive),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Uncompress subcommand.
#[argh(subcommand, name = "uncompress-file")]
struct SubCommandUncompressFile {
    /// source path
    #[argh(positional)]
    source_path: String,

    /// target path
    #[argh(positional)]
    target_path: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Uncompress archive file subcommand.
#[argh(subcommand, name = "uncompress-archive-file")]
struct SubCommandUncompressArchiveFile {
    /// source path
    #[argh(positional)]
    source_path: String,

    /// target path
    #[argh(positional)]
    target_path: String,

    /// target file
    #[argh(positional)]
    target_file: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Uncompress archive subcommand.
#[argh(subcommand, name = "uncompress-archive")]
struct SubCommandUncompressArchive {
    /// source path
    #[argh(positional)]
    source_path: String,

    /// target path
    #[argh(positional)]
    target_path: String,
}

fn main() -> compress_tools::Result<()> {
    let cmd: TopLevel = argh::from_env();

    match cmd.nested {
        CmdLine::UncompressFile(input) => {
            let mut source = std::fs::File::open(input.source_path)?;
            let mut target = std::fs::File::open(input.target_path)?;

            uncompress_file(&mut source, &mut target)?;
        }
        CmdLine::UncompressArchiveFile(input) => {
            let mut source = std::fs::File::open(input.source_path)?;
            let mut target = std::fs::File::open(input.target_path)?;

            uncompress_archive_file(&mut source, &mut target, &input.target_file)?;
        }
        CmdLine::UncompressArchive(input) => {
            let mut source = std::fs::File::open(input.source_path)?;

            uncompress_archive(&mut source, Path::new(&input.target_path))?;
        }
    }

    Ok(())
}
