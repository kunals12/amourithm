use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, prelude::FromRow, Error, PgPool, Result};
use strum_macros::Display;
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

impl std::str::FromStr for Gender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "other" => Ok(Gender::Other),
            _ => Err(format!("Invalid gender value: {}", s)),
        }
    }
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

// #[derive(Deserialize)]
// pub struct UpdateUserRequest {
//     firstname: Option<String>,
//     lastname: Option<String>,
//     // age: Option<u8>,
//     gender: Option<Gender>,
//     bio: Option<String>,
//     profile_url: Option<String>,
// }

impl User {
    async fn get_user_basic_data(db: Data<PgPool>, user_id: Uuid) -> Option<User> {
        let user_data = sqlx::query_as::<_, User>(
            "SELECT firstname, lastname, age, gender, bio, profile_picture_url FROM usersdata WHERE user_id = $1",
        )
        .bind(user_id) // `id` should be of the correct type (likely `Uuid`)
        .fetch_one(&**db)
        .await;

        match user_data {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    // Get User
    pub async fn get_user(db: Data<PgPool>, req: HttpRequest) -> impl Responder {
        match validate_token(req).await {
            Ok(user_id) => {
                let user_data = User::get_user_basic_data(db.clone(), user_id).await;

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

    // pub async fn update_user_details(
    //     db: Data<PgPool>,
    //     req: HttpRequest,
    //     body: Json<UpdateUserRequest>,
    // ) -> impl Responder {
    //     match validate_token(req).await {
    //         Ok(user_id) => {
    //             let mut fields: HashMap<&str, String> = HashMap::new();
    //             let mut query = String::from("UPDATE usersdata SET ");

    //             if let Some(firstname) = &body.firstname {
    //                 fields.insert("firstname", firstname.into());
    //             }
    //             if let Some(lastname) = &body.lastname {
    //                 fields.insert("lastname", lastname.into());
    //             }
    //             // if let Some(age) = body.age {
    //             //     fields.insert("age", age.into());
    //             // }
    //             if let Some(gender) = &body.gender {
    //                 fields.insert("gender", gender.to_string().into());
    //             }
    //             if let Some(bio) = &body.bio {
    //                 fields.insert("bio", bio.into());
    //             }

    //             if fields.is_empty() {
    //                 return HttpResponse::BadRequest().json(ResponseToSend::<()> {
    //                     success: false,
    //                     message: "No fields provided to update".to_string(),
    //                     data: None,
    //                 });
    //             }

    //             let mut set_clauses = Vec::new();
    //             let mut bind_values = Vec::new();

    //             for (key, value) in fields {
    //                 set_clauses.push(format!("{} = ${}", key, bind_values.len() + 1));
    //                 bind_values.push(value);
    //             }

    //             let query = format!(
    //                 "UPDATE usersdata SET {} WHERE id = ${}",
    //                 set_clauses.join(", "),
    //                 bind_values.len() + 1
    //             );

    //             let mut sql_query = sqlx::query(&query);

    //             for value in bind_values {
    //                 sql_query = sql_query.bind(value);
    //             }

    //             sql_query = sql_query.bind(user_id);

    //             match sql_query.execute(&**db).await {
    //                 Ok(_) => HttpResponse::Ok().json(ResponseToSend::<()> {
    //                     success: true,
    //                     message: "User details updated successfully".to_string(),
    //                     data: None,
    //                 }),
    //                 Err(e) => HttpResponse::InternalServerError().json(ResponseToSend::<()> {
    //                     success: false,
    //                     message: format!("Failed to update user details: {}", e),
    //                     data: None,
    //                 }),
    //             }
    //         }
    //         Err(e) => e,
    //     }
    // }
}
