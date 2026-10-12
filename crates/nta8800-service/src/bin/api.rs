//! HTTP API server for the Open Energy Studio NTA 8800 kernel.
//!
//! Options (flags win over environment variables):
//!
//! | flag | environment | default |
//! |---|---|---|
//! | `--bind ADDR` | `OES_API_BIND` | `127.0.0.1` |
//! | `--port PORT` | `OES_API_PORT` | `3007` |
//! | `--cors-origin ORIGIN` (repeatable, `*` for any) | `OES_API_CORS_ORIGINS` (comma separated) | none |
//! | `--body-limit-mb N` | `OES_API_BODY_LIMIT_MB` | `16` |
//! | `--max-json-values N` | `OES_API_MAX_JSON_VALUES` | `500000` |
//! | `--allowed-host HOST` (repeatable) | `OES_API_ALLOWED_HOSTS` (comma separated) | on a loopback address: `localhost`, `127.0.0.1`, `::1` |
//! | `--max-calculations N` | `OES_API_MAX_CALCULATIONS` | number of cores (at most 1024) |
//! | `--max-waiting N` | `OES_API_MAX_WAITING` | twice the calculations |
//! | `--queue-timeout-s N` | `OES_API_QUEUE_TIMEOUT_S` | `30` |
//! | `--calculation-timeout-s N` | `OES_API_CALCULATION_TIMEOUT_S` | `120` |
//! | `--log` / `--no-log` | `OES_API_LOG` (`1`/`0`) | on |
//!
//! SIGINT and SIGTERM stop accepting connections and let running requests finish.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use nta8800_service::HttpConfig;

struct Options {
    bind: IpAddr,
    port: u16,
    http: HttpConfig,
}

const USAGE: &str = "usage: api [--bind ADDR] [--port PORT] [--cors-origin ORIGIN]... [--allowed-host HOST]... [--body-limit-mb N] [--max-json-values N] [--max-calculations N] [--max-waiting N] [--queue-timeout-s N] [--calculation-timeout-s N] [--log|--no-log] [--version]";

