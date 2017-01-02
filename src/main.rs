#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;

use std::collections::{HashMap,HashSet};

use nickel::{Nickel, HttpRouter, MediaType, FormBody};
use nickel::status::StatusCode;



use std::fs::{File,remove_file,read_dir};
use std::io::{Read,Write};
use std::sync::{Arc,RwLock};

mod make_id;

use make_id::new_id;

#[derive(RustcEncodable,Clone,Debug)]
struct Question {
    number: usize,
    text: String,
    options: Option<Vec<String>>
}

#[derive(RustcEncodable,Clone,Debug)]
struct Survey {
    path: String,
    questions: Vec<Question>
}

fn make_questions(qs: &Vec<&str>) -> Vec<Question> {
    let mut result = Vec::new();
    for (i,q) in qs.iter().enumerate() {
        result.push(Question{number:i,text:q.to_string(),options:None})
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

    let mut surveys = HashMap::new();
    let paths = read_dir("surveys/").unwrap();
    for path in paths {
        if let Ok(p) = path {
            let id = p.file_name().into_string().unwrap();
            surveys.insert(id.clone(),survey_from_id(&id).unwrap());
            // println!("{:?}", p.file_name());
        }
    }

    let mut shared_info = Arc::new(RwLock::new(surveys));

    //middleware function logs each request to console
    // taken from https://github.com/Codenator81/nickel-demo
    server.utilize(middleware! { |request|
        println!("logging request: {:?}", request.origin.uri);
    });

    server.get("/survey/new", middleware! { |_, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        return resp.send_file("resources/makeSurvey.html");
    });

    let shared_clone = shared_info.clone();
    server.post("/survey/created", middleware!{ |req, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        let form_data = try_with!(resp,req.form_body());
        let survey_id = new_id(6);

        // println!("{:?}", shared_clone);
        let mut surveys = shared_clone.write().unwrap();
        // (*shared_info).insert(survey_id.clone());


        let file_name = format!("surveys/{}",&survey_id);
        let mut fr = File::create(file_name);
        match fr {
            Ok(mut f) => {
                let qs = form_data.get("questions").unwrap();
                surveys.insert(survey_id.clone(),
                        make_questions(&(qs.split("\r\n").collect())));

                f.write_all(&qs.as_bytes());
                let mut data = HashMap::new();
                data.insert("path",format!("survey/{}",survey_id));
                return resp.render("resources/path.tpl", &data);
            },
            Err(e) => {println!("{:?}",e);}
        }
    });

    server.get("/survey/:foo", middleware!{ |req, mut resp|
        let survey_id = req.param("foo").unwrap();
        let surveys = shared_info.read().unwrap();
        match surveys.get(survey_id) {
            Some(qs) => {
                resp.set(StatusCode::Ok);
                resp.set(MediaType::Html);
                let mut data = HashMap::new();
                data.insert("questions",qs);
                return resp.render("resources/takeSurvey.tpl",&data);
            },
            None => {
                resp.set(StatusCode::NotFound);
                "That survey ID doesn't seem to exist"
            }
        }
        // match survey_from_id(survey_id) {
        //     Ok(qs) => {
        //         resp.set(StatusCode::Ok);
        //         resp.set(MediaType::Html);
        //         let mut data = HashMap::new();
        //         data.insert("questions",qs);
        //         return resp.render("resources/takeSurvey.tpl",&data);
        //     },
        //     Err(e) => {
        //         resp.set(StatusCode::NotFound);
        //         println!("{:?}", e);
        //         "That survey ID doesn't seem to exist"
        //     }
        // }
    });



    server.listen("127.0.0.1:6767");
}
