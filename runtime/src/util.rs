use std::collections::VecDeque;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    error::{Error as NomError, ErrorKind as NomErrorKind, ParseError},
    IResult,
};

use rusty_jsc::{JSContext, JSValue};

/*

#[callback]
fn format(context: JSContext, _function: JSObject, _this: JSObject, arguments: &[JSValue]) {
    if arguments.is_empty() {
        return println!();
    }
    if arguments[0].is_string(&context) {}
    for arg in arguments {}
}

The util.format() method returns a formatted string using the first argument as a printf-like format string which can contain zero or more format specifiers. Each specifier is replaced with the converted value from the corresponding argument. Supported specifiers are:

    %s: String will be used to convert all values except BigInt, Object and -0. BigInt values will be represented with an n and Objects that have no user defined toString function are inspected using util.inspect() with options { depth: 0, colors: false, compact: 3 }.
    %d: Number will be used to convert all values except BigInt and Symbol.
    %i: parseInt(value, 10) is used for all values except BigInt and Symbol.
    %f: parseFloat(value) is used for all values expect Symbol.
    %j: JSON. Replaced with the string '[Circular]' if the argument contains circular references.
    %o: Object. A string representation of an object with generic JavaScript object formatting. Similar to util.inspect() with options { showHidden: true, showProxy: true }. This will show the full object including non-enumerable properties and proxies.
    %O: Object. A string representation of an object with generic JavaScript object formatting. Similar to util.inspect() without options. This will show the full object not including non-enumerable properties and proxies.
    %c: CSS. This specifier is ignored and will skip any CSS passed in.
    %%: single percent sign ('%'). This does not consume an argument.
    Returns: <string> The formatted string



 */

//pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[derive(Clone)]
struct ArgIn<'a> {
    input: &'a str,
    arguments: &'a [JSValue],
    context: &'a JSContext,
}

type ResWithArg<'a> = IResult<ArgIn<'a>, VecDeque<String>, nom::error::Error<&'a str>>;
type Res<'a> = IResult<&'a str, &'a str>;

fn take_until_percent(mut arg_in: ArgIn) -> ResWithArg {
    match take_until("%")(arg_in.input) {
        Ok((rest, out)) => {
            arg_in.input = rest;
            let (new_arg_in, mut next_out) = alt_percent_sign(arg_in)?;
            next_out.push_front(out.to_string());
            Ok((new_arg_in, next_out))
        }
        Err(rest) => Err(rest),
    }
}

impl<'a> ParseError<ArgIn<'a>> for nom::error::Error<&'a str> {
    fn from_error_kind(arg_in: ArgIn<'a>, kind: nom::error::ErrorKind) -> Self {
        nom::error::Error {
            input: arg_in.input,
            code: kind,
        }
    }

    fn append(arg_in: ArgIn<'a>, kind: nom::error::ErrorKind, _: Self) -> Self {
        nom::error::Error {
            input: arg_in.input,
            code: kind,
        }
    }
}

fn alt_percent_sign(arg_in: ArgIn) -> ResWithArg {
    alt((percent_s, percent_d, percent_i))(arg_in)
    //%s, %d, %i, %f, %j, %o, %O, %c, %%
}

fn percent_s(mut arg_in: ArgIn) -> ResWithArg {
    let tmp: Res = tag("%s")(arg_in.input);
    let (rest, res) = tmp?;
    arg_in.input = rest;
    if arg_in.arguments.is_empty() {
        // Don't replace %s if arguments is empty
        Ok((arg_in, VecDeque::from([res.to_string()])))
    } else {
        let res = &arg_in.arguments[0];
        arg_in.arguments = &arg_in.arguments[1..];
        if res.is_string(arg_in.context) {
            let res = res.to_js_string(arg_in.context).unwrap().to_string();
            Ok((arg_in, VecDeque::from([res])))
        } else {
            Err(nom::Err::Error(NomError::new(
                "Parse Error: Unknown JSValue type",
                NomErrorKind::NoneOf,
            )))
        }
    }
}

