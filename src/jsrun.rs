/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use boa_engine::{Context, JsResult, Source};
#[test]
fn test() {
    println!(
        "{:#?}",
        jsrun(
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
    );
}

fn jsrun(js: &str) -> JsResult<JsonString> {
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
    context.module_loader()
    let resultstr = context.eval(Source::from_bytes(js_code))?;
    let resultstring = format!("{}", &resultstr.display()).clone();
    let resultjson = {
        let mut o = resultstring.chars();
        o.next();
        o.next_back();
        o.as_str()
    };

    Ok(resultjson.to_string())
}
type JsonString = String;