use axum::{routing::get, serve, Router};
use std::fs::{create_dir, exists, read_dir,read_to_string};
use std::path::{Path, PathBuf};
use std::{mem, vec};
use tokio::fs;
use tokio::time::{sleep, Duration};
use sysinfo::{System,Components};

#[tokio::main]

async fn  main() {
    let app = Router::new().route("/", get(api));
    let listener =tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener,app).await.unwrap();

}

async  fn api () -> String {
    let mem=memapi().await;
    let cpu =cpuapi().await;
    let  temp =tempapi().await;
    format!(
        "Memory:\n{}\nCPU: {}\ntemp: {:?}\n",
        mem,
        cpu,
        temp
    )
}
async fn  memapi() -> String {
    let info = fs::read_to_string("/proc/meminfo").await.unwrap();
    let mut result = String::new();

    for line in info.lines() {
        if line.starts_with("MemTotal:")
            || line.starts_with("MemAvailable:")
            || line.starts_with("SwapTotal:")
            || line.starts_with("SwapFree:")
        {
            result.push_str(&line);
            result.push('\n');
        }
    }
    result
}
async fn cpuapi() -> String {
    let mut sys = System::new();

    sys.refresh_cpu_usage();

    sleep(Duration::from_millis(200)).await;

    sys.refresh_cpu_usage();

    format!("{:.1}%", sys.global_cpu_usage())
}

async fn tempapi() -> String {
    let components = Components::new_with_refreshed_list();
    let mut result =String::new();

    for component in &components {
        result.push_str(&format!("{}: {:?} °C",
            component.label(),
            component.temperature()
        ));
    }
    result
}

