#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;

use std::collections::{HashMap,HashSet};
// use std::sync::Mutex;

use nickel::{Nickel, MediaType, FormBody};
use nickel::status::StatusCode;

// use std::io;
// use std::path::Path;
use std::fs::File;

extern crate hyper;
use std::io::{Read,Write};
// use hyper::Client;
// use hyper::header::Connection;

mod make_id;

use make_id::new_id;

#[derive(RustcEncodable,Clone)]
struct Question {
    number: usize,
    text: String
}

fn make_questions(qs: &Vec<&str>) -> Vec<Question> {
    let mut result = Vec::new();
    for (i,q) in qs.iter().enumerate() {
        result.push(Question{number:i,text:q.to_string()})
    }
    result
}

fn survey_from_id(id: &str) -> Result<Vec<Question>,u32> {
    let survey_file = format!("surveys/{}",id);
    match File::open(survey_file) {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf);
            let qs: Vec<&str> = buf.split("\r\n").collect();
            Ok(make_questions(&qs))
        },
        Err(e) => Err(400)
    }
}

fn main() {
    let mut server = Nickel::new();
    // let mut surveys = Mutex::new(HashSet::new());

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
            return resp.send_file("resources/frontPage.html");
        }

        get "/survey/new" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            return resp.send_file("resources/makeSurvey.html");
        }

        post "/survey/check" => |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let form_data = try_with!(resp,req.form_body());
            let qs: Vec<&str> = (&form_data).get("questions").unwrap()
                     .split("\r\n").collect();

            let questions = make_questions(&qs);

            let survey_id = new_id(6);
            // let mut surveys = surveys.lock().unwrap();
            // (*surveys).insert(survey_id.clone());

            let file_name = format!("surveys/{}",survey_id.clone());
            let mut fr = File::create(file_name);
            match fr {
                Ok(mut f) => {
                    f.write_all(form_data.get("questions").unwrap().as_bytes());
                    let mut data = HashMap::new();
                    data.insert("questions",questions);
                    return resp.render("resources/surveyCreated.tpl", &data);
                },
                Err(e) => {println!("{:?}",e);}
            }

        }

        get "/survey/:foo" => |req, mut resp| {
            // let surveys = surveys.lock().unwrap();
            // let survey_file = format!("surveys/{}",req.param("foo").unwrap());
            let survey_id = req.param("foo").unwrap();
            match survey_from_id(survey_id) {
                Ok(_) => "yay!",
                Err(_) => "boo!"
            }

        }
            // // let mut survey = (&surveys).get(req.param("foo").unwrap());
            // match (&surveys).get(req.param("foo").unwrap()) {
            //     Some(questions) => {
            //         resp.set(StatusCode::Ok);
            //         resp.set(MediaType::Html);
            //         let mut data = HashMap::new();
            //         data.insert("questions",questions.clone());
            //         return resp.render("resources/survey.tpl", &data);
            //     },
            //     None => {
            //         resp.set(StatusCode::NotFound);
            //         // resp.set(MediaType::Html);
            //         // return resp.send_file
            //     }
            // }
            // let file_name = format!("surveys/{}",req.param("foo").unwrap());
            // let f = File::open(file_name);
            // match f {
            //
            // }
            // let mut text = String::new();
            // f.read_to_string(&buf);
            // match text {
            //     "" => ,
            //     _ =>
            // }

    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
