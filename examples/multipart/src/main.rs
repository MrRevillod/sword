use sword::prelude::*;

#[controller("/")]
struct TestController {}

#[routes]
impl TestController {
    #[post("/multipart")]
    async fn hello(&self, ctx: Context) -> HttpResult<HttpResponse> {
        let mut fields = vec![];
        let mut multipart = ctx.multipart().await?;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            eprintln!("Error reading multipart field: {}", e);
            HttpResponse::BadRequest().message("Failed to read multipart field")
        })? {
            let name = field.name().unwrap_or("Unnamed").to_string();
            let file_name = field.file_name().unwrap_or("No file name").to_string();

            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or("No content type".to_string());

            let data = field.bytes().await.unwrap();

            fields.push(serde_json::json!({
                "name": name,
                "file_name": file_name,
                "content_type": content_type,
                "data_length": data.len(),
            }));
        }

        Ok(HttpResponse::Ok().data(fields).message("Hello, Multipart!"))
    }
}

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_controller::<TestController>()
        .build();

    app.run().await;
}
