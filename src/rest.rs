use vault::*;

use serde_json::{Value, Map};

use clap::{App, Arg, ArgMatches, SubCommand};

use std::str;

use futures::{Future, Stream};
use hyper;
use hyper::{Request, Method, Client};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

use std::path::{Path};
use std::fs::File;
use std::io::prelude::*;

header! { (ApiKey, "X-API-Key") => [String] }

fn read_file_to_string(infile: &Path) -> String {
    let mut file = File::open(infile).expect(format!("Error opening file '{:?}'.", infile).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(format!("Error reading file '{:?}'.", infile).as_str());
    return contents;
}

fn get_api_endpoint(vault_data: &Map<String, Value>, args: &ArgMatches) -> hyper::Uri {
    let api_path: String = value_t!(args, "rest path", String).unwrap();
    let api_endpoint: &str = vault_data["url"].as_str().unwrap();
    let object_url: String = format!("{}/{}", api_endpoint, api_path);
    return object_url.parse().unwrap();
}

fn get_api_token(vault_data: &Map<String, Value>) -> String {
    let token = vault_data["token"].as_str().unwrap().to_string();
    return token;
}

fn get_http_method(args: &ArgMatches) -> hyper::Method {
    match args.value_of("HTTP method") {
        Some("GET")   => Method::Get,
        Some("POST")  => Method::Post,
        Some("HEAD")  => Method::Head,
        None          => {
            if (args.occurrences_of("body") == 1) || (args.occurrences_of("body-file") == 1) {
                Method::Post
            } else {
                Method::Get
            }
        },
        _             => unreachable!()
    }
}

fn get_body(args: &ArgMatches) -> Option<String> {
    if args.occurrences_of("body-file") == 1 {
        let body_path: &str = args.value_of("body-file").unwrap();
        let body: String = read_file_to_string(Path::new(body_path));
        return Some(body);
    } else if args.occurrences_of("body") == 1 {
        let body: String = args.value_of("body").unwrap().to_string();
        return Some(body);
    };

    None
}

pub fn init_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("rest")
                           .about("Fetch API keys and endpoint from Vault and make a REST call")
                           .arg(Arg::with_name("HTTP method")
                                     .help("HTTP method")
                                     .short("x")
                                     .long("method")
                                     .takes_value(true)
                                     .possible_values(&["GET", "POST", "HEAD"]))
                           .arg(Arg::with_name("body")
                                     .short("b")
                                     .long("body")
                                     .takes_value(true))
                           .arg(Arg::with_name("body-file")
                                     .short("f")
                                     .long("body-file")
                                     .takes_value(true))
                           .arg(Arg::with_name("vault path")
                                     .help("vault path")
                                     .index(1)
                                     .required(true))
                           .arg(Arg::with_name("rest path")
                                     .help("rest path")
                                     .index(2)
                                     .required(true))
}

pub fn do_subcommand(matches: &ArgMatches) {
    let vault_path: String = value_t!(matches, "vault path", String).unwrap();


    let vault_data: Map<String, Value> = match fetch_objects_from_vault(&vault_path) {
        Ok(json) => {
            if json.contains_key("errors") {
                panic!("Errors from Vault: {}.", json["errors"]);
            }
            json["data"].as_object().expect("Expected 'data' to be an object.").clone()
        },
        Err(e)   => panic!(e)
    };

    let mut core = Core::new().unwrap();
    let client = Client::configure()
                .connector(HttpsConnector::new(4, &core.handle()).unwrap())
                .build(&core.handle());

    let uri: hyper::Uri = get_api_endpoint(&vault_data, matches);
    let api_token: String = get_api_token(&vault_data);
    let http_method: hyper::Method = get_http_method(matches);

    let mut req: Request = Request::new(http_method, uri);
    req.headers_mut().set(ApiKey(api_token));

    let body: Option<String> = get_body(matches);
    match body {
        Some(body)  => req.set_body(body),
        None        => {}
    }

    let work = client.request(req).and_then(|response| response.body().concat2());

    let result = core.run(work).unwrap();
    let data: &str = str::from_utf8(&result).unwrap();
    println!("{}", data);
}
