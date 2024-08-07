/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use boa_engine::{Context, JsResult, JsValue, Source};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestObject {
    a: u8,
    b: u8,
    c: u8,
}

#[test]
fn test() {
    let test_object: TestObject = serde_json::from_str(
        run_js(
            r#"
let object = {
        a: 1,
        b: 2,
        c: 3
    };


    object.d = 4;
    return object

    "#,
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    let sowieso_correct = TestObject { a: 1, b: 2, c: 3 };
    assert_eq!(test_object.a, sowieso_correct.a);
    assert_eq!(test_object.b, sowieso_correct.b);
    assert_eq!(test_object.c, sowieso_correct.c);
}

pub(crate) fn run_js(js: &str) -> JsResult<JsonString> {
    let mut js_code_string = r#"
let result =( () => {
"#
    .to_string();

    js_code_string.push_str(js);
    js_code_string.push_str(
        r#"})();
JSON.stringify(result);

"#,
    );
    let js_code = js_code_string.as_str();

    // Instantiate the execution context
    let mut context = Context::default();

    // Parse the source code
    let _ = context.module_loader();
    let resultstr = context.eval(Source::from_bytes(js_code))?;
    let resultstring: String = format!("{}", &JsValue::display(&resultstr)).clone();
    let resultjson = {
        let mut o = resultstring.chars();
        o.next();
        o.next_back();
        o.as_str()
    };

    Ok(resultjson.to_string())
}
pub(crate) type JsonString = String;
pub(crate) enum RunJSAndDeserializeResult<T> {
    Ok(T),
    JsError(String),
    SerdeError(serde_json::Error),
}

pub(crate) fn run_js_and_deserialize<T>(js: &str) -> RunJSAndDeserializeResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    match run_js(js) {
        Ok(result) => match serde_json::from_str(result.as_str()) {
            Ok(t) => RunJSAndDeserializeResult::Ok(t),
            Err(e) => RunJSAndDeserializeResult::SerdeError(e),
        },
        Err(e) => RunJSAndDeserializeResult::JsError(e.to_string()),
    }
}
