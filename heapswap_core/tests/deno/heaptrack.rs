use std::thread;
use std::time::Duration;
use tikv_jemalloc_ctl::{stats, epoch};
use human_bytes::human_bytes;
use deno_core::*;
use futures::future::join_all;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;


async fn idle_js_runtime(){
	let mut runtime = JsRuntime::new(RuntimeOptions {
		//extensions: vec![ext],
		..Default::default()
	  });
	  
	runtime.execute_script_static("[untitled]", r#"
	
	function idle() {
		//1+1;
		//Deno.core.print("idle");
		// while (true) {
		// 	1+1;
		// }
	  }
	  
	idle();
	  
	"#).unwrap();
}

#[tokio::test]
async fn main() {

    let handle = tokio::spawn(async move {
        let e = epoch::mib().unwrap();
        let mut e_count = 0;
        let allocated = stats::allocated::mib().unwrap();
        let active = stats::active::mib().unwrap();
        let resident = stats::resident::mib().unwrap();

        loop {
            e.advance().unwrap();

            let allocated = allocated.read().unwrap();
            let active = active.read().unwrap();
            let resident = resident.read().unwrap();

            let allocated_hr = human_bytes(allocated as f64);
            let active_hr = human_bytes(active as f64);
            let resident_hr = human_bytes(resident as f64);

            match e_count {
                2 => {
                    println!("resolving idle js runtimes");
					let start = time::Instant::now();
					
					let runtimes = 1000;
					let mut handles = Vec::new();
					
					for _ in 0..runtimes {
                        let handle = tokio::spawn(idle_js_runtime());
                        handles.push(handle);
                    }
					join_all(handles).await; 
					println!("resolved {} idle js runtimes in {}s ({}ms per runtime)", runtimes, (start.elapsed()).as_seconds_f64().round(), ((start.elapsed() / runtimes as f64).as_seconds_f64() * 1000.0).round());
					
				},
				4 => {
					break;
				}
                _ => {}
            }

            e_count += 1;
			println!("{} allocated - {} active - {} resident", allocated_hr, active_hr, resident_hr);
            thread::sleep(Duration::from_secs(1));
        }
    });

    handle.await.unwrap();
}