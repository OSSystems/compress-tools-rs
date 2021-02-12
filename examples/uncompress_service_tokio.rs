// Copyright (C) 2019-2021 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::tokio_support::uncompress_data;
use tokio::net::TcpListener;

/// Example usage:
/// ```
/// $ ncat localhost 1234 < tests/fixtures/file.txt.gz
/// some_file_content
/// ```

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (read_half, write_half) = socket.into_split();
            println!("{:?}", uncompress_data(read_half, write_half).await);
        });
    }
}
