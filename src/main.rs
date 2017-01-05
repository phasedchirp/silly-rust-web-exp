#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;
extern crate rusqlite;
// extern crate chrono;


use nickel::{Nickel, HttpRouter, MediaType, FormBody};
use nickel::status::StatusCode;

use rusqlite::Connection;

use std::collections::HashMap;
use std::fs::{File,remove_file,read_dir};
use std::io::{Read,Write};
use std::sync::{Arc,RwLock,Mutex};

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

#[derive(Debug)]
struct SResponse {
    id: String,
    vals: Vec<(String,String)>
}

fn make_questions(qs: &Vec<&str>) -> Vec<Question> {
    let mut result = Vec::new();
    for (i,q) in qs.iter().enumerate() {
        let mut q_opts = q.trim().split(':').collect::<Vec<&str>>();
        let opts : Option<Vec<String>> = match q_opts.len() > 1 {
            true => Some(q_opts[1].split(',').map(|s| s.to_string()).collect()),
            false => None
        };
        result.push(Question{number:i,text:q_opts[0].to_string(),options:opts});
    }
    result
}

fn survey_from_file(survey_file: &str) -> Result<Vec<Question>,u32> {
    // let survey_file = format!("surveys/{}",id);
    match File::open(survey_file) {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf);
            let qs: Vec<&str> = buf.trim().split("\r\n").collect();
            Ok(make_questions(&qs))
        },
        Err(_) => Err(400)
    }
}

fn parse_survey(s: Vec<Question>) -> String {
    let mut result = String::new();
    for q in s {
        let current_q = match q.options {
            None => format!("{t}<br><input type=\"text\" name=\"q{n}\"></br>",t = q.text, n = q.number),
            Some(opts) => {
                let mut temp = format!("{t}<br>",t=q.text);
                for opt in opts {
                    temp.push_str(&format!("<input type=\"radio\" name=\"q{n}\" value=\"{o}\">{o}<br>",n=q.number, o=opt));
                }
                temp
            }
        };
        result.push_str(&current_q);
    }
    result
}

fn prep_resp_statement(resp: &SResponse, id: &str, t: &str) -> String {
    let mut stmnt = "INSERT INTO responses (id,".to_string();
    let mut vals = format!("VALUES ({}",id);
    for r in &resp.vals {
        stmnt.push_str(&format!("q{},", r.0));
        vals.push_str(&format!("{},",r.1));
    }
    stmnt.push_str("time)");
    stmnt.push_str(&format!("{})",t));
    stmnt.push_str(&vals);
    stmnt
}

fn prep_insert_statement(s: &Survey) -> String {
    let mut stmnt = "CREATE TABLE responses (id string PRIMARY KEY,".to_string();
    for q in 0..(s.questions.len()) {
        stmnt.push_str(&format!("q{} TEXT,\n",q));
    }
    stmnt.push_str("time string\n)");
    stmnt
}

fn main() {
    let mut server = Nickel::new();

    // currently creating one large db for everything
    // could also create db for each survey instead
    // relevant docs:
    // http://jgallagher.github.io/rusqlite/rusqlite/struct.Connection.html
    let mut conn_arc = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));

    let mut surveys = HashMap::new();
    let paths = read_dir("surveys/").unwrap();
    for path in paths {
        if let Ok(p) = path {
            let id = p.file_name().into_string().unwrap();
            let survey_file = format!("surveys/{}",&id);
            surveys.insert(id.clone(),survey_from_file(&survey_file).unwrap());
        }
    }

    // See following example for approach to sharing data between handlers:
    // https://github.com/nickel-org/nickel.rs/blob/master/examples/route_data.rs
    let mut surveys_arc = Arc::new(RwLock::new(surveys));

    //middleware function logs each request to console
    // taken from https://github.com/Codenator81/nickel-demo
    server.utilize(middleware! { |request|
        println!("logging request: {:?}", request.origin.uri);
    });

    // route for survey creation page:
    server.get("/survey/new", middleware! { |_, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        return resp.send_file("resources/makeSurvey.html");
    });

    // route (plus setup) for adding user-created surveys:
    let surveys_clone_make = surveys_arc.clone();
    let conn_clone = conn_arc.clone();
    server.post("/survey/created", middleware!{ |req, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        let form_data = try_with!(resp,req.form_body());
        let survey_id = new_id(6);

        let mut surveys = surveys_clone_make.write().unwrap();
        let conn = conn_clone.lock().unwrap();


        let file_name = format!("surveys/{}",&survey_id);
        let mut fr = File::create(file_name);
        match fr {
            Ok(mut f) => {
                let qs = form_data.get("questions").unwrap();
                surveys.insert(survey_id.clone(),
                        make_questions(&(qs.split("\r\n").collect())));

                // conn.execute(&insert_survey(surveys.get(&survey_id).unwrap())).unwrap();
                f.write_all(&qs.as_bytes());
                let mut data = HashMap::new();
                data.insert("path",format!("survey/{}",survey_id));
                return resp.render("resources/path.tpl", &data);
            },
            Err(e) => {println!("{:?}",e);}
        }
    });

    // route for taking a survey
    let surveys_clone_take = surveys_arc.clone();
    server.get("/survey/:foo", middleware!{ |req, mut resp|
        let survey_id = req.param("foo").unwrap();
        let surveys = surveys_clone_take.read().unwrap();

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
    });

    // route for submitting completed survey
    server.post("survey/:foo/submit", middleware!{ |req, mut resp|
        let conn = conn_arc.lock().unwrap();
        let survey_id = req.param("foo").unwrap().to_owned();
        let form_data = try_with!(resp,req.form_body());
        let surveys = surveys_arc.read().unwrap();
        let user_id = new_id(10);

    });



    server.listen("127.0.0.1:6767");
}
