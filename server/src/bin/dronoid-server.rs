#![forbid(unsafe_code)]

use clap::Parser;
use dronoid_server::{Rules, new_commands, persistence};
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tokio::signal;

use colored::Color;
use colored::Colorize;
use std::time::Instant;
use tracing::{
    Event, Level,
    field::{Field, Visit},
};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    layer::{Context, SubscriberExt},
};

#[derive(Default)]
struct MessageVisitor {
    message: String,
    sender: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "sender" {
            self.sender = value.to_string();
        }
    }
}

struct CustomLayer {
    time_mark: Instant,
}

impl CustomLayer {
    fn new() -> Self {
        Self {
            time_mark: Instant::now(),
        }
    }
}

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let color = match *event.metadata().level() {
            Level::TRACE => Color::BrightBlack,
            Level::DEBUG => Color::Green,
            Level::INFO => Color::White,
            Level::WARN => Color::Yellow,
            Level::ERROR => Color::Red,
        };

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let formatted = format!(
            "[{:>7.3}][{:^15}]{}",
            (Instant::now() - self.time_mark).as_secs_f32(),
            event.metadata().target(),
            visitor.message
        );
        println!("{}", formatted.color(color));
    }
}

pub fn init() {
    let _ = Registry::default()
        .with(CustomLayer::new())
        .with(EnvFilter::from_default_env())
        .try_init();
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = 8080)]
    port: u16,
    #[arg(long, default_value_t = dronoid_protocol::TERRAIN_SCALE)]
    terrain_scale: f32,
    #[arg(long, default_value_t = dronoid_protocol::MINERAL_THRESHOLD)]
    mineral_threshold: f32,
    #[arg(long, default_value_t = dronoid_protocol::TICK_DURATION)]
    tick_duration: f32,
    #[arg(long, default_value_t = dronoid_protocol::STARTING_MINERALS)]
    starting_minerals: u32,
    #[arg(long, default_value_t = dronoid_protocol::TERRAIN_SEED)]
    terrain_seed: u32,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // dronoid_logger::init();
    let tcp_listener = TcpListener::bind(SocketAddr::from_str(
        format!("127.0.0.1:{}", args.port).as_str(),
    )?)
    .await?;
    let rules = Rules {
        terrain_scale: args.terrain_scale,
        mineral_threshold: args.mineral_threshold,
        tick_duration: args.tick_duration,
        starting_minerals: args.starting_minerals,
        terrain_seed: args.terrain_seed,
        ..Default::default()
    };
    let database = persistence::Database::default();
    let (commands, controls) = new_commands();
    tokio::spawn(async move {
        #[cfg(target_os = "windows")]
        let mut ctrl_close = signal::windows::ctrl_close().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_logoff = signal::windows::ctrl_logoff().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown().unwrap();
        #[cfg(target_os = "windows")]
        tokio::select! {
            _ = signal::ctrl_c() => {}
            _ = ctrl_close.recv() => {}
            _ = ctrl_logoff.recv() => {}
            _ = ctrl_shutdown.recv() => {}
        }
        #[cfg(target_os = "linux")]
        let _ = signal::ctrl_c().await;
        let _ = commands.stop();
    });

    dronoid_server::run(rules, database, tcp_listener, controls).await?;

    anyhow::Ok(())
}
