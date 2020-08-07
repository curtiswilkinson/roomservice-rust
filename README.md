# roomservice-rust
The canonical roomservice repository, replacing roomservice JS

## Installation

Please note the second command maybe require sudo in order to move the binary into /usr/local/bin

### Linux

```sh
curl -L https://github.com/curtiswilkinson/roomservice-rust/releases/download/v4.0.1/x86_64-unknown-linux-musl.tar.gz | tar xz
 
cp target/x86_64-unknown-linux-musl/release/roomservice /usr/local/bin && rm -rf target roomservice.tar.gz
```

### OSX

```sh
curl -L https://github.com/curtiswilkinson/roomservice-rust/releases/download/v4.0.1/x86_64-apple-darwin.tar.gz | tar xz
 
cp target/x86_64-apple-darwin/roomservice /usr/local/bin && rm -rf target roomservice.tar.gz

```
