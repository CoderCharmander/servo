use crate::cli;
use clap::ArgMatches;
use config::ServerVersion;
use servo::{
    config,
    errors::*,
    paper_api,
    servers::{iter_servers_directory, CachedJar, Server},
};
use std::{fs, io::BufRead};

pub fn download(args: &ArgMatches) -> Result<()> {
    let version = ServerVersion::new(args.value_of("VERSION").unwrap())?;

    let output = format!(
        "paper-{}.{}.{}.jar",
        version.minecraft.0, version.minecraft.1, version.minecraft.2
    );
    let output = args.value_of("output").unwrap_or(&output);

    println!("Downloading version {} into {}", version, output);

    let mut file = fs::File::create(output).chain_err(|| "could not create jar file")?;
    paper_api::ProjectVersionList::download(&version, &mut file)?;

    Ok(())
}

pub fn create(args: &ArgMatches) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let version = args.value_of("VERSION").unwrap();
    Server::create(&name, config::ServerVersion::new(&version)?)?;
    Ok(())
}

pub fn start(args: &ArgMatches) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let server = Server::get(&name)?;
    let jar = CachedJar::download(server.config.version)?;
    let mut child = jar.start_server(server)?;
    child.wait().chain_err(|| "wait failed")?;
    Ok(())
}

pub fn list() -> Result<()> {
    for dir in iter_servers_directory()? {
        let server = Server::get(
            dir.chain_err(|| "failed reading directory")?
                .file_name()
                .to_str()
                .ok_or_else(|| Error::from("invalid character in server name"))?,
        )?;
        println!(
            "{} ({})",
            server.config.name,
            cli::SECONDARY.paint(format!("{}", server.config.version))
        );
    }

    Ok(())
}

pub fn remove(args: &ArgMatches) -> Result<()> {
    let server = Server::get(&args.value_of("NAME").unwrap())?;
    let confirm_str = format!(
        "Yes, erase {} completely and irrecoverably.",
        server.config.name
    );
    println!("{} THIS WILL IRRECOVERABLY ERASE {}, ALL OF ITS CONFIGURATION AND WORLDS! TO CONTINUE, TYPE \"{}\".",
        cli::WARNING_HEADER_STYLE.paint("WARNING!"),
        cli::SECONDARY.paint(&server.config.name),
        cli::SECONDARY.paint(&confirm_str)
    );
    let input = std::io::stdin().lock().lines().next().unwrap().unwrap();
    if confirm_str == input {
        std::fs::remove_dir_all(server.server_path()?)
            .chain_err(|| "failed to remove server directory")?;
        std::fs::remove_file(server.config_path()?)
            .chain_err(|| "failed to remove server config")?;
    } else {
        println!("Abort.");
    }
    Ok(())
}
