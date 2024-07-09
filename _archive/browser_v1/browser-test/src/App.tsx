import { useEffect, useState } from "react";
import ky from "ky";
import init from "./wasm/heapswap_browser.js";
import * as hs from "./wasm/heapswap_browser.js";
//export * from "./wasm/heapswap_browser.js"

function App() {
	const [bootstrapAddrs, setBootstrapAddrs] = useState([] as string[]);

	useEffect(() => {
		const fetchBootstrapAddrs = async () => {
      
			// initalize the wasm runtime
			await init();

			// init the logging
			await hs.init_logging();

			// fetch the bootstrap addresses
			const addrs: string[] = await ky
				.get("http://localhost:3000/bootstrap")
				.json();
			setBootstrapAddrs(addrs);
			console.log("addrs:", addrs);

			// create the config
			const config = new hs.Config(addrs);
			await hs.initialize(config);

			// start the main loop
			console.log("entering main loop (js)");
			//hs.main();
      
      hs.connect()
      
      hs.create_unordered_list_of_connected_multiaddrs()
		};

		fetchBootstrapAddrs();
	}, []); // Empty dependency array means this effect runs once on mount

	console.log(bootstrapAddrs);

	return (
		<>
			<h1>Bootstrap Addresses</h1>
			<ul>
				{bootstrapAddrs ? (
					bootstrapAddrs.map((addr: string) => (
						<li key={addr}>{addr}</li>
					))
				) : (
					<li>Loading...</li>
				)}
			</ul>
      <h1>Connected Addresses</h1>
      <div id="connected-addresses"></div>
		</>
	);
}

export default App;
