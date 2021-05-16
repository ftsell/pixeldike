#[macro_use]
extern crate rustler;
use rustler::{Encoder, Env, Error, ResourceArc, Term};

mod atoms {
    rustler::rustler_atoms! {
        atom ok;
    }
}

type SafeVec = ResourceArc<Vec<u8>>;

rustler_export_nifs!(
    "Elixir.NativeArray",
    [("new", 1, new), ("set", 3, set)],
    None
);

fn new<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let length: usize = args[0].decode()?;
    let result: SafeVec = ResourceArc::new(Vec::with_capacity(length));

    Ok(result.into())
}

fn set<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let array: SafeVec = args[0].decode()?;
    let position: i64 = args[1].decode()?;
    let value: u8 = args[2].decode()?;

    Ok(2.encode(env))
}
