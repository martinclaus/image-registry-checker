use dotenv::dotenv;
use serde::Deserialize;
use std::{env, process::Stdio};
use tokio::process::Command;
use warp::{http::Response, Filter};

#[derive(Deserialize)]
struct ImageSlug {
    image: String,
}

async fn check_image_slug(image: impl AsRef<str>) -> std::io::Result<bool> {
    // spawn crane to look up image
    let mut child = Command::new(get_crane_command())
        .arg("manifest")
        .arg(image.as_ref())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let status = child.wait().await?;
    Ok(status.success())
}

fn get_crane_command() -> String {
    match env::var("CRANE") {
        Ok(val) => val,
        Err(_) => "crane".to_owned(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let check_image = warp::get()
        .and(warp::path("exists"))
        .and(warp::query::<ImageSlug>())
        .and_then(|p: ImageSlug| async move {
            let response = match check_image_slug(p.image).await {
                Ok(true) => Response::builder()
                    .status(warp::http::StatusCode::OK)
                    .body("ok"),
                Ok(false) => Response::builder()
                    .status(warp::http::StatusCode::NOT_FOUND)
                    .body("Image does not exist"),
                Err(_) => Response::builder()
                    .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(""),
            };
            response.map_err(|_| warp::reject::reject())
        });

    let health_check = warp::get()
        .and(warp::path("health"))
        .map(|| Response::builder().body("Ok"));

    warp::serve(check_image.or(health_check))
        .run(([127, 0, 0, 1], 8080))
        .await
}

#[cfg(test)]
mod test {
    use super::*;
    use temp_env::with_var;

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
        println!("{:?}", res);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(!res)
        }
    }

    #[tokio::test]
    async fn check_image_slug_returns_error_on_failed_spawn() {
        let res = with_var("CRANE", Some("cran"), || async move {
            env::set_var("CRANE", "cran");
            let res = check_image_slug("docker.io/non-existent").await;
            res
        })
        .await;
        assert!(res.is_err());
    }
}