/// Splits a comma-separated environment list.
fn list(value: Option<String>) -> Vec<String> {
    value
        .map(|list| {
            list.split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn parse(args: Vec<String>) -> Result<Options, String> {
    let env = |key: &str| {
        std::env::var(key)
            .ok()
            .filter(|value| !value.trim().is_empty())
    };
    let mut bind = env("OES_API_BIND").unwrap_or_else(|| "127.0.0.1".into());
    let mut port = env("OES_API_PORT").unwrap_or_else(|| "3007".into());
    let mut origins = list(env("OES_API_CORS_ORIGINS"));
    let mut hosts = list(env("OES_API_ALLOWED_HOSTS"));
    let mut limit = env("OES_API_BODY_LIMIT_MB").unwrap_or_else(|| "16".into());
    let mut values = env("OES_API_MAX_JSON_VALUES")
        .unwrap_or_else(|| nta8800_service::http::DEFAULT_MAX_JSON_VALUES.to_string());
    let mut calculations = env("OES_API_MAX_CALCULATIONS").unwrap_or_else(|| {
        nta8800_service::http::default_max_concurrent_calculations().to_string()
    });
    let mut waiting = env("OES_API_MAX_WAITING");
    let mut queue_timeout = env("OES_API_QUEUE_TIMEOUT_S").unwrap_or_else(|| "30".into());
    let mut timeout = env("OES_API_CALCULATION_TIMEOUT_S").unwrap_or_else(|| "120".into());
    let mut log = env("OES_API_LOG").map(|value| value != "0").unwrap_or(true);
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        let mut value = |name: &str| iter.next().ok_or(format!("{name} needs a value\n{USAGE}"));
        match arg.as_str() {
            "--bind" => bind = value("--bind")?,
            "--port" => port = value("--port")?,
            "--cors-origin" => origins.push(value("--cors-origin")?),
            "--allowed-host" => hosts.push(value("--allowed-host")?),
            "--body-limit-mb" => limit = value("--body-limit-mb")?,
            "--max-json-values" => values = value("--max-json-values")?,
            "--max-calculations" => calculations = value("--max-calculations")?,
            "--max-waiting" => waiting = Some(value("--max-waiting")?),
            "--queue-timeout-s" => queue_timeout = value("--queue-timeout-s")?,
            "--calculation-timeout-s" => timeout = value("--calculation-timeout-s")?,
            "--log" => log = true,
            "--no-log" => log = false,
            "--help" | "-h" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown option {other}\n{USAGE}")),
        }
    }
    let bind: IpAddr = bind
        .parse()
        .map_err(|_| format!("invalid bind address {bind}"))?;
    let port: u16 = port.parse().map_err(|_| format!("invalid port {port}"))?;
    let limit: usize = limit
        .parse()
        .ok()
        .filter(|mb: &usize| (1..=1024).contains(mb))
        .ok_or(format!("invalid body limit {limit} (1–1024 MB)"))?;
    let values: usize = values
        .parse()
        .ok()
        .filter(|n: &usize| (1_000..=100_000_000).contains(n))
        .ok_or(format!(
            "invalid JSON value limit {values} (1000–100000000)"
        ))?;
    // On a loopback address only the loopback names are served (DNS
    // rebinding); extra names come from --allowed-host. On any other
    // address only the configured names are checked, if any.
    if bind.is_loopback() {
        for host in nta8800_service::http::LOOPBACK_HOSTS {
            if !hosts.iter().any(|item| item == host) {
                hosts.push(host.to_string());
            }
        }
    }
    let calculations: usize = calculations
        .parse()
        .ok()
        .filter(|n: &usize| (1..=nta8800_service::http::MAX_CALCULATION_SLOTS).contains(n))
        .ok_or(format!(
            "invalid number of calculations {calculations} (1–1024)"
        ))?;
    let waiting: usize = match waiting {
        None => calculations * 2,
        Some(waiting) => waiting
            .parse()
            .ok()
            .filter(|n: &usize| *n <= 65_536)
            .ok_or(format!(
                "invalid number of waiting requests {waiting} (0–65536)"
            ))?,
    };
    let queue_timeout: u64 = queue_timeout
        .parse()
        .ok()
        .filter(|s: &u64| (1..=86_400).contains(s))
        .ok_or(format!("invalid queue timeout {queue_timeout} (1–86400 s)"))?;
    let timeout: u64 = timeout
        .parse()
        .ok()
        .filter(|s: &u64| (1..=86_400).contains(s))
        .ok_or(format!("invalid calculation timeout {timeout} (1–86400 s)"))?;
    Ok(Options {
        bind,
        port,
        http: HttpConfig {
            body_limit_bytes: limit * 1024 * 1024,
            max_json_values: values,
            cors_origins: origins,
            allowed_hosts: hosts,
            log_requests: log,
            max_concurrent_calculations: calculations,
            max_waiting_requests: waiting,
            queue_timeout: Duration::from_secs(queue_timeout),
            calculation_timeout: Duration::from_secs(timeout),
        },
    })
}

/// Resolves when SIGINT or SIGTERM arrives. The signals are blocked in every
/// thread (the mask is inherited) and received by one waiting thread.
#[cfg(unix)]
fn shutdown_signal() -> impl std::future::Future<Output = ()> {
    let (sender, receiver) = tokio::sync::oneshot::channel::<i32>();
    // SAFETY: plain libc signal-mask calls on a zeroed sigset_t.
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGINT);
        libc::sigaddset(&mut set, libc::SIGTERM);
        libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        let set = set;
        std::thread::spawn(move || {
            let mut signal = 0;
            libc::sigwait(&set, &mut signal);
            let _ = sender.send(signal);
        });
    }
    async move {
        if let Ok(signal) = receiver.await {
            eprintln!("Received signal {signal}; finishing running requests");
        }
    }
}

