// Copyright (C) 2019-2021 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::futures_support::uncompress_data;
use smol::net::TcpListener;

/// Example usage:
/// ```
/// $ ncat localhost 1234 < tests/fixtures/file.txt.gz
/// some_file_content
/// ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:1234").await?;
        loop {
            let (socket, _) = listener.accept().await?;
            smol::spawn(async move {
                let mut reader = socket.clone();
                let mut writer = socket;
                println!("{:?}", uncompress_data(&mut reader, &mut writer).await);
            })
            .detach();
        }
    })
}
