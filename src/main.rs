use axum::{routing::get, serve, Router};
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
    let mem =memapi().await;
    let cpu =cpuapi().await;
    let temp =tempapi().await;
    let volta =voltaapi().await;
    format!(
        "Memory:\n{}\nCPU: {}\ntemp: {:?}\nvolta: {:?}",
        mem,
        cpu,
        temp,
        volta
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

async fn voltaapi() -> f64 {
    let v=fs::read_to_string("/sys/bus/iio/devices/iio:device0/in_voltage2_raw").await.unwrap();
    let volta:f64 =v.trim().parse().unwrap();
    let Ratio_String = fs::read_to_string("/sys/bus/iio/devices/iio:device0/in_voltage_scale").await.unwrap();
    let ratio:f64 = Ratio_String.trim().parse().unwrap();
    volta*ratio*8.0
}