#[cfg(not(unix))]
fn shutdown_signal() -> impl std::future::Future<Output = ()> {
    std::future::pending()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("{}", nta8800_service::operations::version_value());
        return Ok(());
    }
    let options = match parse(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    // Block the signals before the runtime starts its worker threads.
    let shutdown = shutdown_signal();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let address = SocketAddr::new(options.bind, options.port);
        let listener = tokio::net::TcpListener::bind(address).await?;
        if !options.bind.is_loopback() {
            eprintln!("Warning: listening on a non-loopback address without authentication");
        }
        eprintln!(
            "Open Energy Studio API {} (kernel {}, {}) listening on http://{address}",
            env!("CARGO_PKG_VERSION"),
            nta8800_core::KERNEL_VERSION,
            nta8800_core::TARGET_NORM_VERSION
        );
        axum::serve(listener, nta8800_service::app_with(options.http))
            .with_graceful_shutdown(shutdown)
            .await?;
        eprintln!("Open Energy Studio API stopped");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn flags_override_defaults() {
        let options = parse(
            [
                "--bind",
                "0.0.0.0",
                "--port",
                "4000",
                "--cors-origin",
                "http://localhost:5173",
                "--body-limit-mb",
                "2",
                "--max-calculations",
                "3",
                "--max-waiting",
                "0",
                "--queue-timeout-s",
                "5",
                "--calculation-timeout-s",
                "30",
                "--no-log",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        )
        .unwrap();
        assert_eq!(options.port, 4000);
        assert!(!options.bind.is_loopback());
        assert_eq!(
            options.http.cors_origins,
            vec!["http://localhost:5173".to_string()]
        );
        assert_eq!(options.http.body_limit_bytes, 2 * 1024 * 1024);
        assert!(!options.http.log_requests);
        assert_eq!(options.http.max_concurrent_calculations, 3);
        assert_eq!(options.http.calculation_timeout.as_secs(), 30);
        assert_eq!(options.http.max_waiting_requests, 0);
        assert_eq!(options.http.queue_timeout.as_secs(), 5);
    }

    #[test]
    fn defaults_are_within_the_accepted_ranges() {
        let defaults = nta8800_service::HttpConfig::default();
        assert!((1..=nta8800_service::http::MAX_CALCULATION_SLOTS)
            .contains(&defaults.max_concurrent_calculations));
        assert_eq!(
            defaults.max_waiting_requests,
            defaults.max_concurrent_calculations * 2
        );
        assert!(defaults.queue_timeout < defaults.calculation_timeout);
    }

    #[test]
    fn bad_values_are_refused() {
        assert!(parse(vec!["--port".into(), "x".into()]).is_err());
        assert!(parse(vec!["--body-limit-mb".into(), "0".into()]).is_err());
        assert!(parse(vec!["--frobnicate".into()]).is_err());
        assert!(parse(vec!["--max-calculations".into(), "0".into()]).is_err());
        assert!(parse(vec!["--calculation-timeout-s".into(), "0".into()]).is_err());
        assert!(parse(vec!["--queue-timeout-s".into(), "0".into()]).is_err());
        assert!(parse(vec!["--max-waiting".into(), "-1".into()]).is_err());
        assert!(parse(vec!["--max-waiting".into(), "70000".into()]).is_err());
        assert!(parse(vec!["--max-json-values".into(), "10".into()]).is_err());
    }

    #[test]
    fn json_value_cap_and_hosts() {
        let options = parse(vec![
            "--max-json-values".into(),
            "2000".into(),
            "--allowed-host".into(),
            "oes.local".into(),
        ])
        .unwrap();
        assert_eq!(options.http.max_json_values, 2000);
        // Loopback: the loopback names plus the configured one.
        assert!(options.bind.is_loopback());
        for host in ["localhost", "127.0.0.1", "::1", "oes.local"] {
            assert!(
                options.http.allowed_hosts.iter().any(|item| item == host),
                "{host}"
            );
        }
        // A non-loopback address checks only configured names.
        let open = parse(vec!["--bind".into(), "0.0.0.0".into()]).unwrap();
        assert!(open.http.allowed_hosts.is_empty());
        assert_eq!(
            open.http.max_json_values,
            nta8800_service::http::DEFAULT_MAX_JSON_VALUES
        );
    }
}
