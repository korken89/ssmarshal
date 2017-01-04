// Copyright (c) 2017 The Robigalia Project Developers Licensed under the Apache License, Version
// 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. All files in the project
// carrying such notice may not be copied, modified, or distributed except according to those
// terms.

#![feature(plugin, proc_macro, test)]
#![plugin(afl_plugin)]

extern crate afl;
extern crate ssmarshal;
extern crate test;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Debug)]
struct Simple {
    a: u8,
    b: (u16, u8),
    c: char,
    d: [usize; 3],
}
    
#[derive(Deserialize, Debug)]
enum Complex {
    A,
    B(Simple),
    C(u8, u16),
    D(isize),
    E {
        foo: Simple
    },
    F {
        bar: Simple,
        baz: Simple,
        qux: char
    }
}

fn main() {
    afl::handle_bytes(|v| {
        match ssmarshal::deserialize::<Complex>(&v) {
            Ok((val, bytes)) => {
                test::black_box(val);
            },
            Err(e) => {
            }
        }
    });
}
