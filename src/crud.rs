use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
#[derive(Serialize, Deserialize)]
struct Items {
    id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Length {
    total_items: Option<u32>,
    code: Option<u16>,
    error: Option<u16>,
    items: Option<Vec<Items>>,
}
fn construct_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers
}
fn construct_headers_form() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("multipart/form-data"),
    );
    headers
}
pub struct Collection {
    pub(crate) host: String,
    pub(crate) port: u16,
}
pub enum Table {
    Transaction,
    Product,
    History,
    Cart,
    Background,
    Header,
}
impl Table {
    fn get_string(&self) -> String {
        match self {
            Table::Transaction => "transaction".to_string(),
            Table::Product => "product".to_string(),
            Table::History => "history".to_string(),
            Table::Cart => "cart".to_string(),
            Table::Background => "background".to_string(),
            Table::Header => "header".to_string(),
        }
    }
    fn url_struct(&self, con: &Collection) -> String {
        format!(
            "{}:{}/api/collections/{}/records",
            &con.host,
            &con.port,
            &self.get_string()
        )
    }
    pub async fn list(&self, con: &Collection, param: Option<&str>) -> String {
        let mut url = self.url_struct(con);
        if param.is_some() {
            url.push_str(["?", param.unwrap()].concat().as_str())
        }
        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(res) => res.text().await.unwrap_or("no message".to_string()),
            Err(_) => "{\"error\":400}".to_string(),
        }
    }
    pub async fn select(&self, con: &Collection, id: &str) -> String {
        let url = [&self.url_struct(con), "/", id].concat();
        let client = reqwest::Client::new();
        match client.get(&url).send().await {
            Ok(result) => result.text().await.unwrap_or("no message".to_string()),
            Err(_error) => String::from("{\"error\":400}"),
        }
    }
    pub async fn create(&self, con: &Collection, data: &str) -> String {
        let client = reqwest::Client::new();
        match client
            .post(&self.url_struct(con))
            .headers(construct_headers())
            .body(data.to_owned())
            .send()
            .await
        {
            Ok(result) => result.text().await.unwrap_or("no message".to_string()),
            Err(_error) => String::from("{\"error\":400}"),
        }
    }
    pub async fn update(&self, con: &Collection, id: &str, data: &str) -> String {
        let url = [&self.url_struct(con), "/", id].concat();
        let client = reqwest::Client::new();
        match client
            .patch(&url)
            .headers(construct_headers())
            .body(data.to_owned())
            .send()
            .await
        {
            Ok(result) => result.text().await.unwrap_or("no message".to_string()),
            Err(_error) => String::from("{\"error\":400}"),
        }
    }
    pub async fn delete(&self, con: &Collection, id: &str) -> String {
        let url = [&self.url_struct(con), "/", id].concat();
        let client = reqwest::Client::new();
        match client
            .delete(&url)
            .headers(construct_headers())
            .send()
            .await
        {
            Ok(result) => result.text().await.unwrap_or("no message".to_string()),
            Err(_error) => String::from("{\"error\":400}"),
        }
    }
    pub async fn list_all(&self, con: &Collection, param: Option<&str>) -> String {
        let result = &self.list(con, Some("perPage=1")).await;
        let now: Length = serde_json::from_str(result).unwrap();
        if now.error.is_some() {
            return String::from("{\"error\":400}");
        } else if now.code.is_some() {
            return String::from("{\"code\":400}");
        } else {
            match param {
                Some(e) => {
                    self.list(
                        con,
                        Some(&format!("perPage={}&{}", now.total_items.unwrap(), e)),
                    )
                    .await
                }
                None => {
                    self.list(con, Some(&format!("perPage={}", now.total_items.unwrap())))
                        .await
                }
            }
        }
    }
    pub async fn delete_all(&self, con: &Collection, param: Option<&str>) -> String {
        let listed: Length = serde_json::from_str(&self.list_all(con, None).await).unwrap();
        if listed.error.is_some() {
            return String::from("{\"error\":400}");
        } else if listed.code.is_some() {
            return String::from("{\"code\":400}");
        } else {
            for i in listed.items.unwrap() {
                self.delete(con, &i.id).await;
            }
            self.list(con, param).await
        }
    }
    pub async fn update_or_create(&self, con: &Collection, id: &str, data: &str) -> String {
        let listed: Length = serde_json::from_str(&self.update(con, id, data).await).unwrap();
        if listed.error.is_some() {
            return String::from("{\"error\":400}");
        } else if listed.code.is_some() {
            self.create(con, data).await
        } else {
            serde_json::to_string(&listed).unwrap()
        }
    }
    #[tokio::main]
    pub async fn update_form(&self, con: &Collection, id: &str, path: &str) -> String {
        let url = [&self.url_struct(con), "/", id].concat();
        let file = fs::read(path).unwrap();
        let file_part = reqwest::multipart::Part::bytes(file)
            .file_name("bg.jpg")
            .mime_str("image/jpg")
            .unwrap();
        let form = reqwest::multipart::Form::new().part("img", file_part);
        let client = reqwest::Client::new();
        match client.patch(url).multipart(form).send().await {
            Ok(res) => res.text().await.unwrap_or("no message".to_string()),
            Err(_) => "{\"error\":400}".to_string(),
        }
    }
    pub fn blocking(&self, con: &Collection, path: &str) -> String {
        let client = reqwest::blocking::Client::new();
        let form = reqwest::blocking::multipart::Form::new()
            .file("img", path)
            .unwrap();
        match client.post(&self.url_struct(con)).multipart(form).send() {
            Ok(res) => res.text().unwrap(),
            Err(_) => "error".to_string(),
        }
    }
}
//hello
