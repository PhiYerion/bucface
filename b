    Checking bucface_client v0.1.0 (/home/user/projects/bucface/bucface_client)
warning: unused imports: `AtomicUsize`, `Ordering`
 --> bucface_client/src/app.rs:2:25
  |
2 | use std::sync::atomic::{AtomicUsize, Ordering};
  |                         ^^^^^^^^^^^  ^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::sync::Arc`
 --> bucface_client/src/app.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused import: `Events`
 --> bucface_client/src/app.rs:5:28
  |
5 | use bucface_utils::{Event, Events};
  |                            ^^^^^^

warning: unused import: `rmp_serde::Serializer`
 --> bucface_client/src/app.rs:7:5
  |
7 | use rmp_serde::Serializer;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde::Serialize`
 --> bucface_client/src/app.rs:8:5
  |
8 | use serde::Serialize;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `tokio::sync::Mutex`
 --> bucface_client/src/app.rs:9:5
  |
9 | use tokio::sync::Mutex;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused imports: `BorrowMut`, `Borrow`
  --> bucface_client/src/main.rs:14:19
   |
14 | use std::borrow::{Borrow, BorrowMut};
   |                   ^^^^^^  ^^^^^^^^^

warning: unused import: `tokio::sync::Mutex`
  --> bucface_client/src/main.rs:17:5
   |
17 | use tokio::sync::Mutex;
   |     ^^^^^^^^^^^^^^^^^^

error[E0521]: borrowed data escapes outside of method
  --> bucface_client/src/app.rs:91:9
   |
