#[macro_use] extern crate nickel;

use std::collections::HashMap;
use nickel::{Nickel, MediaType};
use nickel::status::StatusCode;

extern crate hyper;
use std::io::Read;
// use hyper::Client;
// use hyper::header::Connection;

// Inefficient prime testing
fn is_prime(x: u64) -> &'static str {
    let mut result = false;
    let upper = 1 + (x as f64).sqrt().ceil() as u64;
    for i in 2..upper {
        if x % i == 0 {
            result = true;
            break;
        }
    }
    match result {
        false => "prime",
        true => "not prime"
    }
}

#[test]
fn composite_test() {
    assert!(is_prime(9) == "not prime");
}

#[test]
fn prime_test() {
    assert!(is_prime(7) == "prime");
}

fn main() {
    let mut server = Nickel::new();

    //middleware function logs each request to console
    // taken from https://github.com/Codenator81/nickel-demo
    server.utilize(middleware! { |request|
        println!("logging request: {:?}", request.origin.uri);
    });

    let router = router! {
        // get "/" => |_, resp| {"this is a test"}
        get "/" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let mut data = HashMap::new();
            data.insert("placeholder","blah");
            return resp.render("resources/default.tpl",&data);
        }

        get "/login" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            return resp.send_file("resources/login.tpl");
        }
        // https://github.com/nickel-org/nickel.rs/issues/240
        post "/loggedin" =>  |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let mut form_data = String::new();
            req.origin.read_to_string(&mut form_data);
            println!("{:?}", form_data);
            let mut data = HashMap::new();
            data.insert("error", form_data);
            return resp.render("resources/loggedIn.tpl", &data);
        }

        get "/foo/:x" => |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let mut data = HashMap::new();
            let x_val = req.param("x").unwrap();

            data.insert("x", x_val);
            data.insert("result",is_prime(x_val.trim().parse().expect("parse error")));
            return resp.render("resources/primes.tpl", &data);
        }
    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
