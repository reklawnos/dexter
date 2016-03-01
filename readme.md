Dexter
======
A cool thing that does stuff like [Scientist](https://github.com/github/scientist), but for Rust!

TODO
----
* Test modifiers
  * `context`: store the item the two options were actually performing logic on
  * `compare`: define how the results are compared
  * `setup`: do computationally intensive setup before running the experiment
  * `ignore`: don't worry about mismatch in certain scenarios
  * `clean`: simplify result type that's stored
  * `run_if`: only run experiment in certain scenarios
* `enabled` function in trait that determines if the test will run (used e.g. when only using the experiment a percentage of the time)
