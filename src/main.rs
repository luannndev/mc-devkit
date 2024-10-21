use std::fs;
use std::path::PathBuf;
use std::process::exit;
use clap::{CommandFactory, Parser, Subcommand};
use libtermcolor::colors;
use crate::server::Software;
use crate::server_manager::check_valid_version;

mod server;
mod server_manager;

#[derive(Parser, Debug)]
#[command(about, long_about, name = "mcdevkit", version)]
#[command(author = "luannndev")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Start {
        #[arg(required = true, value_enum)]
        software: Software,

        #[arg(required = true)]
        version: String,

        #[arg()]
        plugins: Vec<PathBuf>,

        #[arg(short, long, default_value = "none")]
        working_directory: PathBuf,

        #[arg(short, long)]
        args: Vec<String>,

        #[arg(short, long, default_value = "2048")]
        mem: u32,

        #[arg(short, long)]
        gui: bool,

        #[arg(short, long, default_value = "25565")]
        port: u16,

        #[arg(short, long)]
        debug: bool
    }
}

pub fn send_info(msg: String) {
    println!("{}[{}MC-SDK{}]{} {}{}", colors::bright_black().regular, colors::bright_green().regular, colors::bright_black().regular, colors::bright_green().regular, msg, colors::reset())
}

pub fn send_debug(msg: String) {
    println!("{}[{}Debug{}]{} {}{}", colors::bright_black().regular, colors::bright_yellow().regular, colors::bright_black().regular, colors::bright_yellow().regular, msg, colors::reset())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Some(Commands::Start { software, version, plugins, working_directory, mut args, mem, gui, port, debug }) = args.command {
        if !check_valid_version(&version).await {
            exit(1)
        }

        if !gui {
            args.push("--nogui".to_string())
        }

        if port != 25565 {
            args.push(format!("--port={}", port))
        }

        if working_directory != PathBuf::from("none") {
            if !working_directory.exists() {
                if let Err(err) = fs::create_dir(working_directory.clone()) { eprintln!("Error creating directory: {}", err) }
            }

            if !working_directory.is_dir() {
                eprintln!("Error: You need to specify a Directory not a file");
                exit(1)
            }
        }

        if debug {
            send_debug(format!("Software: {:?}", software));
            send_debug(format!("Version: {}", version));
            send_debug("Args: ".parse().unwrap());
            for arg in args.clone() {
                println!(" > {}{}", colors::bright_yellow().regular, arg);
            }

            send_debug("Plugins: ".parse().unwrap());
            for plugin in plugins.clone() {
                println!(" > {}{}", colors::bright_yellow().regular, plugin.file_name().unwrap().to_str().unwrap());
            }
        }

        let mut server = server::Server {
            wd: working_directory,
            software,
            version,
            plugins,
            args,
            mem,
        };

        server.init_server().await;
        if let Err(err) = server.start_server().await {
            eprintln!("Error starting server: {}", err);
            exit(1);
        }

        println!("\n");
        send_info("Server Stopped.".to_string())
    }

    if std::env::args().len() == 1 {
        Args::command().print_help().unwrap();
        exit(0);
    }
    exit(0)
}