use std::{
    io::{Read, Write},
    process::exit,
    str::FromStr,
};

use chrono::prelude::Local;
use clap::{arg, Command as ClapCommand};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::process::Command;
use toml::value::Datetime;
const FILE: &str = "~/.local/share/aptrb/transactions.toml";

#[derive(PartialEq)]
enum TType {
    Transaction,
    Rollback,
}

/// Struct that stores the transacion data
/// [TransactionData] is used to store the transactions in a e.g. toml file
///
/// * `packages`: packages that are part of the transaction
/// * `name`: name of the transaction if it has one or a timestamp
/// * `file`: file with the stored transaction data or the default file in $HOME/.lcoal/share/aptrb/transactions.toml
#[derive(Serialize, Deserialize)]
struct TransactionData {
    packages: Vec<String>,
    /// name is the name of the transaction if it has one or a timestamp
    name: Option<String>,
    /// time of the transaction
    timestamp: Datetime,
}

impl TransactionData {
    fn new() -> Self {
        let time: Datetime = Datetime::from_str(&current_time()).expect("Error while parsing time");
        TransactionData {
            timestamp: time,
            name: None,
            packages: vec![],
        }
    }
}

/// This is a struct that represents the command that will be executed in the command line
/// [TransactionCommand] can be converted to a [std::process::Command]
///
/// * `type_`: type of the command wether it should be a rollback or a transaction
/// * `apt_command`: base apt command like `apt-get`
/// * `apt_subcommand`: apt subcommand is dependen on the tranaction type and what you want to do
/// could be `purge` or `remove`
/// * `packages`: the packages that should be handled with the transaction
struct TransactionCommand {
    apt_command: String,
    apt_subcommand: String,
    packages: Vec<String>,
}

impl TransactionCommand {
    /// Creates a new [`TransactionCommand`].
    fn new(t: TType) -> Self {
        let subcom = if t == TType::Transaction {
            "install".to_string()
        } else {
            "purge".to_string()
        };
        TransactionCommand {
            apt_command: "apt-get".to_string(),
            apt_subcommand: subcom,
            packages: vec!["-y".to_string()],
        }
    }
    /// Returns an error if the spawn of the apt-get command fails else returns nothing
    fn to_cmd(&self) -> Command {
        let mut cmd = Command::new(&self.apt_command);
        cmd.arg(&self.apt_subcommand);
        cmd.args(&self.packages);
        cmd
    }

    fn add_packages(&mut self, packages: &mut Vec<String>) -> &mut Self {
        self.packages.append(packages);
        self
    }
}

/// return the current time in a formated string that can be parsed to toml datetime
fn current_time() -> String {
    let now = Local::now();
    let fmt_str = "%Y-%m-%dT%H:%M:%S%.6f";
    now.format(fmt_str).to_string()
}

fn get_command() -> ClapCommand {
    ClapCommand::new("aptrb")
        .version("0.1")
        .about("Rollback apt installed packages after installing and e.g. building a project")
        .subcommand(
            ClapCommand::new("rollback")
                .about("Rollback a specific or the lates transaciton")
                .visible_alias("r")
                .arg(arg!(-n --name <name> "Rollback name").required(false)),
        )
        .subcommand(
            ClapCommand::new("transaction")
                .visible_alias("t")
                .about("Start a new transaciton")
                .arg(arg!(<packages>... "Packeges of the transaction").required(true))
                .arg(
                    arg!(-f --file <file> "File to rollback from or to save the rollback info in")
                        .required(false),
                )
                .arg(arg!(-n --name <name> "Optional Name for a transaction")),
        )
}

fn main() {
    env_logger::init();
    // todo: feature do we want to give a rollback a name like "optee project"
    // usage would be aptrb
    let app = get_command();
    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("rollback", rollback_matches)) => {
            if let Some(name) = rollback_matches.get_one::<String>("name") {
                println!("Rollback name: {}", name);
            } else {
                println!("No rollback name provided");
            }
        }
        Some(("transaction", transaction_matches)) => {
            let mut transaction = TransactionData::new();
            let mut cmd = TransactionCommand::new(TType::Transaction);
            let mut file = FILE;
            let packages = transaction_matches
                .get_one::<Vec<String>>("packages")
                .expect("There have to be some messages");

            info!("packages: {:?}", packages);
            cmd.add_packages(&mut packages.to_vec());

            if let Some(f) = transaction_matches.get_one::<String>("file") {
                file = &f;
            }
            if let Some(transaction_name) = transaction_matches.get_one::<String>("name") {
                transaction.name = Some(transaction_name.to_string());
            }
            transaction.packages = packages.to_vec();
            cmd.packages = packages.to_vec();

            // vvv todo: this vvv
            // generate the command
            let mut syscmd = cmd.to_cmd();
            //  execute the command
            let res = syscmd.spawn();
            // command was executed successfully
            match res {
                Ok(c) => {
                    let mut buf = String::new();
                    let _ = c.stdout.unwrap().read_to_string(&mut buf);
                    info!("{}", &buf);
                }
                Err(e) => {
                    error!(
                        "There was an error executing this file!\n {}",
                        e.to_string()
                    );
                    exit(1);
                }
            }
            // so we write the transaction to the file
            info!("Writing the packages to the file.");
            let mut fd = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file)
                .expect("Could not open file");
            let toml = toml::to_string(&transaction).expect("Could not serialize to toml");
            fd.write_all(toml.as_bytes())
                .expect("Could not write to file");
        }
        _ => unreachable!(), // Should never happen due to clap's built-in validation
    }

    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn test_transaction() {
        let app = get_command();
        let matches = app.get_matches_from(vec!["aptrb", "transaction", "package1", "package2"]);
        match matches.subcommand() {
            Some(("transaction", transaction_matches)) => {
                let packages = transaction_matches
                    .get_many::<String>("packages")
                    .expect("There have to be some messages");
                assert!(packages.len() == 2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_rollback() {
        let app = get_command();
        let matches = app.get_matches_from(vec!["aptrb", "rollback", "-n", "optee"]);
        match matches.subcommand() {
            Some(("rollback", rollback_matches)) => {
                if let Some(name) = rollback_matches.get_one::<String>("name") {
                    assert_eq!(name, "optee");
                } else {
                    panic!("No rollback name provided");
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_current_time() {
        // get current time
        // try to parse it with to toml datetime
        let time = current_time();
        let _ = Datetime::from_str(&time).expect("Error while parsing time");
    }

    #[test]
    fn test_transaction_command() {
        let cmd = TransactionCommand::new(TType::Transaction);
        let cmd = cmd.to_cmd();
        assert_eq!(cmd.get_program(), "apt-get");
        assert_eq!(
            *cmd.get_args().collect::<Vec<&OsStr>>().first().unwrap(),
            "install"
        );
    }

    #[test]
    fn test_rollback_command() {
        let mut cmd = TransactionCommand::new(TType::Rollback);
        let mut packages = vec!["package1".to_string(), "package2".to_string()];
        cmd.add_packages(&mut packages);
        let cmd = cmd.to_cmd();
        assert_eq!(cmd.get_program(), "apt-get");
        let mut iter = cmd.get_args().collect::<Vec<&OsStr>>().into_iter();
        assert_eq!(iter.next().unwrap(), "purge");
        // this is that we don't have to interact
        assert_eq!(iter.next().unwrap(), "-y");
        assert_eq!(iter.next().unwrap(), "package1");
        assert_eq!(iter.next().unwrap(), "package2");
    }
}
