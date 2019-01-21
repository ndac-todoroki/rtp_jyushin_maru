# rtp_jyushin_maru

Elixirで書いてたやつをシンプルにしてRustで書き直した

パケットは受信時タイムスタンプを付与してPostgresに保存．

## 設定ファイル

`config/postgres.yml`, `config/udp.yml` に設定が必要

ymlでもtomlでもiniでも大丈夫

### 例
#### postgres.yml
```yml
user: postgres
password: postgres
host: localhost
database: packet_jyushin_maru_repo
```

#### udp.yml
```yml
host: 0.0.0.0  # localhostじゃだめ
port: 5004
max_packets: 50000  # 一回の測定で保存するパケット数
```

## HOW TO USE

```bash
## ビルド
cargo build --release
ln -rs target/release/rtp_jyushin_maru

## 受信．保存
# テストケース名が表示されるのでそれをメモっておく
./rtp_jyushin_maru

## 解析
# 途中でさっきのを入力
julia analysis/diffs.jl
```

