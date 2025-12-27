use std::fs::{self, read_link};
use std::io::{self, BufRead};
use std::path::Path;

use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    fn as_str(self) -> &'static str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SocketInfo {
    inode: u64,
    protocol: Protocol,
}

#[derive(Parser)]
#[command(name = "mappp")]
#[command(about = "ポートを使用しているプロセスを探すツール")]
struct Args {
    /// 対象ポート番号
    #[arg(short, long)]
    port: u16,
    /// tcp のみ対象にする
    #[arg(short = 't', long)]
    tcp: bool,
    /// udp のみ対象にする
    #[arg(short = 'u', long)]
    udp: bool,
}

/// ポートからinodeを取得する
/// /proc/net/tcp, tcp6, udp, udp6 を検索する
fn get_inode_from_port(port: u16, protocols: &[Protocol]) -> io::Result<Vec<SocketInfo>> {
    let files = [
        ("/proc/net/tcp", Protocol::Tcp),
        ("/proc/net/tcp6", Protocol::Tcp),
        ("/proc/net/udp", Protocol::Udp),
        ("/proc/net/udp6", Protocol::Udp),
    ];

    let mut sockets = Vec::new();

    for (file_path, protocol) in &files {
        if !protocols.contains(protocol) {
            continue;
        }
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
                                if inode != 0
                                    && !sockets.iter().any(|s: &SocketInfo| {
                                        s.inode == inode && s.protocol == *protocol
                                    })
                                {
                                    sockets.push(SocketInfo {
                                        inode,
                                        protocol: *protocol,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(sockets)
}

/// inodeからpidを取得する
/// /proc/<pid>/fd 配下のシンボリックリンクを確認し、
/// socket:[inode] にリンクしているものを探す
fn get_pid_from_inode(inode: u64) -> io::Result<Vec<u32>> {
    let mut pids = Vec::new();
    let socket_pattern = format!("socket:[{inode}]");

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
                            if link_target.to_string_lossy() == socket_pattern
                                && !pids.contains(&pid) {
                                    pids.push(pid);
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
    let protocols = match (args.tcp, args.udp) {
        (true, false) => vec![Protocol::Tcp],
        (false, true) => vec![Protocol::Udp],
        _ => vec![Protocol::Tcp, Protocol::Udp],
    };

    match get_inode_from_port(port, &protocols) {
        Ok(sockets) => {
            if sockets.is_empty() {
                println!(
                    "ポート {port} を使用しているソケットが見つかりませんでした"
                );
            } else {
                println!("PROTOCOL\tPORT\tINODE\tPID");
                for socket in sockets {
                    match get_pid_from_inode(socket.inode) {
                        Ok(pids) => {
                            println!(
                                "{}\t\t{}\t{}\t{:?}",
                                socket.protocol.as_str(),
                                port,
                                socket.inode,
                                pids
                            );
                        }
                        Err(e) => {
                            eprintln!("PID検索エラー: {e}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("inode検索エラー: {e}");
        }
    }
}
