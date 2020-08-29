// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::uncompress_data;
use std::{net::TcpListener, thread};

/// Example usage:
/// ```
/// $ ncat localhost 1234 < tests/fixtures/file.txt.gz
/// some_file_content
/// ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    loop {
        let (socket, _) = listener.accept()?;
        thread::spawn(move || println!("{:?}", uncompress_data(&socket, &socket)));
    }
}
