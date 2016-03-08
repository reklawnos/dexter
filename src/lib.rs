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

pub struct ExperimentBuilder<'a, CurrentResult: Clone, NewResult: Clone, Param: Clone, E: Experiment<CurrentResult, NewResult, Param> + ?Sized> {
    name: &'static str,
    current: Box<FnMut(Param) -> CurrentResult + 'a>,
    new: Box<FnMut(Param) -> NewResult + 'a>,
    setup: Option<Box<FnMut(Param) -> Param + 'a>>,
    run_if: Option<Box<FnMut(Param) -> bool + 'a>>,
    experiment: PhantomData<E>
}

impl<'a, CurrentResult: Clone, NewResult: Clone, Param: Clone, E: Experiment<CurrentResult, NewResult, Param>> ExperimentBuilder<'a, CurrentResult, NewResult, Param, E> {
    pub fn setup<S>(mut self, setup: S) -> Self
            where S: FnMut(Param) -> Param + 'a {
        self.setup = Some(Box::new(setup));
        self
    }
    pub fn run_if<R>(mut self, run_if: R) -> Self
            where R: FnMut(Param) -> bool + 'a {
        self.run_if = Some(Box::new(run_if));
        self
    }

    pub fn carry_out(mut self, mut param: Param) -> CurrentResult {
        if let Some(mut s) = self.setup {
            param = s(param);
        }
        match self.run_if {
            Some(mut r) => {
                if !r(param.clone()) {
                    return (self.current)(param.clone());
                }
            }
            _ => {}
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
                    current_val = Some((self.current)(param.clone()));
                    current_duration = precise_time_ns() - start;
                }
                _ => {
                    let start = precise_time_ns();
                    new_val = Some((self.new)(param.clone()));
                    new_duration = precise_time_ns() - start;
                }
            }
        }
        E::publish(ExperimentResult::new(
            CohortResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
            CohortResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
            self.name
        ));
        current_val.unwrap()
    }
}


pub trait Experiment<CurrentResult: Clone, NewResult: Clone, Param: Clone> {
    fn publish(_: ExperimentResult<CurrentResult, NewResult>) {}

    fn new<'a, C, N>(name: &'static str, current: C, new: N) -> ExperimentBuilder<'a, CurrentResult, NewResult, Param, Self>
        where C: FnMut(Param) -> CurrentResult + 'a,
              N: FnMut(Param) -> NewResult + 'a {
        ExperimentBuilder {
            name: name,
            current: Box::new(current),
            new: Box::new(new),
            setup: None,
            run_if: None,
            experiment: PhantomData
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Experiment, ExperimentResult};
    struct TestExperiment;

    impl Experiment<String, String, Vec<char>> for TestExperiment {
        fn publish(result: ExperimentResult<String, String>) {
            println!("{:#?}", result);
        }
    }

    #[test]
    fn it_works() {
        let a_str = vec!['a', 'b', 'c'];
        let a = TestExperiment::new("experiment!",
            |p| {
                println!("current went!");
                p.into_iter().collect()
            },
            |p| {
                println!("new went!");
                p.into_iter().collect()
            })
            .setup(|mut p| { p.sort(); p })
            .run_if(|_| true)
            .carry_out(a_str.clone());
        println!("{}", a);
    }
}
