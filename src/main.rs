#[macro_use]
extern crate nickel;

extern crate rand;
extern crate rusqlite;
extern crate chrono;


use nickel::{Nickel, HttpRouter, MediaType, FormBody};
use nickel::status::StatusCode;

use rusqlite::Connection;

use chrono::UTC;

use std::collections::HashMap;
use std::fs::{remove_file,read_dir};
use std::sync::{Arc,RwLock,Mutex};

mod simpoll;
use simpoll::*;

#[cfg(test)]
mod tests;



fn main() {
    let mut server = Nickel::new();
    let conn = Connection::open("surveys.sqlite").unwrap();
    conn.execute("DROP TABLE IF EXISTS example; CREATE TABLE example (id string PRIMARY KEY, q0 string, q1 string);",&[]).unwrap();
    let conn_arc = Arc::new(Mutex::new(conn));

    let mut surveys = HashMap::new();

    // reload surveys from file on re-start:
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
        s.to_stmnt(&conn);
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

        let user_response = SResponse::new(&form_data,                            surveys.get(&survey_id).unwrap(), &survey_id);

        user_response.to_stmnt(&conn,&UTC::now().to_string());
        // conn.execute(&user_response.to_stmnt(&UTC::now().to_string()),&[]).unwrap();

        "Thanks for taking that survey!"
    });

    // retrieving results
    server.get("/results", middleware! {|_, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        return resp.send_file("resources/getResults.html");
    });

    let surveys_clone_get = surveys_arc.clone();
    let conn_get = conn_arc.clone();
    server.post("/results",middleware!{ |req, mut resp|
        let form_data = try_with!(resp,req.form_body());
        let id = form_data.get("id").unwrap();
        let key = form_data.get("key").unwrap();
        // let resp_format = req.param("baz").unwrap();

        let surveys = surveys_clone_get.read().unwrap();
        let conn = conn_get.lock().unwrap();

        match surveys.get(id) {
            Some(s) => {
                if s.key == key {
                    let q_len = s.questions.len();
                    let mut stmnt = s.get_results(&conn);
                    let rows = stmnt.query_map(&[],|r| {
                        let mut row: Vec<String> = Vec::new();
                        for i in 0..(q_len+2) {
                            row.push(r.get(i as i32));
                        }
                        row
                    }).unwrap();

                    let mut csv_string = s.questions.iter()
                              .fold("\"id".to_string(),|a,q| a + "\",\"" + &q.text);
                    csv_string.push_str("\",timestamp\n");
                    for result in rows {
                        let line = format!("{}\n",result.unwrap().join(","));
                        println!("{:?}", &line);
                        csv_string += &line;
                    }
                    resp.set(MediaType::Txt);
                    return resp.send(csv_string);
                } else {
                    resp.set(StatusCode::Forbidden);
                    "You don't have permission to do that"
                }
            },
            None => {
                resp.set(StatusCode::Forbidden);
                "You don't have permission to do that"
            }
        }
    });

    // deleting surveys:
    server.get("/delete", middleware! {|_, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        return resp.send_file("resources/deleteSurvey.html");
    });
    // handling deletions
    let surveys_clone_delete = surveys_arc.clone();
    let conn_delete = conn_arc.clone();
    server.post("/deleted", middleware! { |req, mut resp|
        let form_data = try_with!(resp,req.form_body());
        let id = form_data.get("id").unwrap();
        let key = form_data.get("key").unwrap();
        let mut surveys = surveys_clone_delete.write().unwrap();
        let conn = conn_delete.lock().unwrap();
        let result = match surveys.get(id) {
            Some(s) => {
                if s.key == key {
                    resp.set(StatusCode::Ok);
                    s.to_drop(&conn);
                    remove_file(&format!("surveys/{}-{}",s.id,s.key)).unwrap();
                    "Survey deleted"
                } else {
                    resp.set(StatusCode::Forbidden);
                    "You don't have permission to do that"
                }
            },
            None => {
                resp.set(StatusCode::Forbidden);
                "You don't have permission to do that"
            }
        };
        if result == "Survey deleted" {
            surveys.remove(id);
        }
        result
    });

    server.listen("127.0.0.1:6767").unwrap();
}
