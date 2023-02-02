use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;

#[get("/health")]
async fn health() -> impl Responder {
    "I'm up and running!"
}

#[derive(Deserialize)]
struct ImageSlug {
    image: String,
}

#[get("/exists")]
async fn check_image_exist(info: web::Query<ImageSlug>) -> impl Responder {
    let args = info.into_inner();
    match check_image_slug(args.image).await {
        Ok(success) => {
            if success {
                HttpResponse::Ok()
            } else {
                HttpResponse::NotFound()
            }
        }
        Err(_) => HttpResponse::InternalServerError(),
    }
}

async fn check_image_slug(image: impl AsRef<str>) -> std::io::Result<bool> {
    // spawn process with crane to look up image
    let mut child = Command::new("crane")
        .arg("manifest")
        .arg(image.as_ref())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let status = child.wait().await?;
    Ok(status.success())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health).service(check_image_exist))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn check_image_slug_returns_true_on_success() {
        let res = check_image_slug("docker.io/alpine").await;
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(res)
        }
    }

    #[tokio::test]
    async fn check_image_slug_returns_false_on_invalid_slug() {
        let res = check_image_slug("docker.io/non-existent").await;
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(!res)
        }
    }

    #[tokio::test]
    async fn check_image_slug_returns_error_on_failed_spawn() {
        let res = check_image_slug("docker.io/non-existent").await;
        assert!(res.is_err());
    }
}
