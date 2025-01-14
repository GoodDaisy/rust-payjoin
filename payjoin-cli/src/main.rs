use anyhow::{Context, Result};
use clap::{arg, value_parser, Arg, ArgMatches, Command};

mod app;
use app::{App, AppConfig};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = cli();
    let config = AppConfig::new(&matches)?;
    let app = App::new(config)?;

    match matches.subcommand() {
        Some(("send", sub_matches)) => {
            let bip21 = sub_matches.get_one::<String>("BIP21").context("Missing BIP21 argument")?;
            let fee_rate_sat_per_vb =
                sub_matches.get_one::<f32>("fee_rate").context("Missing --fee-rate argument")?;
            #[cfg(feature = "v2")]
            let is_retry = matches.get_one::<bool>("retry").context("Could not read --retry")?;
            #[cfg(feature = "v2")]
            app.send_payjoin(bip21, fee_rate_sat_per_vb, *is_retry).await?;
            #[cfg(not(feature = "v2"))]
            app.send_payjoin(bip21, fee_rate_sat_per_vb).await?;
        }
        Some(("receive", sub_matches)) => {
            let amount =
                sub_matches.get_one::<String>("AMOUNT").context("Missing AMOUNT argument")?;
            #[cfg(feature = "v2")]
            let is_retry = matches.get_one::<bool>("retry").context("Could not read --retry")?;
            #[cfg(feature = "v2")]
            app.receive_payjoin(amount, *is_retry).await?;
            #[cfg(not(feature = "v2"))]
            app.receive_payjoin(amount).await?;
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }

    Ok(())
}

fn cli() -> ArgMatches {
    Command::new("payjoin")
        .about("Transfer bitcoin and preserve your privacy")
        .arg(Arg::new("rpchost")
            .long("rpchost")
            .short('r')
            .takes_value(true)
            .help("The port of the bitcoin node"))
        .arg(Arg::new("cookie_file")
            .long("cookie-file")
            .short('c')
            .takes_value(true)
            .help("Path to the cookie file of the bitcoin node"))
        .arg(Arg::new("rpcuser")
            .long("rpcuser")
            .takes_value(true)
            .help("The username for the bitcoin node"))
        .arg(Arg::new("rpcpass")
            .long("rpcpass")
            .takes_value(true)
            .help("The password for the bitcoin node"))
        .subcommand_required(true)
        .arg(Arg::new("ohttp_config")
            .long("ohttp-config")
            .help("The ohttp config file"))
        .arg(Arg::new("ohttp_proxy")
            .long("ohttp-proxy")
            .help("The ohttp proxy url"))
        .arg(Arg::new("retry")
            .long("retry")
            .short('e')
            .action(clap::ArgAction::SetTrue)
            .help("Retry the asynchronous payjoin request if it did not yet complete"))
        .subcommand(
            Command::new("send")
                .arg_required_else_help(true)
                .arg(arg!(<BIP21> "The `bitcoin:...` payjoin uri to send to"))
                .arg_required_else_help(true)
                .arg(Arg::new("fee_rate")
                    .long("fee-rate")
                    .value_name("FEE_SAT_PER_VB")
                    .help("Fee rate in sat/vB")
                    .value_parser(value_parser!(f32)),
                )
        )
        .subcommand(
            Command::new("receive")
                .arg_required_else_help(true)
                .arg(arg!(<AMOUNT> "The amount to receive in satoshis"))
                .arg_required_else_help(true)
                .arg(Arg::new("port")
                    .long("host-port")
                    .short('p')
                    .takes_value(true)
                    .help("The local port to listen on"))
                .arg(Arg::new("endpoint")
                    .long("endpoint")
                    .short('e')
                    .takes_value(true)
                    .help("The `pj=` endpoint to receive the payjoin request"))
                .arg(Arg::new("sub_only")
                    .long("sub-only")
                    .short('s')
                    .action(clap::ArgAction::SetTrue)
                    .hide(true)
                    .help("Use payjoin like a payment code, no hot wallet required. Only substitute outputs. Don't contribute inputs."))
        )
        .get_matches()
}
