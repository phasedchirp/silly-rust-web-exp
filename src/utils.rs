use rand::{self, Rng};
use nickel::Params;
use std::fs::File;
use std::io::{Read,Write};



#[derive(RustcEncodable,Clone,Debug)]
pub struct Question {
    number: usize,
    text: String,
    options: Option<Vec<String>>
}

impl Question {
    fn to_html_string(&self) -> String {
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

    fn as_string(&self) -> String {
        match self.options {
            None => format!("{}",self.text),
            Some(ref opts) => {
                format!("{}:{}",self.text,opts.join(","))
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
    pub key: String,
    pub questions: Vec<Question>
}

impl Survey {
    pub fn new(qs: &Vec<&str>) -> Survey {
        let id = new_id(10);
        let key = new_id(10);
        Survey{id: id,key:key,questions:make_questions(qs)}
    }

    pub fn to_file(&self,path: &str) {
        let fr = File::create(path);
        match fr {
            Ok(mut f) => {
                let qs: String = self.questions.iter()
                        .map(|q| q.as_string()).collect::<Vec<String>>().join("\n");
                f.write_all(&qs.as_bytes()).unwrap();
            },
            Err(e) => {println!("{:?}",e);}
        }
    }

    pub fn from_file(survey_file: &str) -> Result<Survey,u32> {
        match File::open(survey_file) {
            Ok(mut f) => {
                let id_key: Vec<&str> = survey_file.split('-').collect();
                let mut buf = String::new();
                f.read_to_string(&mut buf).unwrap();
                let qs = make_questions(&buf.trim().split("\r\n").collect());
                Ok(Survey {id: id_key[0].to_string(), key: id_key[1].to_string(), questions: qs})
            },
            Err(_) => Err(400)
        }
    }

    pub fn to_form(&self) -> String {
        let mut result = String::new();
        for q in &self.questions {
                let current_q = q.to_html_string();
                result.push_str(&current_q);
            }
        result
    }

    pub fn to_stmnt(&self) -> String {
        let mut stmnt = format!("CREATE TABLE \"{}\" (id string PRIMARY KEY,",self.id);
        for q in 0..(self.questions.len()) {
            stmnt.push_str(&format!("q{} TEXT,\n",q));
        }
        stmnt.push_str("time string\n)");
        stmnt
    }

    pub fn to_drop(&self) -> String {
        format!("DROP TABLE \"{}\"",self.id)
    }

    pub fn get_results(&self) -> String {
        format!("SELECT * FROM \"{}\"",self.id)
    }
}


// pub fn prep_insert_statement(s: &Survey) -> String {
//     let mut stmnt = format!("CREATE TABLE \"{}\" (id string PRIMARY KEY,",s.id);
//     for q in 0..(s.questions.len()) {
//         stmnt.push_str(&format!("q{} TEXT,\n",q));
//     }
//     stmnt.push_str("time string\n)");
//     stmnt
// }


#[derive(Debug)]
pub struct SResponse {
    id: String,
    s_id: String,
    vals: Vec<(usize,String,String)>
}

impl SResponse {
    pub fn new(p: &Params, s: &Survey, id: &str) -> SResponse {
        let mut result = Vec::new();
        for i in s.questions.iter() {
            let text = i.text.clone();
            let par = format!("q{}",&i.number);
            match p.get(&par){
                Some(val) => result.push((i.number,text,val.to_string())),
                None      => result.push((i.number,text,"no response".to_string()))
            };
        }
        SResponse {id: new_id(10), s_id: id.to_string(), vals: result}
    }

    pub fn to_stmnt(&self,t: &str) -> String {
        let mut stmnt = format!("INSERT INTO \"{}\" (id, ",self.s_id);
        let mut vals = format!(" VALUES (\"{}\" ,",self.id);
        for r in self.vals.iter() {
            stmnt.push_str(&format!("q{}, ", r.0));
            vals.push_str(&format!("\"{}\", ",r.2));
        }
        stmnt.push_str("time)");
        vals.push_str(&format!("\"{}\")",t));
        stmnt.push_str(&vals);
        println!("{}",stmnt);
        stmnt
    }

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
