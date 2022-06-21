# Emma
Asynchronous IO library based on io_uring.

Plugin for Rust asynchronous runtime like [Tokio](https://tokio.rs/).

## What is "Emma"?
`Emma` is a character in Game [Sekiro](https://www.sekirothegame.com/) by [FromSoftware](https://www.fromsoftware.jp/ww/).

In this case, it refers to a basic coroutine library, which
relies on Rust async/await syntax and Linux io_uring feature,
aiming to build easy, flexible and high-performance
`asynchronous programming framework` in Rust.

## Concept
- Independent from any Rust asynchronous runtime.
- High performance.
- **Send but not Sync** asynchronous task design.

## Usage
Integrate into `Tokio` or `async-std` using Emma as a 3rd party library.

```toml
[dependencies]
tokio = { version = "1", features = ["rt", "rt-multi-thread", "io-util"] }
emma = { git = "https://github.com/sekirio-rs/Emma.git" }
```

## License
Copyright (C) 2022 SKTT1Ryze. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under License.
