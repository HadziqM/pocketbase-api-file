mod crud;
#[allow(dead_code)]
fn main() {
    let con = crud::Collection {
        host: "http://127.0.0.1".to_string(),
        port: 8090,
    };
    let res = crud::Table::Background.update_form(&con, "tah122iaqjoz0ts", "./idk.jpg");
    println!("{}", res)
}
