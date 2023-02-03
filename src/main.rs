use clap::Parser;
use dotenv::dotenv;
use log;
use pretty_env_logger;
use serde::Deserialize;
use std::{fmt::Display, process::Stdio};
use tokio::process::Command;
use warp::{http::Response, Filter};

#[derive(Parser)]
#[command(author, version, about, long_about)]
/// This webserver serves an API to check whether a container image is present
/// in a registry or not. Currently, it only allows to query public registries
/// (no authentication implemented) and serves only http (no encription).
///
/// To query for the image `docker.io/nginx`, run
///
/// curl "http://localhost:8080/exists?image=docker.io/nginx"
struct CliArgs {
    #[arg(short, long, default_value_t = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))]
    /// IP adress to bind to
    ip: std::net::IpAddr,

    #[arg(short, long, default_value_t = 8080)]
    /// Port to listen on
    port: u16,

    #[arg(short, long, default_value = "crane", env = "CRANE_CMD")]
    /// Path and name of the crane executable
    crane_cmd: String,
}
#[derive(Deserialize)]
struct ImageSlug {
    image: String,
}

struct LogString<'a, T>(&'a T);

impl<'a> Display for LogString<'a, warp::log::Info<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "{} {} ({:.2?}) {}",
            self.0.method(),
            self.0.path(),
            self.0.elapsed(),
            self.0.status()
        );

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
struct ImageChecker {
    cmd: String,
}

impl ImageChecker {
    async fn check_image_slug(&self, image: impl AsRef<str>) -> std::io::Result<bool> {
        // spawn crane to look up image
        let mut child = Command::new(&self.cmd)
            .arg("manifest")
            .arg(image.as_ref())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let status = child.wait().await?;
        Ok(status.success())
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    if let Err(e) = dotenv() {
        log::info!("Cannot read environment from .env: {}", e);
    };

    let args = CliArgs::parse();
    let socket_addr = std::net::SocketAddr::new(args.ip, args.port);
    let checker = ImageChecker {
        cmd: args.crane_cmd,
    };

    let log = warp::log::custom(|info| match info.status() {
        code if code.as_u16() >= warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16() => {
            log::error!("{}", LogString(&info))
        }
        _ => log::info!("{}", LogString(&info)),
    });

    let check_image = warp::get()
        .and(warp::path("exists"))
        .and(warp::query::<ImageSlug>())
        .then(move |p: ImageSlug| {
            let checker = checker.clone();
            async move {
                match checker.check_image_slug(p.image).await {
                    Ok(true) => Response::builder()
                        .status(warp::http::StatusCode::OK)
                        .body("ok"),
                    Ok(false) => Response::builder()
                        .status(warp::http::StatusCode::NOT_FOUND)
                        .body("Image does not exist"),
                    Err(e) => {
                        log::error!("Spawn of subprocess failed: {}", e);
                        Response::builder()
                            .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                            .body("")
                    }
                }
            }
        });

    let health_check = warp::get()
        .and(warp::path("health"))
        .map(|| Response::builder().body("Ok"));

    warp::serve(check_image.or(health_check).with(log))
        .run(socket_addr)
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn check_image_slug_returns_true_on_success() {
        let checker = ImageChecker {
            cmd: "crane".into(),
        };
        let res = checker.check_image_slug("docker.io/alpine").await;
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(res)
        }
    }

    #[tokio::test]
    async fn check_image_slug_returns_false_on_invalid_slug() {
        let checker = ImageChecker {
            cmd: "crane".into(),
        };
        let res = checker.check_image_slug("docker.io/non-existent").await;
        println!("{:?}", res);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(!res)
        }
    }

    #[tokio::test]
    async fn check_image_slug_returns_error_on_failed_spawn() {
        let checker = ImageChecker {
            cmd: "not-existent".into(),
        };
        let res = checker.check_image_slug("docker.io/non-existent").await;
        assert!(res.is_err());
    }
}
