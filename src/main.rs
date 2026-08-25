use axum::{routing::get, serve, Router};
use tokio::fs;
use tokio::time::{sleep, Duration};
use sysinfo::{System,Components,Networks};

#[tokio::main]

async fn  main() {
    let app = Router::new().route("/", get(api));
    let listener =tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener,app).await.unwrap();

}

async  fn api () -> String {
    let mem =memapi().await;
    let cpu =cpuapi().await;
    let cpufreq =cpufreqapi().await;
    let net =networkapi().await;
    let temp =tempapi().await;
    let volta =voltaapi().await;
    format!(
        "Memory:\n{}\nCPU: {}\ncppufreq:\n{}\nnet:\n{}\ntemp: \n{:?}\n\nvolta: {:?}V\n",
        mem,
        cpu,
        cpufreq,
        net,
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
        result.push_str(&format!("{}: \n{:?} °C",
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
    let voltage = volta*ratio*8.0/1000.0;
    let vol = (voltage * 1000.0).round() / 1000.0;
    vol
}

async fn networkapi() -> String {
    let mut networks = Networks::new_with_refreshed_list();

    let mut old_data = Vec::new();

    for (name, network) in &networks {
        old_data.push((
            name.clone(),
            network.total_received(),
            network.total_transmitted(),
        ));
    }

    sleep(Duration::from_millis(200)).await;

    networks.refresh(true);

    let interfaces = [
        "eth0",
        "wlan0",
        "tailscale0",
        "lo",
    ];

    let mut result = String::new();

    result.push_str(&format!(
        "{:<12} {:>14} {:>14} {:>14} {:>14}\n",
        "Interface", "RX", "TX", "Total RX", "Total TX"
    ));

    result.push_str(&format!(
        "{:-<12} {:-<14} {:-<14} {:-<14} {:-<14}\n",
        "", "", "", "", ""
    ));

    for name in interfaces {
        if let Some(network) = networks.get(name) {

            if let Some((_, old_rx, old_tx)) =
                old_data.iter().find(|(old_name, _, _)| old_name == name)
            {
                let rx_bytes =
                    network.total_received().saturating_sub(*old_rx);

                let tx_bytes =
                    network.total_transmitted().saturating_sub(*old_tx);

                let rx_speed = rx_bytes as f64 / 0.2;
                let tx_speed = tx_bytes as f64 / 0.2;

                result.push_str(&format!(
                    "{:<12} {:>10.2} KB/s {:>10.2} KB/s {:>10.2} MB {:>10.2} MB\n",
                    name,
                    rx_speed / 1024.0,
                    tx_speed / 1024.0,
                    network.total_received() as f64 / 1024.0 / 1024.0,
                    network.total_transmitted() as f64 / 1024.0 / 1024.0,
                ));
            } else {
                result.push_str(&format!(
                    "{:<12} {:>14} {:>14} {:>14} {:>14}\n",
                    name, "-", "-", "-", "-"
                ));
            }

        } else {
            // 网卡不存在，也保留这个位置
            result.push_str(&format!(
                "{:<12} {:>14} {:>14} {:>14} {:>14}\n",
                name, "-", "-", "-", "-"
            ));
        }
    }

    result
}

async fn cpufreqapi() -> String {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    let cpus = sys.cpus();
    let mut result = String::new();

    for (i, cpu) in cpus.iter().enumerate() {
        result.push_str(&format!(
            "CPU{}: {} MHz\n",
            i,
            cpu.frequency()
        ));
    }
    result
}