extern crate rand;
extern crate time;

use rand::{thread_rng, Rng};
use time::precise_time_ns;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub struct Experiment<'a, P, Cr: Clone, Nr: Clone> {
    name: &'static str,
    current: Box<FnMut(&P) -> Cr + 'a>,
    new: Box<FnMut(&P) -> Nr + 'a>,
    setup: Option<Box<FnMut(P) -> P + 'a>>,
    run_if: Option<Box<FnMut(&P) -> bool + 'a>>,
    ignore_if: Option<Box<FnMut(&P) -> bool + 'a>>
}

impl<'a, P, Cr: Clone, Nr: Clone> Experiment<'a, P, Cr, Nr> {
    pub fn new<C, N>(name: &'static str, current: C, new: N) -> Self
        where C: FnMut(&P) -> Cr + 'a,
              N: FnMut(&P) -> Nr + 'a {
        Experiment {
            name: name,
            current: Box::new(current),
            new: Box::new(new),
            setup: None,
            run_if: None,
            ignore_if: None
        }
    }

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

    pub fn carry_out<Pub: Publisher<P, Cr, Nr>>(mut self, mut param: P, publisher: &mut Pub) -> Cr {
        if !publisher.enabled() {
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

        let comparison = publisher.compare(&current_val.as_ref().unwrap(), &new_val.as_ref().unwrap());

        publisher.publish(ExperimentResult::new(
            self.name,
            CohortResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
            CohortResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
            if ignore {
                MatchType::Ignored
            } else if comparison {
                MatchType::Match
            } else {
                MatchType::NoMatch
            }
        ));

        current_val.unwrap()
    }
}


pub trait Publisher<P, Cr: Clone, Nr: Clone> {
    fn publish(&mut self, _: ExperimentResult<Cr, Nr>) {}

    fn enabled(&mut self) -> bool {
        true
    }

    fn compare(&mut self, current_result: &Cr, new_result: &Nr) -> bool;
}
