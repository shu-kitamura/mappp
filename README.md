# mappp(Mapping Port and Process)

ポート番号から、そのポートを使用しているプロセスの PID を探す CLI ツールです。
`/proc` を参照するため Linux 環境向けです。

## 使い方

```bash
cargo run -- -p 8080
```

もしくはリリースビルドして実行します。

```bash
cargo build --release
./target/release/mappp -p 8080
```

## 例

```bash
$ cargo run -- -p 5461
ポート 5461 のinodeを検索中...
見つかったinode: [123456]
inode 123456 を使用しているPID: [9876]
```

## オプション

- `-p`, `--port` 対象ポート番号

## 注意

- `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, `/proc/net/udp6` を検索します。
- 実行ユーザーの権限によっては PID が取得できない場合があります。
