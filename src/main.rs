#[macro_use] extern crate rocket;
use rocket::http::{Cookie, CookieJar, Method};
use rocket::State;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl, Scope, TokenResponse};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use mongodb::Client;
use serde_json::Value;
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
};
use rocket_cors::{AllowedOrigins, AllowedHeaders, CorsOptions, Cors, Error};
use rocket::serde::json::Json;
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    picture: String,
}

struct MongoDbClient {
    client: Client,
}
struct OAuth2Client {
    client: BasicClient,
}

const CLIENT_ID: &str = "882522453771-s8vtldcld9kfm22jo5votsve7sb1recb.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "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";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
const MONGO_DB_URI: &str = "mongodb+srv://afaqkarim99:Bn4wz2NYMSzQr9Xj@stripe-app.e73tt6s.mongodb.net/";

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/auth/google")]
fn google_login() -> Json<Value> {
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        Some(ClientSecret::new(CLIENT_SECRET.to_string())),
        AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization URL"),
        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token URL")),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080/auth/google/callback".to_string()).expect("Invalid redirect URL"));

    let authorization_request = client.authorize_url(|| CsrfToken::new_random())
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()));

    let (auth_url, _csrf_token) = authorization_request.url();

    Json(json!({"url": auth_url.to_string()}))
}

#[get("/auth/google/callback?<code>")]
async fn google_callback(code: String, jar: &CookieJar<'_>, oauth_client: &State<OAuth2Client>, db_client: &State<MongoDbClient>) -> Json<Value> {
    let token_result = oauth_client.client.exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client).await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret();
            let user_info_response = reqwest::Client::new()
                .get("https://www.googleapis.com/oauth2/v1/userinfo")
                .bearer_auth(access_token)
                .send()
                .await;

            match user_info_response {
                Ok(response) => {
                    if response.status().is_success() {
                        let user_info: Result<Value, _> = response.json().await;
                        match user_info {
                            Ok(user_info) => {
                                let user_id = user_info["id"].as_str().unwrap_or_default().to_string();

                                let user_doc = doc! {
                                    "id": &user_id,
                                    "email": user_info["email"].as_str().unwrap_or_default(),
                                    "name": user_info["name"].as_str().unwrap_or_default(),
                                    "picture": user_info.get("picture").and_then(|v| v.as_str()).map(String::from),
                                };

                                let database = db_client.client.database("rust-backend");
                                let collection = database.collection::<Document>("users");
                                if let Ok(_) = collection.update_one(
                                    doc! { "id": &user_id }, 
                                    doc! { "$set": user_doc }, 
                                    UpdateOptions::builder().upsert(true).build(),
                                ).await {
                                    jar.add_private(Cookie::new("user_id", user_id));
                                    Json(json!({"status": "success", "data": user_info}))
                                } else {
                                    Json(json!({"status": "error", "message": "Failed to update user in MongoDB"}))
                                }
                            },
                            Err(_) => Json(json!({"status": "error", "message": "Failed to parse user info"}))
                        }
                    } else {
                        Json(json!({"status": "error", "message": "Failed to fetch user info"}))
                    }
                },
                Err(_) => Json(json!({"status": "error", "message": "Failed to connect to user info API"}))
            }
        },
        Err(_) => Json(json!({"status": "error", "message": "Failed to authenticate with Google"}))
    }
}

#[launch]
async fn rocket() -> _ {
    let mongo_client = Client::with_uri_str(MONGO_DB_URI).await.expect("Failed to connect to MongoDB");
    let oauth_client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        Some(ClientSecret::new(CLIENT_SECRET.to_string())),
        AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization URL"),
        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token URL")),
    ).set_redirect_uri(RedirectUrl::new("http://localhost:8080/auth/google/callback".to_string()).expect("Invalid redirect URL"));

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&["http://localhost:8080"]))
        .allowed_methods(vec![Method::Get, Method::Post, Method::Options].into_iter().map(rocket_cors::Method::from).collect())
        .allowed_headers(AllowedHeaders::some(&["Content-Type", "Authorization"]))
        .allow_credentials(true)
        .to_cors()
        .expect("error creating CORS fairing");

    rocket::build()
        .attach(cors)
        .manage(MongoDbClient { client: mongo_client })
        .manage(OAuth2Client { client: oauth_client })
        .mount("/", routes![index, google_login, google_callback])
}
