use action::ActionSender;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use nx::Nx;
use tokio::sync::broadcast;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod nx;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();

    let (action_tx, action_rx) = broadcast::channel(100);
    let action_tx: ActionSender = action_tx.into();

    let (mut nx, _command_log_rx) = Nx::new(args.path, action_tx.clone(), action_tx.subscribe());
    let mut app = App::new(args.tick_rate, args.frame_rate, action_tx, action_rx)?;

    tokio::select! {
        res = app.run() => {
            res
        },
        res = nx.run() => {
            res
        },
    }
}
