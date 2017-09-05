use globals;

use std::str;
use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;

use futures::{Future, Stream};
use hyper;
use hyper::{Request, Method, Client};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use hyper::header::{UserAgent};

use serde_json;
use serde_json::{Value, Map};

// https://hyper.rs/hyper/master/hyper/header/index.html
header! { (VaultToken, "X-Vault-Token") => [String] }

fn read_file_to_string(infile: &Path) -> String {
    let mut file = File::open(infile).expect(format!("Error opening file '{:?}'.", infile).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(format!("Error reading file '{:?}'.", infile).as_str());
    contents
}

fn get_vault_token() -> Result<String, String> {
    match env::var("VAULT_TOKEN") {
        Ok(token) => Ok(token),
        Err(_) => {
            let mut search_path: Vec<PathBuf> = Vec::new();

            match env::current_dir() {
                Ok(cwd) => search_path.push(cwd),
                Err(_) => {}
            }

            match env::home_dir() {
                Some(home) => search_path.push(home),
                None => {}
            }

            for pathbuf in search_path {
                let token_path = pathbuf.join(".vault-token");
                if token_path.exists() {
                    return Ok(read_file_to_string(token_path.as_path()));
                }
            }

            Err("No token found".to_string())
        }
    }
}


pub fn fetch_objects_from_vault(vault_path: &String) -> Result<Map<String, Value>, String> {
    let vault_endpoint: String = env::var("VAULT_ADDR").expect("VAULT_ADDR not set, could not determine Vault endpoint.");
    let object_url: String = format!("{}/v1/{}", vault_endpoint, vault_path);

    let mut core = Core::new().unwrap();
    let client = Client::configure()
                .connector(HttpsConnector::new(4, &core.handle()).unwrap())
                .build(&core.handle());

    let uri: hyper::Uri = object_url.parse().unwrap();

    let mut req: Request = Request::new(Method::Get, uri);
    req.headers_mut().set(UserAgent::new(globals::APP_NAME.to_string()));
    req.headers_mut().set(VaultToken(get_vault_token().unwrap()));

    let work = client.request(req).and_then(|response| response.body().concat2());

    let result = core.run(work).unwrap();
    let data: &str = str::from_utf8(&result).unwrap();

    let json: Map<String, Value> = serde_json::from_str(data).expect("Received invalid JSON from Vault.");

    Ok(json)
}
