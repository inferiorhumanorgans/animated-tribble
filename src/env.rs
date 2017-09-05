use vault::*;

use std::borrow::Cow;

use serde_json::{Value, Map};

use clap::{App, Arg, ArgMatches, SubCommand};

use shell_escape;

pub fn init_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("env")
                            .about("Fetch values from Vault and output commands to populate the shell's environment")
                            .arg(Arg::with_name("prefix")
                                 .short("p")
                                 .long("prefix")
                                 .takes_value(true)
                                 .help("String to prepend to environment variable names"))
                            .arg(Arg::with_name("path")
                                 .help("vault path")
                                 .index(1)
                                 .required(true))
}

pub fn do_subcommand(matches: &ArgMatches) {
    let prefix: String = match matches.value_of("prefix") {
        Some(value) => format!("{}_", value),
        None => "".to_string()
    };

    let vault_path = matches.value_of("path").unwrap().to_string();

    match fetch_objects_from_vault(&vault_path) {
        Ok(json) => {
            if json.contains_key("errors") {
                panic!("Errors from Vault: {}.", json["errors"]);
            }

            let m: &Map<String, Value> = json["data"].as_object().expect("Expected 'data' to be an object.");

            for (key, val) in m.iter() {
                let data: Cow<str> = Cow::Owned(val.as_str().unwrap().to_string());
                let env_name: String = format!("{}{}", prefix, key).to_uppercase();
                println!("export {}={};\n", env_name, shell_escape::unix::escape(data));
            };
        },
        Err(_) => {}
    }
}
