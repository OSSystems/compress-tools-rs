// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::{uncompress, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "uncompress")]
enum Opt {
    /// Increase the verboseness level
    #[structopt(name = "tar_gz")]
    TarGz(Args),
    #[structopt(name = "tar_xz")]
    TarXz(Args),
    #[structopt(name = "tar")]
    Tar(Args),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "compression-tools")]
struct Args {
    /// compressed file to use as input
    input: PathBuf,

    /// Path to output the uncompressed result
    output: PathBuf,
}

fn main() -> Result<()> {
    match Opt::from_args() {
        Opt::TarGz(arg) => uncompress(arg.input, arg.output, compress_tools::Kind::TarGZip),
        Opt::TarXz(arg) => uncompress(arg.input, arg.output, compress_tools::Kind::TarXz),
        Opt::Tar(arg) => uncompress(arg.input, arg.output, compress_tools::Kind::Tar),
    }
}
