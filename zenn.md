# ポート番号から pid を特定する実装

## 概要

[sniffnet](https://github.com/GyulyVGC/sniffnet) で「ポート番号から対応する pid を特定したい」という Issue があり、その解決方法を探していました。  
この記事では、Linux 環境でポートとプロセスの対応を辿る実装方法を調べたのでまとめます。  

調査した方法を使って、簡単なCLIツールを実装しました。その成果物は以下にあります。  
https://github.com/shu-kitamura/mappp

## 前提

今回は Linux で動作することを前提にしています。  
`/proc`配下のファイルを読むため root などの権限が必要になる場合があります。  

## 実装方法

以下の流れでポート番号から対応する pid を特定します。

1. 以下のファイルから、対象ポートを持つソケットの inode を探す  
   - `/proc/net/tcp`
   - `/proc/net/tcp6`
   - `/proc/net/udp`
   - `/proc/net/udp6`

   上述のファイルには、以下のような情報が含まれています。  
   `local_address`の`:`以降の値（16進数のため、10進数に変換する）、`inode` の値を確認します。  
   ```
     sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode                                                     
      0: XXXXXXXX:A2C6 XXXXXXXX:8A47 01 00000000:00000000 00:00000000 00000000  1000        0 2515 3 0000000000000000 20 4 33 10 -1
   ```

2. `/proc/<pid>/fd` を走査して、同じ inode を参照しているプロセスを見つける  
   以下のように `socket:[xxxx]` の `xxxx` が inode です。  
   ```
   ~$ sudo ls -l /proc/1010/fd | grep socket
   lrwx------ 1 user user 64 Dec 29 16:39 22 -> socket:[2515]
   ```

3. 見つかった pid を返す

この手順で、ポート番号から pid を特定できます。

## あとがき

今回は Linux での実装方法を調査しました。  
sniffnet はクロスプラットフォームのアプリケーションなので、  
他のOSでの実装方法を調査したいと考えています。  
