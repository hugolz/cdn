#[test]
fn json() {
    let x = rocket::serde::json::serde_json::json!({"abc": "hi"});
    println!("{x}");

    let y = rocket::serde::json::serde_json::Value::Bool(false);
}
