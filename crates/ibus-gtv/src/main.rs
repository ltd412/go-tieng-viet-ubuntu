mod appdetect;
mod config;
mod ibus;
mod session;

use std::path::PathBuf;
use std::time::Duration;

use zbus::connection;

const BUS_NAME: &str = "org.freedesktop.IBus.GTV";

/// Address of the running ibus-daemon: `$IBUS_ADDRESS`, else the file the daemon writes under
/// `~/.config/ibus/bus/`, else whatever `ibus address` prints.
fn ibus_address() -> Option<String> {
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        return Some(addr);
    }
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    let machine_id = std::fs::read_to_string("/var/lib/dbus/machine-id")
        .or_else(|_| std::fs::read_to_string("/etc/machine-id"))
        .ok()?;
    let (host, display) = display_parts();
    let file = home.join("ibus/bus").join(format!("{}-{host}-{display}", machine_id.trim()));
    let text = std::fs::read_to_string(file).ok();
    let from_file = text
        .and_then(|t| t.lines().find_map(|l| l.strip_prefix("IBUS_ADDRESS=").map(str::to_string)));
    from_file.or_else(|| {
        let out = std::process::Command::new("ibus").arg("address").output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    })
}

/// `:0` -> ("unix", "0"), `host:1.0` -> ("host", "1"), no DISPLAY -> Wayland socket name.
fn display_parts() -> (String, String) {
    match std::env::var("DISPLAY") {
        Ok(d) if !d.is_empty() => {
            let (host, rest) = d.split_once(':').unwrap_or(("", &d));
            let host = if host.is_empty() { "unix" } else { host };
            (host.to_string(), rest.split('.').next().unwrap_or("0").to_string())
        }
        _ => {
            ("unix".into(), std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into()))
        }
    }
}

async fn run() -> zbus::Result<()> {
    let addr =
        ibus_address().ok_or_else(|| zbus::Error::Failure("cannot locate ibus-daemon".into()))?;
    let shared = ibus::Shared::new();
    let conn = connection::Builder::address(addr.as_str())?
        .name(BUS_NAME)?
        .serve_at("/org/freedesktop/IBus/Factory", ibus::Factory::new(shared))?
        .build()
        .await?;
    log::info!("serving {BUS_NAME} on {addr}");
    // Exit with the daemon: a dead bus makes any call fail.
    let bus = zbus::fdo::DBusProxy::new(&conn).await?;
    loop {
        async_io::Timer::after(Duration::from_secs(3)).await;
        if bus.list_names().await.is_err() {
            log::info!("ibus-daemon went away, exiting");
            return Ok(());
        }
    }
}

/// `GTV_LOG` pins the level; without it the level follows the `debug` file (see `follow_debug_flag`).
fn pinned_level() -> Option<log::LevelFilter> {
    std::env::var("GTV_LOG").ok().map(|v| v.parse().unwrap_or(log::LevelFilter::Info))
}

fn config_dir() -> PathBuf {
    config::FileConfig::path().with_file_name("")
}

/// A `debug` file next to config.toml turns on full tracing into `engine.log` beside it, and
/// deleting the file turns it off again. Checked at every focus change, so users can switch it on
/// without restarting IBus (which would interrupt their typing).
pub fn follow_debug_flag() {
    static ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if pinned_level().is_some() {
        return;
    }
    let dir = config_dir();
    let want = dir.join("debug").exists();
    if ON.swap(want, std::sync::atomic::Ordering::Relaxed) == want {
        return;
    }
    if want {
        // WHY: a trace left on for days must not fill the disk.
        let log = dir.join("engine.log");
        if std::fs::metadata(&log).is_ok_and(|m| m.len() > 10 << 20) {
            let _ = std::fs::OpenOptions::new().write(true).truncate(true).open(&log);
        }
    }
    log::set_max_level(if want { log::LevelFilter::Trace } else { log::LevelFilter::Info });
    log::info!("tracing {}", if want { "on" } else { "off" });
}

fn main() {
    // ibus-daemon discards the stderr of engines it spawns, hence the log file.
    let mut logger = env_logger::Builder::new();
    logger.filter_level(pinned_level().unwrap_or(log::LevelFilter::Trace));
    logger.format_timestamp_millis();
    let log_file =
        std::env::var_os("GTV_LOG_FILE").unwrap_or_else(|| config_dir().join("engine.log").into());
    if let Ok(file) = std::fs::OpenOptions::new().create(true).append(true).open(log_file) {
        logger.target(env_logger::Target::Pipe(Box::new(file)));
    }
    logger.init();
    if pinned_level().is_none() {
        log::set_max_level(log::LevelFilter::Info);
    }
    follow_debug_flag();
    if std::env::args().any(|a| a == "--version") {
        println!("ibus-engine-gtv {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if let Err(e) = zbus::block_on(run()) {
        log::error!("{e}");
        std::process::exit(1);
    }
}
