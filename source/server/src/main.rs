use std::{io::Write, net::SocketAddr};
use warp::Filter;

#[tokio::main]
async fn main() {
	println!("Hello, world!\n");

	// this channel is used to send a shutdown command to the server
	let (tx_shutdown, mut rx_shutdown) = tokio::sync::mpsc::unbounded_channel();
	let shutdown = tx_shutdown.clone();

	// this channel is used by the server to indicate that it is shutting down
	let (tx_is_shutting_down, is_shutting_down) = tokio::sync::oneshot::channel();

	//------------------------------------------------------------------------//
	// Routing
	//------------------------------------------------------------------------//

	let map_root = warp::path::end().map(|| "kamipad is running");
	let map_hello = warp::path("hi").map(|| "hello world!!!");

	let map_quit = warp::path("quit").map(move || {
		println!("server: received quit request");
		tx_shutdown.send(()).ok();
		"shutting down..."
	});

	//------------------------------------------------------------------------//
	// Server setup
	//------------------------------------------------------------------------//

	// build the complete route map
	let routes = warp::get().and(map_root.or(map_hello).or(map_quit));

	// configure listener
	let addr = "127.0.0.1:0";
	let addr = addr.parse::<SocketAddr>().unwrap();
	let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
		rx_shutdown.recv().await;
		println!("server: started shutdown");
		tx_is_shutting_down.send(()).ok();
	});

	// start server
	let server = tokio::spawn(server);
	println!("server: listening at {}", addr);
	println!("main: press ctrl+c to terminate...");

	//------------------------------------------------------------------------//
	// Shutdown
	//------------------------------------------------------------------------//

	std::io::stdout().flush().ok();

	tokio::select! {
		v = tokio::signal::ctrl_c() => {
			match v {
				Err(e) => {
					println!("error: main: failed to handle ctrl+c: {}", e);
				}
				_ => {
					println!("main: received ctrl+c signal");
					shutdown.send(()).ok();
				}
			}
		}
		_ = is_shutting_down => {
			println!("main: server is shutting down");
		}
	}

	server.await.unwrap();
	println!("main: server shutdown complete");
}
