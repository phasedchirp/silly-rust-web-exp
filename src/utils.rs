use rand::{self, Rng};
use nickel::Params;


// #[derive(Debug)]
// pub struct SResponse {
//     id: String,
//     vals: Vec<(String,String)>
// }

#[derive(RustcEncodable,Clone,Debug)]
pub struct Question {
    number: usize,
    text: String,
    options: Option<Vec<String>>
}

impl Question {
    pub fn to_html_string(&self) -> String {
        match self.options {
            None => format!("{t}<br><input type=\"text\" name=\"q{n}\"></br>",t = self.text, n = self.number),
            Some(ref opts) => {
                let mut result = format!("{t}<br>",t=self.text);
                for opt in opts {
                    result.push_str(&format!("<input type=\"radio\" name=\"q{n}\" value=\"{o}\">{o}<br>",n=self.number, o=opt));
                }
                result
            }
        }
    }

    pub fn new(i: usize, t: &str) -> Question {
        let q_opts: Vec<&str> = t.trim().split(':').collect();
        Question {
            number: i,
            text: q_opts[0].to_string(),
            options: match q_opts.len() > 1 {
                true => Some(q_opts[1].split(',').map(|s| s.to_string()).collect()),
                false => None
            }
        }
    }
}

pub fn make_questions(qs: &Vec<&str>) -> Vec<Question> {
    let mut result = Vec::new();
    for (i,q) in qs.iter().enumerate() {
        result.push(Question::new(i,q));
    }
    result
}

#[derive(RustcEncodable,Clone,Debug)]
pub struct Survey {
    pub id: String,
    pub questions: Vec<Question>
}

impl Survey {
    pub fn to_form(&self) -> String {
        let mut result = String::new();
        for q in &self.questions {
                let current_q = q.to_html_string();
                result.push_str(&current_q);
            }
        result
    }
}

pub fn parse_response(p: &Params, s: &Survey) -> Vec<(usize,String,String)> {
    let mut result = Vec::new();
    for i in s.questions.iter() {
        let text = i.text.clone();
        let par = format!("q{}",&i.number);
        match p.get(&par){
            Some(val) => result.push((i.number,text,val.to_string())),
            None      => result.push((i.number,text,"no response".to_string()))
        };
    }
    result
}


pub fn prep_resp_statement(resp: &Vec<(usize,String,String)>, s_id: &str, id: &str, t: &str) -> String {
    let mut stmnt = format!("INSERT INTO \"{}\" (id, ",s_id);
    let mut vals = format!(" VALUES (\"{}\" ,",id);
    for r in resp {
        stmnt.push_str(&format!("q{}, ", r.0));
        vals.push_str(&format!("\"{}\", ",r.2));
    }
    stmnt.push_str("time)");
    vals.push_str(&format!("\"{}\")",t));
    stmnt.push_str(&vals);
    // println!("{}",stmnt);
    stmnt
}

pub fn prep_insert_statement(s: &Survey) -> String {
    let mut stmnt = format!("CREATE TABLE \"{}\" (id string PRIMARY KEY,",s.id);
    for q in 0..(s.questions.len()) {
        stmnt.push_str(&format!("q{} TEXT,\n",q));
    }
    stmnt.push_str("time string\n)");
    stmnt
}

/// Table to retrieve base62 values from.
const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn new_id(size: usize) -> String {
    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();
    for _ in 0..size {
        id.push(BASE62[rng.gen::<usize>() % 62] as char);
    }
    id
}
