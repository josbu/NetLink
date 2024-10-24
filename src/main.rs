use clap::Parser;
use env_logger::Env;
use std::future::Future;

use anyhow::anyhow;

use clap::error::ErrorKind;

use std::net::Ipv4Addr;
use std::str::FromStr;

use crate::config::{ConfigView, FileConfigView};
use crate::ipc::service::ApiService;

use rustp2p::protocol::node_id::GroupCode;

mod cipher;
mod config;
mod exit_route;
mod ipc;
mod netlink_task;
mod platform;
mod route_listen;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Peer node address.
    /// e.g.: -p tcp://192.168.10.13:23333 -p udp://192.168.10.23:23333
    #[arg(short, long)]
    peer: Vec<String>,
    /// Local node IP and prefix.If there is no 'prefix', Will not enable Tun.
    /// e.g.: -l 10.26.0.2/24
    #[arg(short, long, value_name = "LOCAL IP")]
    local: String,
    /// Nodes with the same group_code can form a network (Maximum length 16).
    #[arg(short, long, value_name = "GROUP CODE")]
    group_code: String,
    /// Listen local port, default is 23333
    #[arg(short = 'P', long)]
    port: Option<u16>,
    /// Bind the outgoing network interface (using the interface name).
    /// e.g.: -b eth0
    #[arg(short, long, value_name = "DEVICE NAME")]
    bind_dev: Option<String>,
    /// Set the number of threads, default is 2
    #[arg(long)]
    threads: Option<usize>,
    /// Enable data encryption.
    /// e.g.: -e "password"
    #[arg(short, long, value_name = "PASSWORD")]
    encrypt: Option<String>,
    /// Set encryption algorithm. Optional aes-gcm/chacha20-poly1305/xor, default is chacha20-poly1305
    #[arg(short, long)]
    algorithm: Option<String>,
    /// Global exit node,please use it together with '--bind-dev'
    #[arg(long)]
    exit_node: Option<String>,
    /// Set tun name
    #[arg(long)]
    tun_name: Option<String>,
    /// Start using configuration file
    #[arg(short = 'f', long)]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
struct ArgsConfig {
    #[arg(short = 'f', long)]
    config: String,
}

#[derive(Parser, Debug)]
struct ArgsBack {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Backend command
    Cmd {
        /// set backend server host. default 127.0.0.1
        #[arg(long)]
        cmd_host: Option<String>,
        /// When opening multiple programs, this port needs to be set. default 23336
        #[arg(long)]
        cmd_port: Option<u16>,
        /// View information about the current program
        #[arg(long)]
        info: bool,
        /// View all nodes in the current group
        #[arg(long)]
        nodes: bool,
        /// View all group codes
        #[arg(long)]
        groups: bool,
        /// View all nodes in the group code
        #[arg(long)]
        others: Option<String>,
    },
}

const CMD_HOST: &str = "127.0.0.1";
const CMD_PORT: u16 = 23336;
const LISTEN_PORT: u16 = 23333;

pub fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = match Args::try_parse() {
        Ok(arg) => arg,
        Err(e) => {
            if e.kind() == ErrorKind::DisplayHelp {
                println!("{e}");
                return Ok(());
            }
            if let Ok(args) = ArgsBack::try_parse() {
                return client_cmd(args);
            }
            if let Ok(args) = ArgsConfig::try_parse() {
                let file_config = FileConfigView::read_file(&args.config)?;
                let worker_threads = file_config.threads;
                return block_on(worker_threads, main_by_config_file(file_config));
            }
            println!("{e}");
            return Ok(());
        }
    };
    let worker_threads = args.threads.unwrap_or(2);
    block_on(worker_threads, main_by_cmd(args))
}

