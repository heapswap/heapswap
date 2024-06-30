//use heapswap_protos::hello;
use heapswap_core::{
    bys,
    messages::{Action, Field, Request, Response},
    traits::*,
    u256::*,
};
use poem::{self, http::StatusCode, IntoResponse, Response as PoemResponse};
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Json, Path},
    Route, Server,
};

#[derive(Debug)]
pub enum HandlerError {
    InvalidBase32,
    InvalidLength,
    InvalidSignerBase32,
    InvalidSignerLength,
    InvalidCosignerBase32,
    InvalidCosignerLength,
    InvalidTangentBase32,
    InvalidTangentLength,
}

fn validate_field(input: &str) -> Result<Option<U256>, HandlerError> {
    if input == "_" {
        return Ok(None);
    }

    let input = bys::from_base32(&input).map_err(|_| HandlerError::InvalidBase32)?;
    if input.len() != 32 {
        return Err(HandlerError::InvalidLength);
    }

    return Ok(Some(U256::from_bytes(&input).unwrap()));
}

fn validate_fields(signer: &str, cosigner: &str, tangent: &str) -> Result<Field, HandlerError> {
    let field = Field::new(
        validate_field(signer)?,
        validate_field(cosigner)?,
        validate_field(tangent)?,
    );

    Ok(field)
}

#[handler]
fn main_get_handler(
    Path((signer, cosigner, tangent)): Path<(String, String, String)>,
) -> Json<Field> {
    let field = validate_fields(signer.as_str(), cosigner.as_str(), tangent.as_str()).unwrap();
    Json(field)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/:signer/:cosigner/:tangent", get(main_get_handler));

    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let address = std::env::var("ADDRESS").unwrap_or("0.0.0.0".to_string());
    let listening_address = format!("{}:{}", address, port);
    let localhost_address = format!("http://localhost:{}", port);

    println!("Listening on {}", localhost_address);
    Server::new(TcpListener::bind(listening_address))
        .run(app)
        .await
}
