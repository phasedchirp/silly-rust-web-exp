use simpoll::*;
use nickel::Params;

// properly parsing questions
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
