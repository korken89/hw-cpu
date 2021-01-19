use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::thread;
use std::time::Duration;
use systemstat::{data::CPULoad, Platform, System};

const USB_VENDOR_ID: u16 = 0x1209; // pid.codes VID.
const USB_PRODUCT_ID: u16 = 0x0001; // In house private testing only.

fn main() {
    if let Some(pinfo) = detect_port() {
        println!("port: {}", pinfo.port_name);
        let mut port = open_port(&pinfo).unwrap();
        loop {
            print_load(&mut port);
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

/// Looks for our monitor hardware on available serial ports.
fn detect_port() -> Option<SerialPortInfo> {
    // Detect serial port for monitor hardware.
    let ports = serialport::available_ports().expect("No serial ports found!");

    ports.into_iter().find(|p| match &p.port_type {
        SerialPortType::UsbPort(info) => info.vid == USB_VENDOR_ID && info.pid == USB_PRODUCT_ID,
        _ => false,
    })
}

fn open_port(port_info: &SerialPortInfo) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let mut port = serialport::new(port_info.port_name.clone(), 115200).open()?;
    port.write_data_terminal_ready(true)?;
    Ok(port)
}

/// CPU load.
fn print_load(w: &mut Box<dyn SerialPort>) {
    let sys = System::new();
    let cpu_load = sys.cpu_load().unwrap();
    let load_agg = sys.cpu_load_aggregate().unwrap();

    thread::sleep(Duration::from_secs(1));

    let load_agg = load_agg.done().unwrap();
    write!(w, "aggregate: {:.2}\r", total_load(&load_agg) * 100.0).unwrap();

    let cpu_load = cpu_load.done().unwrap();
    let min_idle = cpu_load
        .iter()
        .min_by(|a, b| a.idle.partial_cmp(&b.idle).unwrap());
    if let Some(min_idle) = min_idle {
        write!(w, "peak core: {:.2}\r", total_load(min_idle) * 100.0).unwrap();
    }
}

fn total_load(load: &CPULoad) -> f32 {
    1.0f32 - load.idle
}