fn block_on<F: Future>(worker_threads: usize, f: F) -> F::Output {
    if worker_threads <= 1 {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn client_cmd(args: ArgsBack) -> anyhow::Result<()> {
    let Commands::Cmd {
        cmd_host,
        cmd_port,
        info,
        nodes,
        groups,
        others,
    } = args.command;
    let host = cmd_host.unwrap_or(CMD_HOST.to_string());
    let port = cmd_port.unwrap_or(CMD_PORT);
    let addr = format!("{host}:{port}");
    if nodes {
        if let Err(e) = ipc::udp::client::nodes(addr).await {
            Err(anyhow!("Perhaps the backend service has not been started. Use '--cmd-port' to change the port. error={e}"))?;
        }
    } else if groups {
        if let Err(e) = ipc::udp::client::groups(addr).await {
            Err(anyhow!("Perhaps the backend service has not been started. Use '--cmd-port' to change the port. error={e}"))?;
        }
    } else if let Some(group_code) = others {
        if let Err(e) = ipc::udp::client::other_nodes(addr, group_code).await {
            Err(anyhow!("Perhaps the backend service has not been started. Use '--cmd-port' to change the port. error={e}"))?;
        }
    } else if info {
        if let Err(e) = ipc::udp::client::current_info(addr).await {
            Err(anyhow!("Perhaps the backend service has not been started. Use '--cmd-port' to change the port. error={e}"))?;
        }
    } else {
        Err(anyhow!("Use specific commands to view data"))?;
    }
    Ok(())
}

async fn main_by_cmd(args: Args) -> anyhow::Result<()> {
    let Args {
        peer,
        local,
        group_code,
        port,
        bind_dev,
        encrypt,
        algorithm,
        exit_node,
        tun_name,
        command,
        ..
    } = args;
    let mut split = local.split('/');
    let self_id = Ipv4Addr::from_str(split.next().expect("--local error")).expect("--local error");
    let prefix = if let Some(mask) = split.next() {
        u8::from_str(mask).expect("unable to parse the prefix in the arguments for --local")
    } else {
        0
    };
    let config_view = ConfigView {
        group_code,
        node_ipv4: format!("{self_id}"),
        prefix,
        node_ipv6: None,
        tun_name,
        encrypt,
        algorithm,
        port: port.unwrap_or(LISTEN_PORT),
        peer,
        bind_dev_name: bind_dev,
        exit_node,
        ..ConfigView::default()
    };
    let addr = if let Some(Commands::Cmd {
        cmd_host, cmd_port, ..
    }) = command
    {
        format!(
            "{}:{}",
            cmd_host.unwrap_or(CMD_HOST.to_string()),
            cmd_port.unwrap_or(CMD_PORT)
        )
    } else {
        format!("{CMD_HOST}:{CMD_PORT}")
    };
    start_by_config(config_view, addr).await?;
    Ok(())
}

async fn main_by_config_file(file_config: FileConfigView) -> anyhow::Result<()> {
    let addr = format!("{}:{}", file_config.cmd_host, file_config.cmd_port);
    let config_view = ConfigView::from(file_config);
    start_by_config(config_view, addr).await
}

async fn start_by_config(config_view: ConfigView, cmd_server_addr: String) -> anyhow::Result<()> {
    let config = config_view.into_config()?;

    let api_service = ApiService::new(config);
    if let Err(e) = ipc::server_start(cmd_server_addr, api_service.clone()).await {
        return Err(anyhow!("The backend command port has already been used. Please use '--cmd-port' to change the port, err={e}"));
    }
    let (tx, mut quit) = tokio::sync::mpsc::channel::<()>(1);

    ctrlc2::set_async_handler(async move {
        let _ = tx.send(()).await;
    })
    .await;

    netlink_task::start_netlink(&api_service).await?;
    _ = quit.recv().await;
    _ = api_service.close();
    log::info!("exit!!!!");
    Ok(())
}

fn string_to_group_code(input: &str) -> Option<GroupCode> {
    let mut array = [0u8; 16];
    let bytes = input.as_bytes();
    if bytes.len() > 16 {
        return None;
    }
    let len = bytes.len();
    array[..len].copy_from_slice(&bytes[..len]);
    Some(array.into())
}
