extern crate hyper;
extern crate regex;
extern crate url;

use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;
use std::io::Read;
use regex::Regex;
use url::percent_encoding::percent_decode;

fn hello(req: Request, res: Response) {
    let uri = match req.uri {
        RequestUri::AbsolutePath(uri) => uri,
        _ => panic!(),
    };
    let re = Regex::new(r"^/(.+)$").unwrap();

    let name = if re.is_match(&uri) {
        let percent_encoded_name = re.captures(&uri).unwrap().at(1).unwrap();
        percent_decode(percent_encoded_name.as_bytes()).decode_utf8().unwrap().to_string()
    } else {
        "World".to_string()
    };

    let response_body = "Hello, ".to_string() + &name + "!";
    res.send(response_body.as_bytes()).unwrap();
}

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

    fn handle<H: hyper::server::Handler + 'static>(self, handler: H) ->
        hyper::Result<hyper::server::Listening> {
        self.hyper_server.handle(handler)
    }
}

fn main() {
    let app = App::new("0.0.0.0:5000").unwrap();
    app.handle(hello).unwrap();
    // TODO:
    // app.register_route("(.*)", |req, res, name| {
    //     let response_body = match name {
    //         Some(name) => "Hello, ".to_string() + &name + "!",
    //         None => "Hello, World!",
    //     }
    //     res.send(response_body.as_bytes()).unwrap();
    // });
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
