extern crate hyper;
extern crate regex;
extern crate url;

use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;
use std::io::Read;
use regex::Regex;
use url::percent_encoding::percent_decode;

struct App {
    hyper_server: hyper::server::Server,
}

impl App {
    fn new(listen_addr: &str) -> Result<App, hyper::Error> {
        match Server::http(listen_addr) {
            Ok(server) => Ok(App { hyper_server: server }),
            Err(err) => Err(err),
        }
    }

    fn register_route<H>(self, route: String, handler: H) where H: Fn(Request, Response, Option<String>) + std::marker::Sync + std::marker::Send  + 'static {
        self.hyper_server.handle(move |req: Request, res: Response| {
            let uri = match req.uri.clone() {
                RequestUri::AbsolutePath(uri) => uri,
                _ => panic!(),
            };
            let route = "^/".to_string() + &route + "$";
            let re = Regex::new(&route).unwrap();

            let route_match = if re.is_match(&uri) {
                let percent_encoded_name = re.captures(&uri).unwrap().at(1).unwrap();
                Some(percent_decode(percent_encoded_name.as_bytes()).decode_utf8().unwrap().to_string())
            } else {
                None
            };

            handler(req, res, route_match);
        }).unwrap();
    }
}

fn main() {
    let app = App::new("0.0.0.0:5000").unwrap();
    app.register_route("(.+)".to_string(), move |_req, res, name| {
        let response_body = match name {
            Some(name) => "Hello, ".to_string() + &name + "!",
            None => "Hello, World!".to_string(),
        };
        res.send(response_body.as_bytes()).unwrap();
    });
}

#[test]
fn integration_test() {
    std::thread::spawn(move || {
        main();
    });

    let client = hyper::client::Client::new();

    let mut res = client.get("http://localhost:5000/").send().unwrap();
    let mut response_body = String::new();
    res.read_to_string(&mut response_body).unwrap();

    assert_eq!(response_body, "Hello, World!");

    let mut res = client.get("http://localhost:5000/mike").send().unwrap();
    let mut response_body = String::new();
    res.read_to_string(&mut response_body).unwrap();

    assert_eq!(response_body, "Hello, mike!");

    let mut res = client.get("http://localhost:5000/jœ").send().unwrap();
    let mut response_body = String::new();
    res.read_to_string(&mut response_body).unwrap();

    assert_eq!(response_body, "Hello, jœ!");
}
