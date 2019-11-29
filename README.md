# rust-html-nlu-scrapper

```
cargo build --bin tgnews --release && mv target/release/tgnews ./

cargo run debug ./DataClusteringSample0817

cargo run languages ./DataClusteringSample0817

cargo run news ./DataClusteringSample0817

```

487s - 300k

3x index = 242s - 300k (677s total)

2x index = 329s - 300k

1x index = 421s - 300k (722s total)