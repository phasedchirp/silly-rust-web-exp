#[macro_use] extern crate nickel;
extern crate rand;

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

// struct Question {
//     text: String
// }
fn make_questions(qs: Vec<&str>) -> String {
    let mut result = String::new();
    // let temp = format!("{q}<input type=\"text\" name=\"q{i}\"><br>",
                        // q="hello",i=1);
    // result = result + &temp;
    for (i,q) in qs.iter().enumerate() {
        result = result + &(format!("{q}<br><input type=\"text\" name=\"q{i}\"><br>",
        q=q,i=i));
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

        get "/login" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            return resp.send_file("resources/login.html");
        }
        // https://github.com/nickel-org/nickel.rs/issues/240
        post "/loggedin" =>  |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let form_data = try_with!(resp, req.form_body());
            println!("{:?}", form_data);
            let mut data = HashMap::new();
            data.insert("key1","email: ");
            data.insert("key2","password: ");
            data.insert("val1",form_data.get("email").unwrap());
            data.insert("val2",form_data.get("password").unwrap());
            return resp.render("resources/loggedIn.tpl", &data);
        }

    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
