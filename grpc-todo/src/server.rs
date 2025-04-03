use crate::{create_todo, db_init, delete_todo, get_todos, update_todo};
use anyhow::Result;
use sqlx::PgPool;
use todo::{
    CreateTodoRequest, DeleteTodoRequest, Empty, UpdateTodoRequest,
    todo_service_server::{TodoService, TodoServiceServer},
};
use tonic::{Request, Response, Status, transport::Server};
use tower_http::cors::{Any, CorsLayer};

mod todo {
    tonic::include_proto!("todo");
}

#[derive(Debug)]
pub struct MyTodoService {
    pool: PgPool,
}

impl From<crate::Todo> for todo::Todo {
    fn from(value: crate::Todo) -> Self {
        todo::Todo {
            id: Some(value.id),
            title: value.title,
            completed: value.completed,
        }
    }
}

#[tonic::async_trait]
impl TodoService for MyTodoService {
    async fn create_todo(
        &self,
        request: Request<CreateTodoRequest>,
    ) -> Result<Response<todo::Todo>, Status> {
        let req = request.into_inner();
        let todo = create_todo(&self.pool, &req.title).await;
        Ok(Response::new(todo.into()))
    }

    async fn get_todos(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<todo::TodoList>, Status> {
        let todos = get_todos(&self.pool).await;
        Ok(Response::new(todo::TodoList {
            todos: todos.into_iter().map(|t| t.into()).collect(),
        }))
    }

    async fn delete_todo(
        &self,
        request: Request<DeleteTodoRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        delete_todo(&self.pool, req.id.unwrap()).await;
        Ok(Response::new(Empty {}))
    }

    async fn update_todo(
        &self,
        request: Request<UpdateTodoRequest>,
    ) -> Result<Response<todo::Todo>, Status> {
        let req = request.into_inner();
        let todo = update_todo(&self.pool, req.id.unwrap(), &req.title, req.completed).await;
        Ok(Response::new(todo.into()))
    }
}

pub async fn run_server() -> Result<()> {
    let addr = "[::]:50051".parse().unwrap();
    println!("Server {} listening...", addr);
    let pool = db_init().await?;

    let service = MyTodoService { pool };

    let cors = CorsLayer::new().allow_origin(Any);

    let grpc_web_service = TodoServiceServer::new(service);

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(grpc_web_service)
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}
