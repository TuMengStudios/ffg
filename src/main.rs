use clap::{arg, Command};

use ffg::biz::CommandAction;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    let matches = Command::new("ffg")
        .version("0.1.4")
        .author("2356450144@qq.com")
        .about("a golang multi version manager tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
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
        .subcommand(
            Command::new("ins")
                .args([arg!([version]).help("install version").required(true)])
                .about("install specific version"),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("use", sub_matches)) => {
            let default_version = String::new();
            let version = sub_matches
                .get_one::<String>("version")
                .unwrap_or(&default_version);

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

            CommandAction::rm(version).await?;
        }
        Some(("ins", sub_matches)) => {
            let default_version = String::new();
            let version = sub_matches
                .get_one::<String>("version")
                .unwrap_or(&default_version);

            CommandAction::ins(version).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
