fn main() {
    let json = serde_json::json!({"foo": "bar"});
    println!("{:?}", json);
    println!("{}", json);
}
