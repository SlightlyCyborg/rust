// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


/*
  A parallel version of fibonacci numbers.

  This version is meant mostly as a way of stressing and benchmarking
  the task system. It supports a lot of old command-line arguments to
  control how it runs.

*/

extern crate getopts;

use std::os;
use std::result::Result::{Ok, Err};
use std::task;
use std::time::Duration;

fn fib(n: int) -> int {
    fn pfib(tx: &Sender<int>, n: int) {
        if n == 0 {
            tx.send(0);
        } else if n <= 2 {
            tx.send(1);
        } else {
            let (tx1, rx) = channel();
            let tx2 = tx1.clone();
            task::spawn(move|| pfib(&tx2, n - 1));
            let tx2 = tx1.clone();
            task::spawn(move|| pfib(&tx2, n - 2));
            tx.send(rx.recv() + rx.recv());
        }
    }

    let (tx, rx) = channel();
    spawn(move|| pfib(&tx, n) );
    rx.recv()
}

struct Config {
    stress: bool
}

fn parse_opts(argv: Vec<String> ) -> Config {
    let opts = vec!(getopts::optflag("", "stress", ""));

    let argv = argv.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let opt_args = argv.slice(1, argv.len());

    match getopts::getopts(opt_args, opts.as_slice()) {
      Ok(ref m) => {
          return Config {stress: m.opt_present("stress")}
      }
      Err(_) => { panic!(); }
    }
}

fn stress_task(id: int) {
    let mut i = 0i;
    loop {
        let n = 15i;
        assert_eq!(fib(n), fib(n));
        i += 1;
        println!("{}: Completed {} iterations", id, i);
    }
}

fn stress(num_tasks: int) {
    let mut results = Vec::new();
    for i in range(0, num_tasks) {
        results.push(task::try_future(move|| {
            stress_task(i);
        }));
    }
    for r in results.into_iter() {
        r.unwrap();
    }
}

fn main() {
    let args = os::args();
    let args = if os::getenv("RUST_BENCH").is_some() {
        vec!("".to_string(), "20".to_string())
    } else if args.len() <= 1u {
        vec!("".to_string(), "8".to_string())
    } else {
        args.into_iter().map(|x| x.to_string()).collect()
    };

    let opts = parse_opts(args.clone());

    if opts.stress {
        stress(2);
    } else {
        let max = from_str::<uint>(args[1].as_slice()).unwrap() as int;

        let num_trials = 10;

        for n in range(1, max + 1) {
            for _ in range(0u, num_trials) {
                let mut fibn = None;
                let dur = Duration::span(|| fibn = Some(fib(n)));
                let fibn = fibn.unwrap();

                println!("{}\t{}\t{}", n, fibn, dur);
            }
        }
    }
}
