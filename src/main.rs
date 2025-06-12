use std::{thread, time::Duration};

use anyhow::Result;
use procfs::net;
use sysinfo::{Disks, LoadAvg, NetworkData, Networks, System};

const BYTES_PER_MIB: f64 = 1_048_576.0;      // 1024²
const BYTES_PER_GIB: f64 = 1_073_741_824.0;  // 1024³

/// Accumulate total received and transmitted bytes across all network interfaces.
fn net_totals(nets: &Networks) -> (u64, u64) {
    nets.iter()
        .map(|(_, n): (_, &NetworkData)| (n.total_received(), n.total_transmitted()))
        .fold((0, 0), |acc, v| (acc.0 + v.0, acc.1 + v.1))
}

/// Format memory in GiB when total ≥ 1 GiB, otherwise in MiB.
fn format_mem(bytes: u64) -> String {
    if bytes as f64 >= BYTES_PER_GIB {
        format!("{:.2} GiB", bytes as f64 / BYTES_PER_GIB)
    } else {
        format!("{:.2} MiB", bytes as f64 / BYTES_PER_MIB)
    }
}

fn main() -> Result<()> {
    let mut sys   = System::new_all();
    let mut nets  = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list();

    // first snapshot
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    nets.refresh(false);
    disks.refresh(false);
    let (rx0, tx0) = net_totals(&nets);

    thread::sleep(Duration::from_secs(1));

    // second snapshot
    sys.refresh_cpu_usage();
    nets.refresh(false);
    let (rx1, tx1) = net_totals(&nets);

    // static counters
    let uptime_days = System::uptime() / 86_400;
    let LoadAvg { one, five, fifteen } = System::load_average();

    // memory
    let mem_used  = sys.used_memory();   // bytes
    let mem_total = sys.total_memory();  // bytes
    let swap_used = sys.used_swap();     // bytes
    let swap_total= sys.total_swap();    // bytes

    // disk
    let disk_used: u64  = disks.list().iter().map(|d| d.total_space() - d.available_space()).sum();
    let disk_total: u64 = disks.list().iter().map(|d| d.total_space()).sum();

    // processes / threads
    let proc_cnt   = sys.processes().len();
    let thread_cnt = sys.processes().values()
        .map(|p| p.tasks().map_or(1, |set| set.len() + 1))
        .sum::<usize>();

    // sockets
    let tcp = net::tcp()?.len() + net::tcp6()?.len();
    let udp = net::udp()?.len() + net::udp6()?.len();

    // output
    println!("运行时间 {} 天",   uptime_days);
    println!("负载  {:.2}  {:.2}  {:.2}", one, five, fifteen);
    println!("CPU   {:.1}%",   sys.global_cpu_usage());
    println!("内存  {} / {}",   format_mem(mem_used),  format_mem(mem_total));
    println!("硬盘  {:.2} GiB / {:.2} GiB",
             disk_used as f64 / BYTES_PER_GIB,
             disk_total as f64 / BYTES_PER_GIB);
    println!("网络  ↓ {} B/s  ↑ {} B/s", rx1 - rx0, tx1 - tx0);
    println!("流量  ↓ {:.2} GiB ↑ {:.2} GiB",
             rx1 as f64 / BYTES_PER_GIB, tx1 as f64 / BYTES_PER_GIB);
    println!("SWAP  {:.2} MiB / {:.2} MiB",
             swap_used  as f64 / BYTES_PER_MIB,
             swap_total as f64 / BYTES_PER_MIB);
    println!("TCP {}  UDP {}", tcp, udp);
    println!("进程 {}  线程 {}", proc_cnt, thread_cnt);

    Ok(())
}
