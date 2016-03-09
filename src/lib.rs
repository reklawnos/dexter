//! A library for using the Branch by Abstraction pattern to test for performance and correctness.
//!
//! # Examples
//! ```
//! use dexter::*;
//! struct ExamplePublisher;
//!
//! impl Publisher<Vec<char>, String, String> for ExamplePublisher {
//!     fn publish(&mut self, result: ExperimentResult<String, String>) {
//!         println!("{:#?}", result);
//!     }
//!
//!     fn compare(&mut self, current_result: &String, new_result: &String) -> bool {
//!         current_result == new_result
//!     }
//! }
//!
//! fn main() {
//!   let chars = vec!['a', 'b', 'c'];
//!   let mut p = ExamplePublisher;
//!   let result = Experiment::new("experiment",
//!       |a: &Vec<char>| {
//!           a.clone().into_iter().collect()
//!       },
//!       |a: &Vec<char>| {
//!           a.clone().into_iter().collect()
//!       })
//!       .run_if(|p| { p.len() == 3 })
//!       .carry_out(chars.clone(), &mut p);
//!   println!("{}", result);
//! }
//! ```
#![warn(missing_docs, missing_debug_implementations,
        missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code,
        unstable_features, unused_extern_crates,
        unused_import_braces, unused_qualifications,
        unused_results, variant_size_differences)]
extern crate rand;
extern crate time;

use std::fmt;
use rand::{thread_rng, Rng};
use time::precise_time_ns;

/// Result for a subject in an experiment.
#[derive(Debug)]
pub struct SubjectResult<T: Clone> {
    /// Time spent running this subject's code
    pub duration: f64,
    /// The value produced by this subject's code
    pub result: T
}

impl<T: Clone> SubjectResult<T> {
    fn new(duration: f64, result: &T) -> Self {
        SubjectResult {
            duration: duration,
            result: result.clone()
        }
    }
}

/// Result of an experiment.
#[derive(Debug)]
pub struct ExperimentResult<Cr: Clone, Nr: Clone> {
    /// Name of the experiment
    pub name: &'static str,
    /// Result for the current subject
    pub current: SubjectResult<Cr>,
    /// Result for the new subject
    pub new: SubjectResult<Nr>,
    /// The match type for this experiment
    pub match_type: MatchType
}



/// Matching type for experiments.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchType {
    /// The two subjects had matching results
    Match,
    /// The two subjects did not have matching results
    NoMatch,
    /// The matching state was ignored
    Ignored
}

impl<Cr: Clone, Nr: Clone> ExperimentResult<Cr, Nr> {
    fn new(name: &'static str, current: SubjectResult<Cr>, new: SubjectResult<Nr>, match_type: MatchType) -> Self {
        ExperimentResult {
            name: name,
            current: current,
            new: new,
            match_type: match_type
        }
    }
}

/// A struct for building Dexter experiments.
#[must_use]
pub struct Experiment<'a, P, Cr: Clone, Nr: Clone> {
    name: &'static str,
    current: Box<FnMut(&P) -> Cr + 'a>,
    new: Box<FnMut(&P) -> Nr + 'a>,
    setup: Option<Box<FnMut(P) -> P + 'a>>,
    run_if: Option<Box<FnMut(&P) -> bool + 'a>>,
    ignore_if: Option<Box<FnMut(&P) -> bool + 'a>>
}

impl<'a, P, Cr: Clone, Nr: Clone> Experiment<'a, P, Cr, Nr> {
    /// Constructs a new `Experiment` with a current and given subject.
    pub fn new<C, N>(name: &'static str, current_subject: C, new_subject: N) -> Self
        where C: FnMut(&P) -> Cr + 'a,
              N: FnMut(&P) -> Nr + 'a {
        Experiment {
            name: name,
            current: Box::new(current_subject),
            new: Box::new(new_subject),
            setup: None,
            run_if: None,
            ignore_if: None
        }
    }

    /// Adds a setup step to the experiment.
    ///
    /// The setup function can alter the parameter that's passed into the experiment before it is
    /// passed to the `run_if` closure, the `ignore_if` closure, and the two subject closures.
    pub fn setup<S>(mut self, setup: S) -> Self
            where S: FnMut(P) -> P + 'a {
        self.setup = Some(Box::new(setup));
        self
    }

    /// Adds a check step that will disable the experiment in certain cases.
    ///
    /// If the passed closure returns false when passed the experiment's parameter, then the
    /// experiment will return the current subject's result without publishing.
    pub fn run_if<R>(mut self, run_if: R) -> Self
            where R: FnMut(&P) -> bool + 'a {
        self.run_if = Some(Box::new(run_if));
        self
    }

    /// Adds an check step that will ignore mismatches in certain cases.
    ///
    /// If the passed closure returns true when passed the experiment's parameter, then the
    /// result will have a `MatchType` of `Ignored`.
    pub fn ignore_if<I>(mut self, ignore_if: I) -> Self
            where I: FnMut(&P) -> bool + 'a {
        self.ignore_if = Some(Box::new(ignore_if));
        self
    }

    /// Carry out the experiment given a parameter and a publisher.
    ///
    /// Returns the result of the current subject closure.
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
            SubjectResult::new(current_duration as f64 * 1e-9, &current_val.as_ref().unwrap()),
            SubjectResult::new(new_duration as f64 * 1e-9, &new_val.as_ref().unwrap()),
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

impl<'a, P, Cr: Clone, Nr: Clone> fmt::Debug for Experiment<'a, P, Cr, Nr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fn_str = "Fn(...)";
        let some_fn_str = "Some(Fn(...))";
        let none_str = "None";
        f.debug_struct("Experiment")
            .field("name", &self.name)
            .field("current", &fn_str)
            .field("new", &fn_str)
            .field("setup", if self.setup.is_some() {
                &some_fn_str
            } else {
                &none_str
            })
            .field("run_if", if self.run_if.is_some() {
                &some_fn_str
            } else {
                &none_str
            })
            .field("ignore_if", if self.ignore_if.is_some() {
                &some_fn_str
            } else {
                &none_str
            })
            .finish()
    }
}

/// Trait for publishers, which are used by Dexter to store results of experiments.
pub trait Publisher<P, Cr: Clone, Nr: Clone> {
    /// Publish the result of an experiment.
    fn publish(&mut self, result: ExperimentResult<Cr, Nr>);

    /// Comparison function for the results of the subjects.
    ///
    /// This function should return `true` if the results "match," and `false` if they do not.
    fn compare(&mut self, current_result: &Cr, new_result: &Nr) -> bool;

    /// Only run the experiment in some cases.
    ///
    /// If `enabled` returns false, then the result of the current subject is used and no results
    /// are published. This is meant to be an inexpensive function that gets called when every
    /// experiment runs.
    fn enabled(&mut self) -> bool {
        true
    }
}
