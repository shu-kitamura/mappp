use std::fs::{self, read_link};
use std::io::{self, BufRead};
use std::path::Path;

use clap::Parser;

#[derive(Parser)]
#[command(name = "mappp")]
#[command(about = "ポートを使用しているプロセスを探すツール")]
struct Args {
    /// 対象ポート番号
    #[arg(short, long)]
    port: u16,
}

/// ポートからinodeを取得する
/// /proc/net/tcp, tcp6, udp, udp6 を検索する
fn get_inode_from_port(port: u16) -> io::Result<Vec<u64>> {
    let files = [
        "/proc/net/tcp",
        "/proc/net/tcp6",
        "/proc/net/udp",
        "/proc/net/udp6",
    ];

    let mut inodes = Vec::new();

    for file_path in &files {
        if let Ok(file) = fs::File::open(file_path) {
            let reader = io::BufReader::new(file);

            for line in reader.lines().skip(1) {
                // 最初の行はヘッダーなのでスキップ
                let line = line?;
                let parts: Vec<&str> = line.split_whitespace().collect();

                if parts.len() < 10 {
                    continue;
                }

                // local_address は "IP:PORT" の形式（16進数）
                // 例: "00000000:0050" (ポート80)
                let local_address = parts[1];
                if let Some(port_hex) = local_address.split(':').nth(1) {
                    if let Ok(parsed_port) = u16::from_str_radix(port_hex, 16) {
                        if parsed_port == port {
                            // inode は10番目のフィールド（インデックス9）
                            if let Ok(inode) = parts[9].parse::<u64>() {
                                if inode != 0 && !inodes.contains(&inode) {
                                    inodes.push(inode);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(inodes)
}

/// inodeからpidを取得する
/// /proc/<pid>/fd 配下のシンボリックリンクを確認し、
/// socket:[inode] にリンクしているものを探す
fn get_pid_from_inode(inode: u64) -> io::Result<Vec<u32>> {
    let mut pids = Vec::new();
    let socket_pattern = format!("socket:[{}]", inode);

    let proc_dir = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_dir) {
        for entry in entries.flatten() {
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            // PIDディレクトリかどうか確認（数字のみ）
            if let Ok(pid) = dir_name_str.parse::<u32>() {
                let fd_dir = entry.path().join("fd");

                if let Ok(fd_entries) = fs::read_dir(&fd_dir) {
                    for fd_entry in fd_entries.flatten() {
                        if let Ok(link_target) = read_link(fd_entry.path()) {
                            if link_target.to_string_lossy() == socket_pattern {
                                if !pids.contains(&pid) {
                                    pids.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pids)
}

fn main() {
    let args = Args::parse();
    let port = args.port;

    match get_inode_from_port(port) {
        Ok(inodes) => {
            if inodes.is_empty() {
                println!("ポート {} を使用しているソケットが見つかりませんでした", port);
            } else {

                for inode in inodes {
                    match get_pid_from_inode(inode) {
                        Ok(pids) => {
                            if pids.is_empty() {
                                println!("ポート {} を使用しているプロセスが見つかりませんでした", port);
                            } else {
                                println!("ポート {} を使用しているPID: {:?}", port, pids);
                            }
                        }
                        Err(e) => {
                            eprintln!("PID検索エラー: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("inode検索エラー: {}", e);
        }
    }
}
