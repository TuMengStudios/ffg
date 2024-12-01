use std::process::exit;

use clap::{arg, Command};

use rg::biz::CommandAction;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    // std::env::set_var("RG_MIRROR", "https://studygolang.com");
    let matches = Command::new("rg")
        .version("0.0.1")
        .author("2356450144@qq.com")
        .about("golang multi version manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("info").arg(arg!(--hello <VALUE>).help("String").required(true)))
        .subcommand(
            Command::new("use")
                .args([arg!([version]).help("use golang version").required(true)])
                .about("use golang version"),
        )
        .subcommand(Command::new("ls").about("list local go version"))
        .subcommand(
            Command::new("rm")
                .args([arg!([version]).help("remove version").required(true)])
                .about("remove go version"),
        )
        .subcommand(Command::new("ls-remote").about("list remote go version"))
        .subcommand(Command::new("about").about("print command info"))
        .get_matches();

    match matches.subcommand() {
        Some(("info", sub_matches)) => {
            println!("info  abc {:?}", sub_matches.get_one::<String>("hello"));
            exit(0);
        }
        Some(("use", sub_matches)) => {
            let default_version = String::new();
            let version = sub_matches
                .get_one::<String>("version")
                .unwrap_or(&default_version);

            // biz::download("https://studygolang.com/dl/golang/go1.22.9.linux-amd64.tar.gz").await?;
            CommandAction::use_action(version).await?;
        }
        Some(("ls", _sub_matches)) => {
            CommandAction::ls().await?;
        }
        Some(("ls-remote", _sub)) => {
            CommandAction::ls_remote().await?;
        }
        Some(("rm", sub_matches)) => {
            let default_version = String::new();
            let version = sub_matches
                .get_one::<String>("version")
                .unwrap_or(&default_version);
            // println!("remove version {}", version.bold().red(),);
            // println!("")
            CommandAction::rm(version).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
