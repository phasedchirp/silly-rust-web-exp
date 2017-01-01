#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;

use std::collections::HashMap;

use nickel::{Nickel, MediaType, FormBody};
use nickel::status::StatusCode;



use std::fs::{File,remove_file};
use std::io::{Read,Write};


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

        get "/survey/new" => |_, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            return resp.send_file("resources/makeSurvey.html");
        }

        post "/survey/created" => |req, mut resp| {
            resp.set(StatusCode::Ok);
            resp.set(MediaType::Html);
            let form_data = try_with!(resp,req.form_body());
            let survey_id = new_id(6);


            let file_name = format!("surveys/{}",&survey_id);
            let mut fr = File::create(file_name);
            match fr {
                Ok(mut f) => {
                    f.write_all(form_data.get("questions").unwrap().as_bytes());
                    let mut data = HashMap::new();
                    data.insert("path",format!("survey/{}",survey_id));
                    return resp.render("resources/path.tpl", &data);
                },
                Err(e) => {println!("{:?}",e);}
            }

        }


        get "/survey/:foo" => |req, mut resp| {
            let survey_id = req.param("foo").unwrap();
            match survey_from_id(survey_id) {
                Ok(qs) => {
                    resp.set(StatusCode::Ok);
                    resp.set(MediaType::Html);
                    let mut data = HashMap::new();
                    data.insert("questions",qs);
                    return resp.render("resources/takeSurvey.tpl",&data);
                },
                Err(e) => {
                    resp.set(StatusCode::NotFound);
                    println!("{:?}", e);
                    "That survey ID doesn't seem to exist"
                }
            }

        }

    };
    server.utilize(router);

    server.listen("127.0.0.1:6767");
}
