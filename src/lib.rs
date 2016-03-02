#![allow(dead_code)]
extern crate rand;
extern crate time;

use rand::{thread_rng, Rng};
use time::{precise_time_ns};

#[derive(Debug)]
pub struct CohortResult<T: Clone> {
    pub duration: f64,
    pub result: T
}

impl<T: Clone> CohortResult<T> {
    pub fn new(duration: f64, result: &T) -> Self {
        CohortResult {
            duration: duration,
            result: result.clone()
        }
    }
}

#[derive(Debug)]
pub struct ExperimentResult<CurrentT: Clone, NewT: Clone> {
    pub current: CohortResult<CurrentT>,
    pub new: CohortResult<NewT>,
    pub name: &'static str
}

impl<CurrentT: Clone, NewT: Clone> ExperimentResult<CurrentT, NewT> {
    pub fn new(current: CohortResult<CurrentT>, new: CohortResult<NewT>, name: &'static str) -> Self {
        ExperimentResult {
            current: current,
            new: new,
            name: name
        }
    }
}

pub trait Experiment<CurrentResult: Clone, NewResult: Clone> {
    fn publish(_: ExperimentResult<CurrentResult, NewResult>) {}

    fn carry_out<C, N, Param>(name: &'static str, mut current: C, mut new: N, param: Param) -> CurrentResult
        where C: FnMut(Param) -> CurrentResult,
              N: FnMut(Param) -> NewResult,
              Param: Clone {
        let mut rng = thread_rng();
        let mut did_one = false;
        let current_goes_first: bool = rng.gen();
        let mut current_val = None;
        let mut new_val = None;
        let mut current_duration = 0;
        let mut new_duration = 0;
        loop {
            if (current_goes_first || did_one) && !(current_goes_first && did_one) {
                let start = precise_time_ns();
                current_val = Some(current(param.clone()));
                current_duration = precise_time_ns() - start;
                if did_one {
                    break;
                } else {
                    did_one = true;
                }
            }
            if (!current_goes_first || did_one) && !(!current_goes_first && did_one) {
                let start = precise_time_ns();
                new_val = Some(new(param.clone()));
                new_duration = precise_time_ns() - start;
                if did_one {
                    break;
                } else {
                    did_one = true;
                }
            }
        }
        Self::publish(ExperimentResult::new(
            CohortResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
            CohortResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
            name
        ));
        current_val.unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::{Experiment, ExperimentResult};
    struct TestExperiment;

    impl Experiment<String, String> for TestExperiment {
        fn publish(result: ExperimentResult<String, String>) {
            println!("{:?}", result);
        }
    }

    #[test]
    fn it_works() {
        let a_str = "bagelman";
        let a = TestExperiment::carry_out(
            "experiment!",
            |_| {
                println!("current went!");
                a_str.to_string()
            },
            |_| {
                println!("new went!");
                a_str.to_string()
            },
            ()
        );
        println!("{}", a);
    }
}

#[doc(hidden)]
pub mod internal {
    pub use rand;
    pub use time;
}
