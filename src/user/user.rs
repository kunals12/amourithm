use std::sync::Arc;

use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, prelude::FromRow, Error, PgPool, Result};
use strum_macros::Display;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    auth::jwt::validate_token,
    common::{
        handle_bad_request, handle_internal_server_error, handle_not_found_error, ResponseToSend,
    },
};

#[derive(sqlx::Type, Debug, Deserialize, Display, Serialize)]
#[sqlx(type_name = "VARCHAR")]
enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Deserialize, Debug, FromRow, Serialize)]
pub struct User {
    firstname: Option<String>,
    lastname: Option<String>,
    age: Option<i32>,
    gender: Option<Gender>,
    bio: Option<String>,
    profile_picture_url: Option<String>,
}

impl User {
    async fn get_user_basic_data(
        db: Data<PgPool>,
        redis: Data<Arc<Mutex<MultiplexedConnection>>>,
        user_id: Uuid,
    ) -> Option<User> {
        let mut redis_conn = redis.lock().await;

        let redis_key = format!("user_data:{}", user_id); // Use a unique key
        let get_user_data_from_redis: Result<Option<String>, redis::RedisError> =
            redis_conn.get(redis_key.clone()).await;

        match get_user_data_from_redis {
            Ok(Some(redis_user_data)) => {
                // println!("Returning from redis");
                // Deserialize the cached data from Redis
                match serde_json::from_str::<User>(&redis_user_data) {
                    Ok(user_data) => Some(user_data),
                    Err(_) => None, // If deserialization fails, return None
                }
            }
            Ok(None) => {
                let user_data = sqlx::query_as::<_, User>(
                    "SELECT firstname, lastname, age, gender, bio, profile_picture_url FROM usersdata WHERE user_id = $1",
                )
                .bind(user_id) // `id` should be of the correct type (likely `Uuid`)
                .fetch_one(&**db)
                .await;

                match user_data {
                    Ok(data) => {
                        // println!("Setting in redis");

                        // Serialize the user data to store in Redis
                        let serialized_data = match serde_json::to_string(&data) {
                            Ok(json_data) => json_data,
                            Err(_) => return None, // Return None if serialization fails
                        };

                        // Store the data in Redis with an expiration time (1 hour)
                        let _: () = redis_conn
                            .set_ex(&redis_key, serialized_data, 3600)
                            .await
                            .unwrap();

                        Some(data)
                    }
                    Err(_) => None,
                }
            }
            Err(e) => {
                // Handle Redis error if any
                println!("Error fetching from Redis: {}", e);
                None
            }
        }
    }

    // Get User
    pub async fn get_user(
        db: Data<PgPool>,
        redis: Data<Arc<Mutex<MultiplexedConnection>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match validate_token(req).await {
            Ok(user_id) => {
                let user_data = Self::get_user_basic_data(db.clone(), redis.clone(), user_id).await;

                if let Some(data) = user_data {
                    HttpResponse::Ok().json(ResponseToSend {
                        success: true,
                        message: "Token validated successfully".to_string(),
                        data: Some(data),
                    })
                } else {
                    return handle_not_found_error("User Data Not Found");
                }
            }
            Err(err) => err,
        }
    }

    pub async fn insert_user_data(
        db: Data<PgPool>,
        req: HttpRequest,
        user: Json<User>,
    ) -> impl Responder {
        match validate_token(req).await {
            Ok(user_id) => {
                let mut response: Result<PgQueryResult, Error> = Err(sqlx::Error::RowNotFound); // Initialize with a default error or a valid result type

                // Check Firstname
                if let Some(firstname) = &user.firstname {
                    let is_user_data_exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM usersdata WHERE user_id = $1)",
                    )
                    .bind(user_id)
                    .fetch_one(&**db)
                    .await
                    // .map_err(|_| HttpResponse::InternalServerError().finish())?
                    .unwrap_or(false);

                    if !is_user_data_exists {
                        println!("creating user");
                        let usersdata_id = Uuid::new_v4();

                        response = sqlx::query(
                            "INSERT INTO usersdata (id, firstname, user_id) VALUES ($1, $2, $3)",
                        )
                        .bind(usersdata_id)
                        .bind(firstname.clone())
                        .bind(user_id)
                        .execute(&**db)
                        .await;
                    } else {
                        println!("updating user");

                        response = sqlx::query(
                            "UPDATE usersdata SET firstname = $1, updated_at = CURRENT_TIMESTAMP WHERE user_id = $2",
                        )
                            .bind(firstname.clone())
                            .bind(user_id)
                            .execute(&**db)
                            .await;
                    }
                }

                // Update Lastname
                if let Some(lastname) = &user.lastname {
                    response = sqlx::query(
                        "UPDATE usersdata SET lastname = $1, updated_at = CURRENT_TIMESTAMP WHERE user_id = $2",
                    )
                        .bind(lastname.clone())
                        .bind(user_id)
                        .execute(&**db)
                        .await;
                }

                // Update Age
                if let Some(user_age) = user.age {
                    const MINIMUM_AGE: i32 = 18;
                    const MAXIMUM_AGE: i32 = 50;

                    // Validate age
                    if user_age < MINIMUM_AGE {
                        return handle_bad_request("Age must be greater than 18");
                    } else if user_age > MAXIMUM_AGE {
                        return handle_bad_request("Age must be less than 50");
                    } else {
                        // println!("User age type: {:?}", std::any::type_name::<i8>());

                        // Update age in the database
                        response = sqlx::query(
                                            "UPDATE usersdata SET age = $1, updated_at = CURRENT_TIMESTAMP WHERE user_id = $2",
                                        )
                                        .bind(user_age)
                                        .bind(user_id)
                                        .execute(&**db)
                                        .await;
                    }
                }

                // Update Gender
                if let Some(gender) = &user.gender {
                    let gender_str = match gender {
                        Gender::Male => "Male",
                        Gender::Female => "Female",
                        Gender::Other => "Other",
                    };

                    // Update the gender in the database
                    response = sqlx::query(
                                        "UPDATE usersdata SET gender = $1, updated_at = CURRENT_TIMESTAMP WHERE user_id = $2",
                                    )
                                    .bind(gender_str)  // Binding the gender string
                                    .bind(user_id)
                                    .execute(&**db)
                                    .await;
                }

                match response {
                    Ok(_) => HttpResponse::Ok().json(ResponseToSend::<()> {
                        success: true,
                        message: "User Data Updated Successfully".to_string(),
                        data: None,
                    }),
                    Err(e) => handle_internal_server_error(&e.to_string()),
                }
            }
            Err(e) => e,
        }
    }
}
