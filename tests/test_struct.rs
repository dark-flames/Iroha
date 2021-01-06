use iroha::ToTokens;
use helpers::TokenizableError;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};

#[derive(ToTokens)]
struct TestUnit;

#[derive(ToTokens)]
struct TestTuple(i32, i64, Vec<u8>);

#[derive(ToTokens)]
#[Iroha(mod_path = "test")]
struct TestStruct {
    basic: i32,
    vec: Vec<i64>,
    string: String,
    vec_string: Vec<String>,
    option_string: Option<String>,
    option_string_none: Option<String>,
    result: Result<String, TokenizableError>,
    map: HashMap<usize, String>,
    hash_set: HashSet<String>,
    str: &'static str,
}

fn get_result<T: ToTokens>(value: T) -> String {
    let tokens = quote::quote! {
        #value
    };

    tokens
        .to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn test_unit() {
    let unit = TestUnit;
    assert_eq!(get_result(&unit), "TestUnit::new()");
    let tuple = TestTuple(1, -1, vec![1, 2, 3]);
    assert_eq!(
        get_result(&tuple),
        "TestTuple::new(1i32,-1i64,vec![1u8,2u8,3u8])"
    );
}

#[test]
fn test_struct() {
    let st = TestStruct {
        basic: 0,
        vec: vec![1, 2, 3],
        string: "iroha".to_string(),
        vec_string: vec![
            "yuikino".to_string(),
            "yui".to_string(),
            "iroha".to_string(),
        ],
        option_string: Some("iroha".to_string()),
        option_string_none: None,
        result: Ok("233".to_string()),
        map: vec![(1, "yuikino".to_string()), (2, "yui".to_string())]
            .into_iter()
            .collect(),
        hash_set: vec![
            "yuikino".to_string(),
            "yui".to_string(),
            "iroha".to_string(),
        ]
        .into_iter()
        .collect(),
        str: "test",
    };

    let result = quote::quote! {#st};

    println!("{}", result);
}