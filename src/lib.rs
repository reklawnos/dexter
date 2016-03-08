#![allow(dead_code)]
extern crate rand;
extern crate time;

use rand::{thread_rng, Rng};
use time::{precise_time_ns};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct CohortResult<T: Clone> {
    pub duration: f64,
    pub result: T
}

impl<T: Clone> CohortResult<T> {
    fn new(duration: f64, result: &T) -> Self {
        CohortResult {
            duration: duration,
            result: result.clone()
        }
    }
}

#[derive(Debug)]
pub struct ExperimentResult<Cr: Clone, Nr: Clone> {
    pub name: &'static str,
    pub current: CohortResult<Cr>,
    pub new: CohortResult<Nr>,
    pub match_type: MatchType
}

#[derive(Debug)]
pub enum MatchType {
    Match,
    NoMatch,
    Ignored
}

impl<Cr: Clone, Nr: Clone> ExperimentResult<Cr, Nr> {
    fn new(name: &'static str, current: CohortResult<Cr>, new: CohortResult<Nr>, match_type: MatchType) -> Self {
        ExperimentResult {
            name: name,
            current: current,
            new: new,
            match_type: match_type
        }
    }
}

pub struct ExperimentBuilder<'a, P, Cr: Clone, Nr: Clone, E: Experiment<P, Cr, Nr> + ?Sized> {
    name: &'static str,
    current: Box<FnMut(&P) -> Cr + 'a>,
    new: Box<FnMut(&P) -> Nr + 'a>,
    setup: Option<Box<FnMut(P) -> P + 'a>>,
    run_if: Option<Box<FnMut(&P) -> bool + 'a>>,
    ignore_if: Option<Box<FnMut(&P) -> bool + 'a>>,
    experiment: PhantomData<E>
}

impl<'a, P, Cr: Clone, Nr: Clone, E: Experiment<P, Cr, Nr>> ExperimentBuilder<'a, P, Cr, Nr, E> {
    pub fn setup<S>(mut self, setup: S) -> Self
            where S: FnMut(P) -> P + 'a {
        self.setup = Some(Box::new(setup));
        self
    }

    pub fn run_if<R>(mut self, run_if: R) -> Self
            where R: FnMut(&P) -> bool + 'a {
        self.run_if = Some(Box::new(run_if));
        self
    }

    pub fn ignore_if<I>(mut self, ignore_if: I) -> Self
            where I: FnMut(&P) -> bool + 'a {
        self.ignore_if = Some(Box::new(ignore_if));
        self
    }

    pub fn carry_out(mut self, mut param: P) -> Cr {
        if !E::enabled() {
            return (self.current)(&param);
        }
        if let Some(mut setup) = self.setup {
            param = setup(param);
        }
        if let Some(mut run_if) = self.run_if {
            if !run_if(&param) {
                return (self.current)(&param);
            }
        }

        let mut rng = thread_rng();
        let mut current_val = None;
        let mut new_val = None;
        let mut current_duration = 0;
        let mut new_duration = 0;
        let mut order = [0, 1];
        rng.shuffle(&mut order);
        for i in &order {
            match *i {
                0 => {
                    let start = precise_time_ns();
                    current_val = Some((self.current)(&param));
                    current_duration = precise_time_ns() - start;
                }
                _ => {
                    let start = precise_time_ns();
                    new_val = Some((self.new)(&param));
                    new_duration = precise_time_ns() - start;
                }
            }
        }

        let ignore = if let Some(mut ignore_if) = self.ignore_if {
            ignore_if(&param)
        } else {
            false
        };

        E::publish(ExperimentResult::new(
            self.name,
            CohortResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
            CohortResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
            if ignore {
                MatchType::Ignored
            } else if E::compare(&current_val.as_ref().unwrap(), &new_val.as_ref().unwrap()) {
                MatchType::Match
            } else {
                MatchType::NoMatch
            }
        ));

        current_val.unwrap()
    }
}


pub trait Experiment<P, Cr: Clone, Nr: Clone> {
    fn publish(_: ExperimentResult<Cr, Nr>) {}

    fn enabled() -> bool {
        true
    }

    fn new<'a, C, N>(name: &'static str, current: C, new: N) -> ExperimentBuilder<'a, P, Cr, Nr, Self>
        where C: FnMut(&P) -> Cr + 'a,
              N: FnMut(&P) -> Nr + 'a {
        ExperimentBuilder {
            name: name,
            current: Box::new(current),
            new: Box::new(new),
            setup: None,
            run_if: None,
            ignore_if: None,
            experiment: PhantomData
        }
    }

    fn compare(current_result: &Cr, new_result: &Nr) -> bool;
}

#[cfg(test)]
mod test {
    use super::{Experiment, ExperimentResult};
    struct TestExperiment;

    impl Experiment<Vec<char>, String, String> for TestExperiment {
        fn publish(result: ExperimentResult<String, String>) {
            println!("{:#?}", result);
        }

        fn compare(current_result: &String, new_result: &String) -> bool {
            current_result == new_result
        }
    }

    #[test]
    fn it_works() {
        let a_str = vec!['a', 'b', 'c'];
        let a = TestExperiment::new("experiment!",
            |p| {
                println!("current went!");
                p.clone().into_iter().collect()
            },
            |p| {
                println!("new went!");
                let mut p = p.clone();
                p.reverse();
                p.into_iter().collect()
            })
            .setup(|mut p| { p.sort(); p })
            .run_if(|_| true)
            .ignore_if(|_| false)
            .carry_out(a_str);
        println!("{}", a);
    }
}
