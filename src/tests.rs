use simpoll::*;
use nickel::Params;

// parsing question without options:
#[test]
fn question_no_opts() {
    let test_string = "What is your name".to_string();
    let test_q = Question::new(0,&test_string);
    let ref_q = Question{number: 0,
                         text: "What is your name".to_string(),
                         options: None};
    assert_eq!(test_q,ref_q);
}

#[test]
fn question_with_opts() {
    let test_string = "Does correlation imply causation?:yes,no".to_string();
    let test_q = Question::new(0,&test_string);
    let ref_q = Question{number:0,
                         text: "Does correlation imply causation?".to_string(),
                         options: Some(vec!["yes".to_string(),"no".to_string()])};
    assert_eq!(test_q,ref_q);
}


// properly parsing vec of strings into a survey
#[test]
fn survey_qs_vec() {
    let qs_test = vec!["Is this real life?",
                        "Is this just fantasy?",
                        "Will you do the Fandango?:yes,no"];
    let test_survey = Survey::new(&qs_test);

    let qs_ref = vec![Question{ number: 0,
                               text:"Is this real life?".to_string(),
                               options:None},
                      Question{ number:1,
                               text:"Is this just fantasy?".to_string(),
                               options:None},
                      Question{ number:2,
                          text:"Will you do the Fandango?".to_string(),
                          options:Some(vec!["yes".to_string(),"no".to_string()])}];

    // Only checking question parsing -- id is non-deterministic
    assert_eq!(test_survey.questions,qs_ref);
}



// generating html strings:
#[test]
fn qs_html() {
    let qs_test = vec!["Is this real life?",
                        "Is this just fantasy?",
                        "Will you do the Fandango?:yes,no"];
    let test_survey = Survey::new(&qs_test);

    let ref_string = "Is this real life?<br><input type=\"text\" name=\"q0\"></br>Is this just fantasy?<br><input type=\"text\" name=\"q1\"></br>Will you do the Fandango?<br><input type=\"radio\" name=\"q2\" value=\"yes\">yes<br><input type=\"radio\" name=\"q2\" value=\"no\">no<br>".to_string();

    assert_eq!(test_survey.to_form(),ref_string);
}

// #[test]
// fn result_parsing() {
//     let qs_test = vec!["Is this real life?",
//                         "Is this just fantasy?",
//                         "Will you do the Fandango?:yes,no"];
//     let test_survey = Survey::new(&qs_test);
//
//     input = Params{"q0":"something","q1":"something else");
//     resp_test = SResponse::new(&input,&test_survey,&test_survey.id);
//
// }
