#[macro_use] extern crate nickel;

use std::collections::HashMap;
use nickel::{Nickel, MediaType};
use nickel::status::StatusCode;

// Inefficient prime testing
fn test(x: u64) -> &'static str {
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

        post "/login" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let mut data = HashMap::new();
            data.insert("error", "hello");
            return resp.render("resources/login.tpl", &data);
        }

        get "/foo/:x" => |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let mut data = HashMap::new();
            let x_val = req.param("x").unwrap();

            data.insert("x", x_val);
            data.insert("result",test(x_val.trim().parse().expect("parse error")));
            return resp.render("resources/primes.tpl", &data);
        }
    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
