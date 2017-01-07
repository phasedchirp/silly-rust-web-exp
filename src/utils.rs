use rand::{self, Rng};


#[derive(RustcEncodable,Clone,Debug)]
pub struct Question {
    number: usize,
    text: String,
    options: Option<Vec<String>>
}

#[derive(RustcEncodable,Clone,Debug)]
pub struct Survey {
    pub id: String,
    pub questions: Vec<Question>
}

#[derive(Debug)]
pub struct SResponse {
    id: String,
    vals: Vec<(String,String)>
}

pub fn make_questions(qs: &Vec<&str>) -> Vec<Question> {
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

pub fn parse_survey(s: &Vec<Question>) -> String {
    let mut result = String::new();
    for q in s {
        let current_q = match q.options {
            None => format!("{t}<br><input type=\"text\" name=\"q{n}\"></br>",t = q.text, n = q.number),
            Some(ref opts) => {
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


pub fn prep_resp_statement(resp: &SResponse, id: &str, t: &str) -> String {
    let mut stmnt = "INSERT INTO responses (id,".to_string();
    let mut vals = format!("VALUES ({}",id);
    for r in &resp.vals {
        stmnt.push_str(&format!("q{},", r.0));
        vals.push_str(&format!("\"{}\",",r.1));
    }
    stmnt.push_str("time)");
    stmnt.push_str(&format!("\"{}\")",t));
    stmnt.push_str(&vals);
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