90 |       async fn update_logs(&self) -> tokio::task::JoinHandle<impl Future> {
   |                            -----
   |                            |
   |                            `self` is a reference that is only valid in the method body
   |                            has type `&app::App<'1>`
91 | /         tokio::spawn(async move {
92 | |             get_events(
93 | |                 "http://127.0.0.1:8080/events".to_string(),
94 | |                 self.client.clone(),
95 | |                 self.events.len(),
96 | |             )
97 | |         })
   | |          ^
   | |          |
   | |__________`self` escapes the method body here
   |            argument requires that `'1` must outlive `'static`

error[E0521]: borrowed data escapes outside of method
  --> bucface_client/src/app.rs:91:9
   |
90 |       async fn update_logs(&self) -> tokio::task::JoinHandle<impl Future> {
   |                            -----
   |                            |
   |                            `self` is a reference that is only valid in the method body
   |                            let's call the lifetime of this reference `'2`
91 | /         tokio::spawn(async move {
92 | |             get_events(
93 | |                 "http://127.0.0.1:8080/events".to_string(),
94 | |                 self.client.clone(),
95 | |                 self.events.len(),
96 | |             )
97 | |         })
   | |          ^
   | |          |
   | |__________`self` escapes the method body here
   |            argument requires that `'2` must outlive `'static`

error[E0308]: mismatched types
   --> bucface_client/src/app.rs:103:19
    |
90  |     async fn update_logs(&self) -> tokio::task::JoinHandle<impl Future> {
    |                                                            ----------- the expected future
...
103 |         while let Err(e) = self.update_logs().await {
    |                   ^^^^^^   ------------------------ this expression has type `tokio::task::JoinHandle<impl std::future::Future>`
    |                   |
    |                   expected `JoinHandle<impl Future>`, found `Result<_, _>`
    |
    = note: expected struct `tokio::task::JoinHandle<impl std::future::Future>`
                 found enum `std::result::Result<_, _>`

error[E0308]: mismatched types
  --> bucface_client/src/input.rs:8:47
   |
8  |         AppMode::Normal => normal_key_handler(app)?,
   |                            ------------------ ^^^ types differ in mutability
   |                            |
   |                            arguments to this function are incorrect
   |
   = note: expected mutable reference `&mut app::App<'_>`
                      found reference `&app::App<'_>`
note: function defined here
  --> bucface_client/src/input.rs:43:4
   |
43 | fn normal_key_handler(app: &mut App) -> std::io::Result<()> {
   |    ^^^^^^^^^^^^^^^^^^ -------------

error[E0599]: no method named `lock` found for enum `app::AppMode` in the current scope
  --> bucface_client/src/input.rs:24:27
   |
24 |                 *app.mode.lock().unwrap() = AppMode::Normal.into();
   |                           ^^^^ method not found in `AppMode`
   |
  ::: bucface_client/src/app.rs:14:1
   |
14 | pub enum AppMode {
   | ---------------- method `lock` not found for this enum
   |
   = help: items from traits can only be used if the trait is implemented and in scope
   = note: the following trait defines an item `lock`, perhaps you need to implement it:
           candidate #1: `lock_api::mutex::RawMutex`

error[E0308]: mismatched types
  --> bucface_client/src/input.rs:28:24
   |
28 |                 return app.send_buf().await;
   |                        ^^^^^^^^^^^^^^^^^^^^ expected `Result<(), Error>`, found `Option<JoinHandle<()>>`
   |
   = note: expected enum `std::result::Result<(), std::io::Error>`
              found enum `std::option::Option<tokio::task::JoinHandle<()>>`

error[E0599]: no method named `lock` found for struct `std::vec::Vec<u8>` in the current scope
  --> bucface_client/src/input.rs:31:25
   |
31 |                 app.buf.lock().unwrap().pop();
   |                         ^^^^ method not found in `Vec<u8>`

error[E0599]: no method named `lock` found for struct `std::vec::Vec<u8>` in the current scope
  --> bucface_client/src/input.rs:34:25
   |
34 |                 app.buf.lock().unwrap().push(c as u8);
   |                         ^^^^ method not found in `Vec<u8>`

error[E0599]: no method named `lock` found for struct `std::vec::Vec<bucface_utils::Event>` in the current scope
  --> bucface_client/src/ui/main_window.rs:40:29
   |
40 |     let events = app.events.lock().await;
   |                             ^^^^ method not found in `Vec<Event>`

error[E0599]: no method named `lock` found for struct `std::boxed::Box<str>` in the current scope
   --> bucface_client/src/app.rs:115:20
    |
115 |         *self.name.lock().unwrap() = self
    |                    ^^^^ method not found in `Box<str>`

error[E0599]: no method named `lock` found for struct `std::vec::Vec<u8>` in the current scope
   --> bucface_client/src/app.rs:117:14
    |
115 |           *self.name.lock().unwrap() = self
    |  ______________________________________-
116 | |             .buf
117 | |             .lock()
    | |             -^^^^ method not found in `Vec<u8>`
    | |_____________|
    | 

error[E0599]: no method named `lock` found for struct `std::vec::Vec<u8>` in the current scope
   --> bucface_client/src/app.rs:123:18
    |
123 |         self.buf.lock().unwrap().clear();
    |                  ^^^^ method not found in `Vec<u8>`

error[E0308]: mismatched types
  --> bucface_client/src/main.rs:39:37
   |
39 |                 let _ = key_handler(app).await;
   |                         ----------- ^^^ expected `&App<'_>`, found `Arc<App<'_>>`
   |                         |
   |                         arguments to this function are incorrect
   |
   = note: expected reference `&app::App<'_>`
                 found struct `std::sync::Arc<app::App<'_>>`
note: function defined here
  --> bucface_client/src/input.rs:5:14
   |
5  | pub async fn key_handler<'a>(app: &App<'a>) -> std::io::Result<()> {
   |              ^^^^^^^^^^^     -------------
help: consider borrowing here
   |
39 |                 let _ = key_handler(&app).await;
   |                                     +

Some errors have detailed explanations: E0308, E0521, E0599.
For more information about an error, try `rustc --explain E0308`.
warning: `bucface_client` (bin "bucface_client") generated 8 warnings
error: could not compile `bucface_client` (bin "bucface_client") due to 13 previous errors; 8 warnings emitted
