use anyhow::Result;
use procfs::net;
use std::time::Duration;
use sysinfo::{Disks, LoadAvg, NetworkData, Networks, System};
use tokio::time::sleep; // 替换 thread::sleep

const BYTES_PER_MIB: f64 = 1_048_576.0;
const BYTES_PER_GIB: f64 = 1_073_741_824.0;

fn os_name() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("PRETTY_NAME=") {
                    return Some(v.trim_matches('"').to_string());
                } else if let Some(v) = line.strip_prefix("NAME=") {
                    return Some(v.trim_matches('"').to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| "Unknown".into())
}

#[derive(serde::Serialize)]
pub struct Metrics {
    pub os_name: String,
    pub uptime_days: u64,
    pub load: (f64, f64, f64),
    pub cpu: f32,
    pub mem_used: String,
    pub mem_total: String,
    pub disk_used_gib: f64,
    pub disk_total_gib: f64,
    pub rx_rate: u64,
    pub tx_rate: u64,
    pub rx_total_gib: f64,
    pub tx_total_gib: f64,
    pub swap_used_mib: f64,
    pub swap_total_mib: f64,
    pub tcp: usize,
    pub udp: usize,
    pub processes: usize,
    pub threads: usize,
}

fn net_totals(nets: &Networks) -> (u64, u64) {
    nets.iter()
        .map(|(_, n): (_, &NetworkData)| (n.total_received(), n.total_transmitted()))
        .fold((0, 0), |acc, v| (acc.0 + v.0, acc.1 + v.1))
}

fn fmt(bytes: u64) -> String {
    if bytes as f64 >= BYTES_PER_GIB {
        format!("{:.2} GiB", bytes as f64 / BYTES_PER_GIB)
    } else {
        format!("{:.2} MiB", bytes as f64 / BYTES_PER_MIB)
    }
}

/// 一次性快照；耗时 1 s 用于计算网速。
pub async fn snapshot() -> Result<Metrics> {
    let mut sys = System::new_all();
    let mut nets = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list();

    sys.refresh_cpu_usage();
    sys.refresh_memory();
    nets.refresh(false);
    disks.refresh(false);
    let (rx0, tx0) = net_totals(&nets);

    sleep(Duration::from_secs(1)).await;

    sys.refresh_cpu_usage();
    nets.refresh(false);
    let (rx1, tx1) = net_totals(&nets);

    let uptime_days = System::uptime() / 86_400;
    let LoadAvg { one, five, fifteen } = System::load_average();

    let mem_used = sys.used_memory();
    let mem_total = sys.total_memory();
    let swap_used = sys.used_swap();
    let swap_total = sys.total_swap();

    let disk_used: u64 = disks
        .list()
        .iter()
        .map(|d| d.total_space() - d.available_space())
        .sum();
    let disk_total: u64 = disks.list().iter().map(|d| d.total_space()).sum();

    let proc_cnt = sys.processes().len();
    let thread_cnt = sys
        .processes()
        .values()
        .map(|p| p.tasks().map_or(1, |set| set.len() + 1))
        .sum::<usize>();

    let tcp = net::tcp()?.len() + net::tcp6()?.len();
    let udp = net::udp()?.len() + net::udp6()?.len();

    Ok(Metrics {
        os_name: os_name(),
        uptime_days,
        load: (one, five, fifteen),
        cpu: sys.global_cpu_usage(),
        mem_used: fmt(mem_used),
        mem_total: fmt(mem_total),
        disk_used_gib: disk_used as f64 / BYTES_PER_GIB,
        disk_total_gib: disk_total as f64 / BYTES_PER_GIB,
        // saturating_sub handles counter wrap
        rx_rate: rx1.saturating_sub(rx0),
        tx_rate: tx1.saturating_sub(tx0),
        rx_total_gib: rx1 as f64 / BYTES_PER_GIB,
        tx_total_gib: tx1 as f64 / BYTES_PER_GIB,
        swap_used_mib: swap_used as f64 / BYTES_PER_MIB,
        swap_total_mib: swap_total as f64 / BYTES_PER_MIB,
        tcp,
        udp,
        processes: proc_cnt,
        threads: thread_cnt,
    })
}
