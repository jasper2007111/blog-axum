# Blog Axum版
这个主要是为了配合Yew的UI组件演示的后端项目，一开始我对Rocket比较感兴趣，但那个项目目前来看前途不是很明朗。Actix Web是另个选择，这个项目似乎很稳定，而且至少手头的两本书中都有讲这个框架的例子。不过最后选择了Axum，这个框架是tokio官方出的，前者都会依靠tokio这个异步运行时，Axum另外就是隐约的感受到大家对这个呼声很高，而且这个是搭配着一些列的配套设施的，虽然选择还没有到1.0，另外我上次研究Trunk这个打包框架时，有看到它内部也使用的是Axum，实际上不管怎样选择，都是我使用的第一个Rust后端项目，那就选Axum这个试试看。

### 坑

在加认证的中间件时，出现了这个错误。

```rust
use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
    response::{IntoResponse, Response},
    middleware::{self, Next},
    extract::State,
};

#[derive(Clone)]
struct AppState { /* ... */ }

async fn my_middleware<B>(
    State(state): State<AppState>,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // do something with `request`...

    let response = next.run(request).await;

    // do something with `response`...

    response
}

let state = AppState { /* ... */ };

let app = Router::new()
    .route("/", get(|| async { /* ... */ }))
    .route_layer(middleware::from_fn_with_state(state.clone(), my_middleware))
    .with_state(state);

```

一开始我找的一个代码出错了，于是我就找了上面这个官方文档的事例，但一直报下面的错误。

```sh
error[E0277]: the trait bound `axum::middleware::FromFn<fn(axum::extract::State<AppState>, Request<_>, Next<_>) -> impl Future<Output = Response<http_body::combinators::box_body::UnsyncBoxBody<axum::body::Bytes, axum::Error>>> {my_middleware::<_>}, Arc<AppState>, Route<_>, _>: tower_service::Service<Request<_>>` is not satisfied
   --> src/main.rs:103:18
    |
103 |     .route_layer(middleware::from_fn_with_state(state.clone(), my_middleware))
    |      ----------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `tower_service::Service<Request<_>>` is not implemented for `axum::middleware::FromFn<fn(axum::extract::State<AppState>, Request<_>, Next<_>) -> impl Future<Output = Response<http_body::combinators::box_body::UnsyncBoxBody<axum::body::Bytes, axum::Error>>> {my_middleware::<_>}, Arc<AppState>, Route<_>, _>`
    |      |
    |      required by a bound introduced by this call
    |
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S, B>::route_layer`
   --> /Users/jasperji/.cargo/registry/src/github.com-1ecc6299db9ec823/axum-0.6.20/src/routing/mod.rs:255:21
    |
255 |         L::Service: Service<Request<B>> + Clone + Send + 'static,
    |                     ^^^^^^^^^^^^^^^^^^^ required by this bound in `Router::<S, B>::route_layer`

For more information about this error, try `rustc --explain E0277`.
warning: `blog_axum` (bin "blog_axum") generated 4 warnings

```

这个猛一看，感觉毫无头绪，因为一开始的时候我就遇到一个错误，不过那个是函数参数顺序发生了变化的问题，但这个仔细研究后发现好像也不是，于是我又把范型和特性的章节给看了下，也没有发现问题所在。最后我试着尝试`middleware::from_fn`的官方例子，发现并没有问题。然后再回到`middleware::from_fn_with_state`，再仔细一看最后发现了问题所在，官方的例子没有错，而是我一开始没有修改`app_state`这个声明的问题，原来的是Arc类型，而官方的例子就是AppState这个类型。这时再看Rust的编译器提示的错误，你会发现很难一眼看出问题`{my_middleware::<_>}, Arc<AppState>, Route<_>, _>`，如果不是仔细的看，或许根本想不到这个问题所在。这也告诫我们Rust再实际编程中难度，这类问题其实不只出现一次，之前也曾有过类似的问题，希望以后再遇到这类问题能够冷静的思考下。


