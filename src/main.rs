#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;
extern crate rusqlite;
extern crate chrono;
extern crate tempfile;


use nickel::{Nickel, HttpRouter, MediaType, FormBody};
use nickel::status::StatusCode;

use rusqlite::Connection;

use chrono::{UTC,DateTime};

use tempfile::tempfile;

use std::collections::HashMap;
use std::fs::{File,remove_file,read_dir};
use std::io::{Read,Write};
use std::sync::{Arc,RwLock,Mutex};

mod utils;
use utils::*;

// #[cfg(test)]
// mod tests;


// fn survey_from_file(survey_file: &str) -> Result<Vec<Question>,u32> {
//     match File::open(survey_file) {
//         Ok(mut f) => {
//             let mut buf = String::new();
//             f.read_to_string(&mut buf).unwrap();
//             let qs: Vec<&str> = buf.trim().split("\r\n").collect();
//             Ok(make_questions(&qs))
//         },
//         Err(_) => Err(400)
//     }
// }



fn main() {
    let mut server = Nickel::new();
    let conn_arc = Arc::new(Mutex::new(Connection::open("surveys.sqlite").unwrap()));

    let mut surveys = HashMap::new();
    let paths = read_dir("surveys/").unwrap();
    for path in paths {
        if let Ok(p) = path {
            let id = p.file_name().into_string().unwrap();
            let s = Survey::from_file("surveys",&id).unwrap();
            surveys.insert(s.id.clone(),s);
        }
    }

    let surveys_arc = Arc::new(RwLock::new(surveys));

    // log requests to stdout:
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

        let qs = form_data.get("questions").unwrap();
        let s = Survey::new(&(qs.split("\r\n").collect()));

        let mut surveys = surveys_clone_make.write().unwrap();
        let conn = conn_clone.lock().unwrap();


        s.to_file(&format!("surveys/{}-{}",&s.id,&s.key));
        surveys.insert(s.id.clone(),s.clone());
        conn.execute(&s.to_stmnt(),&[]).unwrap();
        let mut data = HashMap::new();
        data.insert("id",s.id.clone());
        data.insert("key",s.key.clone());
        data.insert("path",format!("survey/{}",s.id));
        data.insert("results",format!("survey/{}/{}/<format>",s.id,s.key));
        data.insert("delete",format!("survey/{}/{}/delete",s.id,s.key));
        return resp.render("resources/path.tpl", &data);
    });

    // route for taking a survey
    let surveys_clone_take = surveys_arc.clone();
    server.get("/survey/:foo", middleware!{ |req, mut resp|
        let survey_id = req.param("foo").unwrap().to_string();
        let surveys = surveys_clone_take.read().unwrap();

        let mut data = HashMap::new();

        match surveys.get(&survey_id) {
            Some(qs) => {
                resp.set(StatusCode::Ok);
                resp.set(MediaType::Html);
                data.insert("id",survey_id);
                let qs_parsed = qs.to_form();
                data.insert("questions",qs_parsed);
                return resp.render("resources/takeSurvey.tpl",&data);
            },
            None => {
                resp.set(StatusCode::NotFound);
                "That survey ID doesn't seem to exist"
            }
        }
    });

    // route for submitting completed survey
    let conn_submit = conn_arc.clone();
    let surveys_clone_submit = surveys_arc.clone();
    server.post("/survey/:foo/submit", middleware!{ |req, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        let conn = conn_submit.lock().unwrap();
        let surveys = surveys_clone_submit.read().unwrap();

        let survey_id = req.param("foo").unwrap().to_owned();
        let form_data = try_with!(resp,req.form_body());

        let user_response = SResponse::new(&form_data,
                            surveys.get(&survey_id).unwrap(),
                            &survey_id);

        let timestamp : DateTime<UTC> = UTC::now();

        conn.execute(&user_response.to_stmnt(&timestamp.to_string()),&[]).unwrap();

        "Thanks for taking that survey!"
    });

    let surveys_clone_get = surveys_arc.clone();
    let conn_get = conn_arc.clone();
    server.get("/survey/:foo/:bar/:baz",middleware!{ |req, mut resp|
        let id = req.param("foo").unwrap();
        let key = req.param("bar").unwrap();
        let resp_format = req.param("baz").unwrap();

        let surveys = surveys_clone_get.read().unwrap();
        let conn = conn_get.lock().unwrap();

        match surveys.get(id) {
            Some(s) => {
                if s.key == key {
                    let q_len = s.questions.len();
                    let mut stmnt = conn.prepare(&s.get_results()).unwrap();
                    let mut rows = stmnt.query_map(&[],|r| {
                        let mut row: Vec<String> = Vec::new();
                        for i in 0..(q_len+2) {
                            row.push(r.get(i as i32));
                        }
                        row
                    }).unwrap();
                    // let temp_file = format!("temp/{}",s.id);
                    // let mut f: File = tempfile().expect("failed to create tempfile");
                    //File::create(&temp_file).unwrap();
                    let mut csv_string = s.questions.iter()
                              .fold("\"id".to_string(),|a,q| a + "\",\"" + &q.text);
                    csv_string.push_str("\",timestamp\n");
                    for result in rows {
                        let line = format!("{}\n",result.unwrap().join(","));
                        println!("{:?}", &line);
                        csv_string += &line;
                        // println!("{:?}", result);
                    resp.set(MediaType::Txt);
                    println!("{:?}", &csv_string);
                    return resp.send(csv_string);
                    }
                } else {
                    ()// return resp.send_file("resources/notPermitted.html");
                }
            },
            None => ()//return resp.send_file("resources/notPermitted.html");
        }

        // resp.set(StatusCode::Ok);
        // resp.set(MediaType::)
    });

    let surveys_clone_delete = surveys_arc.clone();
    let conn_delete = conn_arc.clone();
    server.get("/survey/:foo/:bar/delete", middleware! { |req, mut resp|
        let id = req.param("foo").unwrap();
        let key = req.param("bar").unwrap();
        let mut surveys = surveys_clone_delete.write().unwrap();
        let conn = conn_delete.lock().unwrap();
        let result = match surveys.get(id) {
            Some(s) => {
                if s.key == key {
                    resp.set(StatusCode::Ok);
                    conn.execute(&s.to_drop(),&[]).unwrap();
                    remove_file(&format!("surveys/{}-{}",s.id,s.key));
                    "Survey deleted"
                } else {
                    resp.set(StatusCode::Forbidden);
                    "Not allowed"
                }
            },
            None => {
                resp.set(StatusCode::Forbidden);
                "Not allowed"
            }
        };
        if result == "Survey deleted" {
            surveys.remove(id);
        }
        result
    });



    server.listen("127.0.0.1:6767").unwrap();
}
