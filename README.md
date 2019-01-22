# rtp_jyushin_maru

Elixirで書いてたやつをシンプルにしてRustで書き直した

パケットは受信時タイムスタンプを付与してPostgresに保存可能
標準出力に流すことでpacatに渡すことも可能

## 設定ファイル

udp用，postgres用にそれぞれ設定ファイルが必要
`.yml`でも`.toml`でも`.ini`でも大丈夫

**デフォルトで(unixの場合) `~/.config/rtp_jyushin_maru/udp.yml` を見に行きます **

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
./rtp_jyushin_maru store [-c config/udp.yml] [-p config/postgres.yml]

## そのままpacatで再生
 ./rtp_jyushin_maru redirect -c config/udp.yml | pacat -p --raw --latency=1 --channels=1 --rate=48000 --format=s24be

## 解析
# 途中でさっきのを入力
julia analysis/diffs.jl
```

