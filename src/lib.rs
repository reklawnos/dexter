#![allow(dead_code)]
extern crate rand;
extern crate time;


#[derive(Debug)]
pub struct CohortResult<T: Clone> {
    duration: f64,
    result: T
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
    current: CohortResult<CurrentT>,
    new: CohortResult<NewT>,
    name: &'static str
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

#[macro_export]
macro_rules! experiment {
    ($dexter_type:ident, $name:expr, current:$current:block new:$new:block) => {
        {
            use rand::{thread_rng, Rng};
            use time::{precise_time_ns};

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
                    current_val = Some($current);
                    current_duration = precise_time_ns() - start;
                    if did_one {
                        break;
                    } else {
                        did_one = true;
                    }
                }
                if (!current_goes_first || did_one) && !(!current_goes_first && did_one) {
                    let start = precise_time_ns();
                    new_val = Some($new);
                    new_duration = precise_time_ns() - start;
                    if did_one {
                        break;
                    } else {
                        did_one = true;
                    }
                }
            }
            $dexter_type::publish(ExperimentResult::new(
                CohortResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
                CohortResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
                $name
            ));
            current_val.unwrap()
        }
    }
}

pub trait Dexter<CurrentT: Clone, NewT: Clone> {
    fn publish(_: ExperimentResult<CurrentT, NewT>) {}
}

#[cfg(test)]
mod test {
    use super::*;
    struct TestExperiment;

    impl Dexter<String, String> for TestExperiment {
        fn publish(result: ExperimentResult<String, String>) {
            println!("{:?}", result);
        }
    }

    #[test]
    fn it_works() {
        let a = experiment!{
            TestExperiment,
            "experiment!",
            current: {
                println!("current went!");
                "current result".to_string()
            }
            new: {
                println!("new went!");
                "new result".to_string()
            }
        };
        println!("{}", a);
    }
}
