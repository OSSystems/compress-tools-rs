// Copyright (C) 2019-2021 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use async_std::net::TcpListener;
use compress_tools::futures_support::uncompress_data;

/// Example usage:
/// ```
/// $ ncat localhost 1234 < tests/fixtures/file.txt.gz
/// some_file_content
/// ```

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        async_std::task::spawn(async move {
            println!("{:?}", uncompress_data(&socket, &socket).await);
        });
    }
}
