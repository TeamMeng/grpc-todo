use serde::Deserialize;
use std::collections::HashMap;
use todo::{
    CreateTodoRequest, DeleteTodoRequest, Todo, UpdateTodoRequest,
    todo_service_client::TodoServiceClient,
};
use tonic::transport::Channel;
use warp::{Filter, reject::Rejection, reply::Reply};

mod todo {
    tonic::include_proto!("todo");
}

#[derive(Debug, Clone)]
struct AppState {
    todo_client: TodoServiceClient<Channel>,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    #[serde(flatten)]
    extra: HashMap<String, String>,
    #[serde(default)]
    completed: String,
}

#[tokio::main]
async fn main() {
    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();

    let todo_client = TodoServiceClient::new(channel);

    let state = AppState { todo_client };
    let state_filter = warp::any().map(move || state.clone());

    // Routes
    let index = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(include_str!("index.html")));

    let get_todos = warp::path!("todos")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(get_todos_handler);

    let create_todo = warp::path!("todos")
        .and(warp::post())
        .and(warp::body::form())
        .and(state_filter.clone())
        .and_then(create_todo_handler);

    let update_todo = warp::path!("todos" / i64)
        .and(warp::put())
        .and(warp::body::form())
        .and(state_filter.clone())
        .and_then(update_todo_handler);

    let delete_todo = warp::path!("todos" / i64)
        .and(warp::delete())
        .and(state_filter.clone())
        .and_then(delete_todo_handler);

    let routes = index
        .or(get_todos)
        .or(create_todo)
        .or(update_todo)
        .or(delete_todo)
        .with(warp::cors().allow_any_origin());

    println!("Server listening on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn get_todos_handler(state: AppState) -> Result<impl Reply, Rejection> {
    let mut client = state.todo_client;
    let request = tonic::Request::new(todo::Empty {});
    let response = client.get_todos(request).await.map_err(|e| {
        eprintln!("Error calling get todos: {:?}", e);
        warp::reject::not_found()
    })?;

    let todos = response.into_inner().todos;

    Ok(warp::reply::html(render_todo_list(&todos)))
}

async fn create_todo_handler(form: CreateTodo, state: AppState) -> Result<impl Reply, Rejection> {
    let mut client = state.todo_client;
    let request = tonic::Request::new(CreateTodoRequest { title: form.title });

    let response = client.create_todo(request).await.map_err(|e| {
        eprintln!("Error calling create todos: {:?}", e);
        warp::reject::not_found()
    })?;

    let todo = response.into_inner();

    Ok(warp::reply::html(render_todo_item(&todo)))
}

async fn update_todo_handler(
    id: i64,
    form: UpdateTodo,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    let mut client = state.todo_client;

    let title_key = format!("title-{}", id);
    let title = form.extra.get(&title_key).cloned().unwrap_or_default();

    let completed = matches!(form.completed.to_lowercase().as_str(), "true" | "on");

    let request = tonic::Request::new(UpdateTodoRequest {
        id: Some(id),
        title,
        completed,
    });

    let response = client.update_todo(request).await.map_err(|e| {
        eprintln!("Error calling update_todo: {:?}", e);
        warp::reject::not_found()
    })?;

    let todo = response.into_inner();
    Ok(warp::reply::html(render_todo_item(&todo)))
}

async fn delete_todo_handler(id: i64, state: AppState) -> Result<impl Reply, Rejection> {
    let mut client = state.todo_client;
    let request = tonic::Request::new(DeleteTodoRequest { id: Some(id) });

    client.delete_todo(request).await.map_err(|e| {
        eprintln!("Error calling delete_todo: {:?}", e);
        warp::reject::not_found()
    })?;

    Ok(warp::reply::html(""))
}

fn render_todo_list(todos: &[Todo]) -> String {
    let mut html = String::new();
    for todo in todos {
        html.push_str(&render_todo_item(todo));
    }
    html
}

fn render_todo_item(todo: &Todo) -> String {
    format!(
        r#"
    <div class="todo-item" id="todo-{}" hx-target="this" hx-swap="outerHTML">
        <input type="checkbox"
            hx-put="/todos/{}"
            hx-include="[name='title-{}']"
            name="completed"
            {} />
        <span class="{}">{}</span>
        <input type="hidden" name="title-{}" value="{}" />
        <button hx-delete="/todos/{}">Delete</button>
    </div>
    "#,
        todo.id.unwrap_or(0),
        todo.id.unwrap_or(0),
        todo.id.unwrap_or(0),
        if todo.completed { "checked" } else { "" },
        if todo.completed { "completed" } else { "" },
        todo.title,
        todo.id.unwrap_or(0),
        todo.title,
        todo.id.unwrap_or(0),
    )
}
