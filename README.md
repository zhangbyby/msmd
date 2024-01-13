## monster-siren music downloader

### before release

```shell
cargo run -h
# 1. init albums meta
cargo run init-albums-meta
# 2. create album dirs and meta
cargo run create-album-dirs-and-meta
# 3. download album pics
cargo run download-album-pics
# 4. download album songs
cargo run download-album-songs
```

### release

```shell
cargo build --release
```

### after release

```shell
./msmd -h
# 1. init albums meta
./msmd init-albums-meta
# 2. create album dirs and meta
./msmd create-album-dirs-and-meta
# 3. download album pics
./msmd download-album-pics
# 4. download album songs
./msmd download-album-songs
```