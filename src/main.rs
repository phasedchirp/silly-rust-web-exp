#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;

use std::collections::HashMap;
use nickel::{Nickel, MediaType, FormBody};
use nickel::status::StatusCode;

// use std::io;
// use std::path::Path;
// use std::fs::File;

extern crate hyper;
use std::io::Read;
// use hyper::Client;
// use hyper::header::Connection;

mod make_id;

use make_id::MakeID;

#[derive(RustcEncodable)]
struct Question {
    number: usize,
    text: String
}

fn make_questions(qs: Vec<&str>) -> Vec<Question> {
    let mut result = Vec::new();
    for (i,q) in qs.iter().enumerate() {
        result.push(Question{number:i,text:q.to_string()})
        //  = result + &(format!("{q}<br><input type=\"text\" name=\"q{i}\"></br>", q=q,i=i));
    }
    result
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

        get "/survey/new" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            return resp.send_file("resources/makeSurvey.html");
        }

        post "/makeSurvey" => |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let form_data = try_with!(resp,req.form_body());
            let qs: Vec<&str> = form_data.get("questions").unwrap()
                     .split("\r\n").collect();

            let questions = make_questions(qs);

            let mut data = HashMap::new();
            data.insert("questions",questions);
            return resp.render("resources/surveyCreated.tpl", &data);
        }

    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
