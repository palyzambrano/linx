use async_graphql::{Context, InputObject, Result, SimpleObject};
use migration::DbErr;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

use crate::context::SharedContext;
use crate::graphql::user::{User, UserError, UserErrorCode};

#[derive(Debug, Default, InputObject)]
pub struct UserCreateInput {
    pub name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Default, Deserialize, Serialize, SimpleObject)]
pub struct UserCreate {
    user: Option<User>,
    error: Option<UserError>,
}

impl UserCreate {
    pub async fn exec(ctx: &Context<'_>, input: UserCreateInput) -> Result<Self> {
        let context = ctx.data_unchecked::<SharedContext>();
        let hash = context.services.auth.hash_password(&input.password)?;
        let user = entity::user::ActiveModel {
            name: Set(input.name),
            last_name: Set(input.last_name),
            email: Set(input.email),
            hash: Set(hash),
            ..Default::default()
        };

        match user.save(&context.conn()).await {
            Ok(user) => Ok(Self {
                user: Some(User {
                    id: user.id.unwrap(),
                    name: user.name.unwrap(),
                    last_name: user.last_name.unwrap(),
                    email: user.email.unwrap(),
                }),
                ..Default::default()
            }),
            Err(db_error) => {
                if matches!(db_error, DbErr::Query(_)) {
                    return Ok(Self {
                        error: Some(UserError {
                            code: UserErrorCode::EmailTaken,
                            message: db_error.to_string(),
                        }),
                        ..Default::default()
                    });
                }

                Ok(Self {
                    error: Some(UserError {
                        code: UserErrorCode::Unknown,
                        message: db_error.to_string(),
                    }),
                    ..Default::default()
                })
            }
        }
    }
}
