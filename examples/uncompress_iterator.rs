// SPDX-License-Identifier: MIT OR Apache-2.0

use argh::FromArgs;
use compress_tools::*;

#[derive(FromArgs, PartialEq, Eq, Debug)]
/// Top-level command.
struct TopLevel {
    /// source path
    #[argh(positional)]
    source_path: String,
}

fn main() -> compress_tools::Result<()> {
    let cmd: TopLevel = argh::from_env();

    let source = std::fs::File::open(cmd.source_path)?;

    for content in ArchiveIterator::from_read(source, Password::empty())? {
        if let ArchiveContents::StartOfEntry(name, stat) = content {
            println!("{name}: size={}", stat.st_size);
        }
    }

    Ok(())
}
