use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Props<'a> {
    title: &'a str,
}

kivact::view::constant_value!(Props<'static>);

fn borrowed(label: &str) -> Value<Props<'static>> {
    Props { title: label }.into_value()
}

fn main() {}
