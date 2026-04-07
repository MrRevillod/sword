use std::pin::Pin;
use std::sync::Arc;
use sword::grpc::*;
use sword::prelude::*;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    shared::AuthInterceptor,
    users::{CreateUserDto, UpdateUserDto, UserRepository, proto::*},
};

#[controller(kind = Controller::Grpc, service = UserServiceServer)]
#[interceptor(AuthInterceptor)]
pub struct UsersController {
    users: Arc<UserRepository>,
}

#[sword::grpc::async_trait]
impl UserService for UsersController {
    type StreamUsersStream = Pin<Box<dyn Stream<Item = Result<UserItem, Status>> + Send>>;

    async fn list_users(&self, _req: Request<ListUsersRequest>) -> GrpcResult<ListUsersReply> {
        let users = self.users.find_all().await;

        let users = users
            .into_iter()
            .map(|u| UserItem {
                id: u.id,
                username: u.username,
            })
            .collect();

        Ok(Response::new(ListUsersReply { users }))
    }

    async fn stream_users(
        &self,
        _req: Request<StreamUsersRequest>,
    ) -> GrpcResult<Self::StreamUsersStream> {
        let users = self.users.find_all().await;

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            for user in users {
                let item = UserItem {
                    id: user.id,
                    username: user.username,
                };

                if tx.send(Ok(item)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::StreamUsersStream
        ))
    }

    async fn create_user(&self, req: Request<CreateUserRequest>) -> GrpcResult<UserReply> {
        let payload = req.into_inner();

        let dto = CreateUserDto {
            username: payload.username,
            password: payload.password,
        };

        let user = self.users.create(dto).await?;

        Ok(Response::new(UserReply {
            user: Some(UserItem {
                id: user.id,
                username: user.username,
            }),
        }))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> GrpcResult<UserReply> {
        let id = req.into_inner().id;
        let user = self.users.find_by_id(&id).await?;

        Ok(Response::new(UserReply {
            user: Some(UserItem {
                id: user.id,
                username: user.username,
            }),
        }))
    }

    async fn update_user(&self, req: Request<UpdateUserRequest>) -> GrpcResult<UserReply> {
        let payload = req.into_inner();

        let dto = UpdateUserDto {
            id: payload.id,
            username: payload.username,
            password: payload.password,
        };

        let user = self.users.update(dto).await?;

        Ok(Response::new(UserReply {
            user: Some(UserItem {
                id: user.id,
                username: user.username,
            }),
        }))
    }

    async fn delete_user(&self, req: Request<DeleteUserRequest>) -> GrpcResult<DeleteUserReply> {
        let id = req.into_inner().id;

        self.users.delete(&id).await?;

        Ok(Response::new(DeleteUserReply {
            message: "User deleted".to_string(),
        }))
    }
}