fn percent_d(mut arg_in: ArgIn) -> ResWithArg {
    let tmp: Res = tag("%d")(arg_in.input);
    let (rest, res) = tmp?;
    arg_in.input = rest;
    if arg_in.arguments.is_empty() {
        // Don't replace %d if arguments is empty
        Ok((arg_in, VecDeque::from([res.to_string()])))
    } else {
        let res = &arg_in.arguments[0];
        arg_in.arguments = &arg_in.arguments[1..];
        if res.is_string(arg_in.context) {
            let res = res.to_js_string(arg_in.context).unwrap().to_string();
            Ok((arg_in, VecDeque::from([res])))
        } else {
            Err(nom::Err::Error(NomError::new(
                "Parse Error: Unknown JSValue type",
                NomErrorKind::NoneOf,
            )))
        }
    }
}

fn percent_i(mut arg_in: ArgIn) -> ResWithArg {
    fn float_to_strint(f: f64) -> String {
        if f.is_infinite() || f.is_nan() || f.is_subnormal() {
            "NaN".to_string()
        } else {
            unsafe { f.to_int_unchecked::<u64>() }.to_string()
        }
    }

    let tmp: Res = tag("%i")(arg_in.input);
    let (rest, res) = tmp?;
    arg_in.input = rest;
    if arg_in.arguments.is_empty() {
        // Don't replace %d if arguments is empty
        Ok((arg_in, VecDeque::from([res.to_string()])))
    } else {
        let res = &arg_in.arguments[0];
        arg_in.arguments = &arg_in.arguments[1..];
        if res.is_string(arg_in.context) {
            let res = match res.to_number(arg_in.context) {
                Ok(num) => float_to_strint(num.trunc()),
                Err(_) => {
                    let str = res.to_js_string(arg_in.context).unwrap().to_string();
                    if let Ok(test) = str.replace(',', ".").parse::<f64>() {
                        float_to_strint(test.trunc())
                    } else {
                        "NaN".to_string()
                    }
                }
            };
            Ok((arg_in, VecDeque::from([res])))
        } else if res.is_number(arg_in.context) {
            let res = res.to_number(arg_in.context).unwrap().to_string();
            Ok((arg_in, VecDeque::from([res])))
        } else {
            Ok((arg_in, VecDeque::from(["NaN".to_string()])))
        }
    }
}

pub fn format_to_string(context: &JSContext, dest: &mut Vec<String>, src_arguments: &[JSValue]) {
    for arg in src_arguments {
        if arg.is_string(context) {
            dest.push(arg.to_js_string(context).unwrap().to_string());
        } else if arg.is_bool(context) {
            if arg.to_bool(context) {
                dest.push("true".to_string());
            } else {
                dest.push("false".to_string());
            }
        } else if arg.is_number(context) {
            dest.push(arg.to_number(context).unwrap().to_string());
        } else if arg.is_null(context) {
            dest.push("null".to_string());
        } else if arg.is_undefined(context) {
            dest.push("undefined".to_string());
        } else if arg.is_date(context) {
            dest.push(arg.to_js_string(context).unwrap().to_string());
        }
    }
}

pub fn format_parser<'a>(
    context: &'a JSContext,
    arguments: &'a [JSValue],
) -> Result<Vec<String>, ()> {
    let mut rest = &arguments[1..];
    let mut res = vec![];

    if !arguments[0].is_string(context) {
        format_to_string(context, &mut res, arguments);
        return Ok(res);
    }

    let mut arg_in = ArgIn {
        arguments: rest,
        context,
        input: &arguments[0].to_js_string(context).unwrap().to_string(),
    };

    loop {
        match take_until_percent(arg_in) {
            Ok((new_arg_in, out)) => {
                arg_in = new_arg_in;
                rest = arg_in.arguments;
                res.append(&mut out.into());
            }
            Err(nom::Err::Error(NomError { input, code: _ })) => {
                res.push(input.to_string());
                break;
            }
            Err(_) => return Err(()), // TODO: Complete and return errors
        }
    }

    if !rest.is_empty() {
        format_to_string(context, &mut res, rest);
    }
    Ok(res)
}
