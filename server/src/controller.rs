use deadpool_diesel::sqlite::Pool;
use http_body_util::BodyExt;
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use shared::rest_dtos::{DeviceCreateDTO, DeviceDeleteDTO, EnrollDeviceDTO};

use crate::repository::{device_create, device_delete, enroll_device_into_environment, user_create};

pub fn json_response(status_code: StatusCode, msg: String, data: serde_json::Value) -> Response<http_body_util::Full<tokio_tungstenite::tungstenite::Bytes>> {
    let mut payload = serde_json::json!({});

    payload["msg"] = msg.into();
    payload["data"] = data.clone();

    let bytes: tokio_tungstenite::tungstenite::Bytes = payload.to_string().into();

    return Response::builder()
        .header("content-type", "application/json")
        .status(status_code)
        .body(http_body_util::Full::from(bytes))
        .expect("Failed to build response");
}

pub async fn handle_http_connection(req: &mut Request<Incoming>, database_pool: Pool) -> Response<http_body_util::Full<tokio_tungstenite::tungstenite::Bytes>> { 
    let (uri, method) = (req.uri().clone().to_string(), req.method().clone());

    let body: Vec<u8> = req.collect()
        .await
        .unwrap()
        .to_bytes()
        .into_iter()
        .collect();

    match (uri.as_str(), method) {
        ("/device", Method::POST) => {
            let json: DeviceCreateDTO = serde_json::from_slice(body.as_slice()).unwrap();

            let result = device_create(json.name, json.machine_id, database_pool).await;

            println!("device create result {result:?}");
            
            return json_response(
                StatusCode::OK,
                String::from("Created device successfully"),
                serde_json::Value::Null,
            )
        },
        ("/device", Method::DELETE) => {
            let json: DeviceDeleteDTO = serde_json::from_slice(body.as_slice()).unwrap();
            
            let result = device_delete(json.name, database_pool).await;

            return json_response(
                StatusCode::OK,
                String::from("Deleted device successfully"),
                serde_json::Value::Null,
            )
        },
        ("/user", Method::POST) => {
            // TODO: implement
            return json_response(
                StatusCode::OK,
                String::from("Created user successfully"),
                serde_json::Value::Null,
            )
        },
        ("/user", Method::DELETE) => {
            // TODO: implement
            return json_response(
                StatusCode::OK,
                String::from("Deleted user successfully"),
                serde_json::Value::Null,
            )
        },
        ("/enroll_device", Method::POST) => {
            let json: EnrollDeviceDTO = serde_json::from_slice(body.as_slice()).unwrap();

            let result = enroll_device_into_environment(
                json.machine_id,
                json.project_name,
                json.environment_name,
                database_pool
            ).await;

            return if let Err(err) = result {
                json_response(
                   StatusCode::INTERNAL_SERVER_ERROR,
                   String::from(err.to_string()),
                   serde_json::Value::Null,
                )
            } else {
                json_response(
                   StatusCode::OK,
                   String::from("Device enrolled successfully"),
                   serde_json::Value::Null,
                )
            }
        },
        _ => {
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::default(),
                serde_json::Value::Null,
            )
        }
    };
}
