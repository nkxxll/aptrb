use clap::{arg, Command};

enum TType {
    Transaction,
    Rollback,
}

/// Struct that stores the transacion data
/// [TransactionData] is used to store the transactions in a e.g. toml file
///
/// * `packages`: [TODO:parameter]
struct TransactionData {
    packages: Vec<String>,
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
    type_: TType,
    apt_command: String,
    apt_subcommand: String,
    packages: Vec<String>,
}

fn main() {
    // todo: feature do we want to give a rollback a name like "optee project"
    // usage would be aptrb
    let app = Command::new("aptrb")
        .version("0.1")
        .about("Rollback apt installed packages after installing and e.g. building a project")
        .subcommand(
            Command::new("rollback")
                .about("Rollback a specific or the lates transaciton")
                .visible_alias("r")
                .arg(arg!(-n --name <name> "Rollback name").required(false)),
        )
        .subcommand(
            Command::new("transaction")
                .visible_alias("t")
                .about("Start a new transaciton")
                .arg(arg!([PACKAGES]...).required(true))
                .arg(
                    arg!(-f --file <file> "File to rollback from or to save the rollback info in")
                        .required(false),
                )
                .arg(arg!(-n --name <name> "Optional Name for a transaction")),
        )
        .get_matches();

    match app.subcommand() {
        Some(("rollback", rollback_matches)) => {
            if let Some(name) = rollback_matches.get_one::<String>("name") {
                println!("Rollback name: {}", name);
            } else {
                println!("No rollback name provided");
            }
        }
        Some(("transaction", transaction_matches)) => {
            let packages = transaction_matches
                .get_one::<Vec<String>>("packages")
                .expect("There have to be some messages");
            if let Some(file) = transaction_matches.get_one::<String>("file") {}
            if let Some(transaction_name) = transaction_matches.get_one::<String>("name") {}

            println!("Packages: {:?}", packages);
            println!("File: {:?}", file);
            println!("Transaction Name: {:?}", transaction_name);
        }
        _ => unreachable!(), // Should never happen due to clap's built-in validation
    }

    println!("Hello, world!");
}
