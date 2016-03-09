Dexter
======
A cool thing that does stuff like [Scientist](https://github.com/github/scientist), but for Rust!

Here's an example:

```rust
use dexter::*;
struct ExamplePublisher;

impl Publisher<Vec<char>, String, String> for ExamplePublisher {
    fn publish(&mut self, result: ExperimentResult<String, String>) {
        println!("{:#?}", result);
    }

    fn compare(&mut self, current_result: &String, new_result: &String) -> bool {
        current_result == new_result
    }
}

fn main() {
  let chars = vec!['a', 'b', 'c'];
  let mut p = ExamplePublisher;
  let result = Experiment::new("experiment",
      |a: &Vec<char>| {
          a.clone().into_iter().collect()
      },
      |a: &Vec<char>| {
          a.clone().into_iter().collect()
      })
      .run_if(|p| { p.len() == 3 })
      .carry_out(chars.clone(), &mut p);
  println!("{}", result);
}
```
