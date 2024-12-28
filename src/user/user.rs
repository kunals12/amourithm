use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::PgPool;
use strum_macros::Display;

use crate::{auth::jwt::validate_token, common::ResponseToSend};

#[derive(Display, Deserialize)]
enum Gender {
    Male,
    Female,
    Other,
}

pub struct User {
    firstname: String,
    lastname: String,
    age: u8,
    gender: Gender,
    bio: String,
    profile_url: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    firstname: Option<String>,
    lastname: Option<String>,
    age: Option<u8>,
    gender: Option<Gender>,
    bio: Option<String>,
    profile_url: Option<String>,
}

impl User {
    pub async fn update_user_details(
        db: Data<PgPool>,
        req: HttpRequest,
        body: Json<UpdateUserRequest>,
    ) -> impl Responder {
        match validate_token(req).await {
            Ok(user_id) => {
                println!("{}", user_id);
                let mut field_to_update: Vec<String> = Vec::new();

                if let Some(firstname) = &body.firstname {
                    field_to_update.push(format!("firstname = '{}'", firstname));
                }
                if let Some(lastname) = &body.lastname {
                    field_to_update.push(format!("lastname = '{}'", lastname));
                }
                if let Some(age) = body.age {
                    field_to_update.push(format!("age = {}", age));
                }
                if let Some(gender) = &body.gender {
                    match gender {
                        Gender::Male => {
                            field_to_update.push(format!("gender = '{}'", Gender::Male));
                        }
                        Gender::Female => {
                            field_to_update.push(format!("gender = '{}'", Gender::Female));
                        }
                        Gender::Other => {
                            field_to_update.push(format!("gender = '{}'", Gender::Other));
                        } // _ => {}
                    }
                }
                if let Some(bio) = &body.bio {
                    field_to_update.push(format!("bio = '{}'", bio));
                }

                // Construct the SQL query
                let query = format!(
                    "UPDATE usersdata SET {} WHERE id = $1",
                    field_to_update.join(", ")
                );

                println!("{}", query);
                // Execute the query with the ID parameter
                let response = sqlx::query(&query).bind(user_id).execute(&**db).await;

                match response {
                    Ok(_) => HttpResponse::Ok().json(ResponseToSend::<()> {
                        success: true,
                        message: "User Updated".to_string(),
                        data: None,
                    }),
                    Err(e) => HttpResponse::InternalServerError().json(ResponseToSend::<()> {
                        success: false,
                        message: e.to_string(),
                        data: None,
                    }),
                }
            }
            Err(e) => e,
        }
    }
}
