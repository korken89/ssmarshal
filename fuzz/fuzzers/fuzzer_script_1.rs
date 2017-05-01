#![feature(test)]

#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate ssmarshal;

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Debug)]
struct Simple {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u8,
    f: f32,
    g: u8,
    h: f64,
}

#[derive(Deserialize, Debug)]
struct Unit;

#[derive(Deserialize, Debug)]
struct Complex {
    a: Simple,
    e: Unit,
    b: Simple,
    c: [u8; 7],
    d: (),
    f: [Unit; 3]
}


#[derive(Deserialize, Debug)]
enum ComplexEnum {
    A,
    B(Simple),
    C(u8, u16),
    D(isize),
    E {
        foo: Simple,
    },
    F {
        bar: Simple,
        qux: char,
        baz: Simple,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() >= ::std::mem::size_of::<ComplexEnum>() { match ssmarshal::deserialize::<ComplexEnum>(data) {
            Ok((val, bytes)) => { },
            Err(ssmarshal::Error::InvalidRepresentation) => { },
            Err(ssmarshal::Error::Custom(_)) => { },
            Err(e) => panic!("{:?}", e),
        }
    }
});
