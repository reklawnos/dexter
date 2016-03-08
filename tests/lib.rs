extern crate dexter;
use dexter::{Experiment, ExperimentResult, Publisher, MatchType};

struct TestPublisher {
    pub result: Option<ExperimentResult<String, String>>
}

impl Publisher<Vec<char>, String, String> for TestPublisher {
    fn publish(&mut self, result: ExperimentResult<String, String>) {
        self.result = Some(result);
    }

    fn compare(&mut self, current_result: &String, new_result: &String) -> bool {
        current_result == new_result
    }
}

#[test]
fn test_basic() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .carry_out(chars, &mut p);
    assert!(p.result.is_some());
    let experiment_result = p.result.unwrap();
    assert!(experiment_result.name == "experiment");
    assert!(experiment_result.match_type == MatchType::Match);
    assert!(result == experiment_result.current.result);
    assert!(experiment_result.current.result == experiment_result.new.result);
    assert!(experiment_result.current.duration > 0f64);
    assert!(experiment_result.new.duration > 0f64);
}

#[test]
fn test_no_match() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            let mut s = a.clone();
            s.reverse();
            s.into_iter().collect()
        })
        .carry_out(chars, &mut p);
    assert!(p.result.is_some());
    let experiment_result = p.result.unwrap();
    assert!(experiment_result.match_type == MatchType::NoMatch);
    assert!(result == experiment_result.current.result);
    assert!(experiment_result.current.result != experiment_result.new.result);
}


#[test]
fn test_setup() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .setup(|mut p| { p.push('d'); p })
        .run_if(|p| { p.len() == 4 })
        .carry_out(chars.clone(), &mut p);

    assert!(p.result.is_some());
    assert!(result == "abcd".to_string());
}

#[test]
fn test_run_if() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .run_if(|p| { p.len() == 3 })
        .carry_out(chars.clone(), &mut p);

    assert!(p.result.is_some());
    assert!(result == "abc".to_string());

    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .run_if(|p| { p.len() != 3 })
        .carry_out(chars.clone(), &mut p);

    assert!(p.result.is_none());
    assert!(result == "abc".to_string());
}

#[test]
fn test_ignore() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = TestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .ignore_if(|p| { p.len() == 3 })
        .carry_out(chars, &mut p);
    assert!(p.result.is_some());
    let experiment_result = p.result.unwrap();
    assert!(experiment_result.match_type == MatchType::Ignored);
    assert!(result == experiment_result.current.result);
}

struct DisabledTestPublisher {
    pub result: Option<ExperimentResult<String, String>>
}

impl Publisher<Vec<char>, String, String> for DisabledTestPublisher {
    fn publish(&mut self, result: ExperimentResult<String, String>) {
        self.result = Some(result);
    }

    fn enabled(&mut self) -> bool {
        false
    }

    fn compare(&mut self, current_result: &String, new_result: &String) -> bool {
        current_result == new_result
    }
}

#[test]
fn test_disabled() {
    let chars = vec!['a', 'b', 'c'];
    let mut p = DisabledTestPublisher{ result: None };
    let result = Experiment::new("experiment",
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        },
        |a: &Vec<char>| {
            a.clone().into_iter().collect()
        })
        .carry_out(chars.clone(), &mut p);

    assert!(p.result.is_none());
    assert!(result == "abc".to_string());
}
