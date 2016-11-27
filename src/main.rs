#[macro_use] extern crate nickel;

use std::collections::HashMap;
use nickel::Nickel;

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
        get "/blah" => |_, response| {"this is boring"}

        get "/user/:userId" => |req, resp| {
            let mut data = HashMap::new();
            data.insert("name",req.param("userId").unwrap());
            return resp.render("template.tpl", &data);
        }

        // get "/foo/*" => |_, response| {"you've reached a foo handler"}

        get "/foo/:x" => |req, resp| {
            let mut data = HashMap::new();
            let x_val = req.param("x").unwrap();

            data.insert("x", x_val);
            data.insert("result",test(x_val.trim().parse().expect("parse error")));
            return resp.render("template-2.tpl", &data);
        }
    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
