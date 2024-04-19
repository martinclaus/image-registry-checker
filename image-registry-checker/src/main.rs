use std::sync::Arc;

use dotenv::dotenv;
use log;
use logging::LogString;
use utoipa_swagger_ui::Config;
use warp::Filter;

fn log_func(info: warp::log::Info) {
    match info.status() {
        code if code.as_u16() >= warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16() => {
            log::error!("{}", LogString(&info))
        }
        _ => log::info!("{}", LogString(&info)),
    }
}

#[tokio::main]
async fn main() {
    logging::init();

    let env_pars_res = dotenv();

    let config = Arc::new(Config::from("/api-doc.json"));

    let args = cli::parse_args();

    if let Err(e) = env_pars_res {
        log::info!("Cannot read environment from .env: {}", e);
    }

    let socket_addr = std::net::SocketAddr::new(args.ip(), args.port());
    let crane_cmd = args.crane_cmd();

    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(filter::serve_swagger);

    warp::serve(
        filter::check_image(crane_cmd)
            .or(filter::health())
            .or(filter::api_doc())
            .or(swagger_ui)
            .with(filter::log(log_func)),
    )
    .run(socket_addr)
    .await
}

mod filter {
    use std::{convert::Infallible, sync::Arc};

    use serde::Deserialize;
    use utoipa::{IntoParams, OpenApi};
    use utoipa_swagger_ui::Config;
    use warp::{
        http::Response,
        hyper::{StatusCode, Uri},
        log::{Info, Log},
        path::{FullPath, Tail},
        Filter, Rejection, Reply,
    };

    use crate::image_exist::check_image_slug;

    #[derive(Deserialize, IntoParams)]
    #[into_params(parameter_in = Query)]
    pub struct ImageSlug {
        /// URI of an image in a public remote container repository
        #[param(inline, example = "docker.io/nginx")]
        pub image: String,
    }

    pub fn log(log_func: fn(Info)) -> Log<fn(Info)> {
        warp::log::custom(log_func)
    }

    pub async fn health_check() -> Result<String, Infallible> {
        Ok("Ok".into())
    }

    #[utoipa::path(
        get,
        path = "/health",
        responses(
            (status = 200, description = "Service is up and running", content_type="text/plain")
        )
    )]
    pub fn health() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::get().and(warp::path("health")).and_then(health_check)
    }

    #[utoipa::path(
        get,
        path = "/exists",
        params(ImageSlug),
        responses(
            (status = StatusCode::OK, description = "Image exists", content_type="text/plain"),
            (status = StatusCode::NOT_FOUND, description = "Image lookup failed", content_type="text/plain"),
            (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", content_type="text/plain"),
        )
    )]
    pub fn check_image(
        cmd: String,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::get()
            .and(warp::path("exists"))
            .and(warp::query::<ImageSlug>())
            .then(move |p: ImageSlug| {
                let cmd = cmd.clone();
                async move {
                    match check_image_slug(cmd.as_str(), p.image.as_str()).await {
                        Ok(true) => Response::builder()
                            .status(StatusCode::OK)
                            .body("ok".to_owned()),
                        Ok(false) => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(format!("Image {} does not exist", p.image)),
                        Err(_) => Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body("".to_owned()),
                    }
                }
            })
    }

    pub fn api_doc() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::path("api-doc.json")
            .and(warp::get())
            .map(|| warp::reply::json(&crate::doc::ApiDoc::openapi()))
    }

    pub async fn serve_swagger(
        full_path: FullPath,
        tail: Tail,
        config: Arc<Config<'static>>,
    ) -> Result<Box<dyn Reply + 'static>, Rejection> {
        if full_path.as_str() == "/swagger-ui" {
            return Ok(Box::new(warp::redirect::found(Uri::from_static(
                "/swagger-ui/",
            ))));
        }

        let path = tail.as_str();
        match utoipa_swagger_ui::serve(path, config) {
            Ok(file) => {
                if let Some(file) = file {
                    Ok(Box::new(
                        Response::builder()
                            .header("Content-Type", file.content_type)
                            .body(file.bytes),
                    ))
                } else {
                    Ok(Box::new(StatusCode::NOT_FOUND))
                }
            }
            Err(error) => Ok(Box::new(
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(error.to_string()),
            )),
        }
    }
}

/// Logging
mod logging {
    use pretty_env_logger;
    use std::fmt::Display;

    pub struct LogString<'a, T>(pub &'a T);

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

    pub fn init() {
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
}

/// Image checker use case
mod image_exist {
    use tokio::process::Command;

    /// Spawn crane to look up image
    pub async fn check_image_slug(
        cmd: impl AsRef<str>,
        image: impl AsRef<str>,
    ) -> std::io::Result<bool> {
        match Command::new(cmd.as_ref())
            .arg("manifest")
            .arg(image.as_ref())
            .output()
            .await
        {
            Ok(output) => {
                if !output.status.success() {
                    log::error!(
                        "\"{}\" failed with status code {}: {}",
                        cmd.as_ref(),
                        output.status.code().unwrap(),
                        String::from_utf8(output.stderr)
                            .expect(format!("got non utf-8 from {}", cmd.as_ref()).as_ref())
                    );
                }
                Ok(output.status.success())
            }
            Err(e) => {
                log::error!("Failed to spawn subprocess: {}", e);
                Err(e)
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::check_image_slug;

        #[tokio::test]
        async fn check_image_slug_returns_true_on_success() {
            let res = check_image_slug("crane", "docker.io/alpine").await;
            assert!(res.is_ok());
            if let Ok(res) = res {
                assert!(res)
            }
        }

        #[tokio::test]
        async fn check_image_slug_returns_false_on_invalid_slug() {
            let res = check_image_slug("crane", "docker.io/non-existent").await;
            println!("{:?}", res);
            assert!(res.is_ok());
            if let Ok(res) = res {
                assert!(!res)
            }
        }

        #[tokio::test]
        async fn check_image_slug_returns_error_on_failed_spawn() {
            let res = check_image_slug("non-existent", "docker.io/alpine").await;
            assert!(res.is_err());
        }
    }
}

mod cli {
    /// CLI config
    use clap::Parser;

    #[derive(Parser)]
    #[command(author, version, about, long_about)]
    /// This webserver serves an API to check whether a container image is present
    /// in a registry or not. Currently, it only allows to query public registries
    /// (no authentication implemented) and serves only http (no encription).
    ///
    /// To query for the image `docker.io/nginx`, run
    ///
    /// curl "http://localhost:8080/exists?image=docker.io/nginx"
    pub struct Args {
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

    impl Args {
        pub fn ip(&self) -> std::net::IpAddr {
            self.ip
        }
        pub fn port(&self) -> u16 {
            self.port
        }
        pub fn crane_cmd(&self) -> String {
            self.crane_cmd.clone()
        }
    }

    /// Parse CLI args
    pub fn parse_args() -> Args {
        Args::parse()
    }
}

mod doc {
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(crate::filter::health, crate::filter::check_image))]
    pub struct ApiDoc;
}
