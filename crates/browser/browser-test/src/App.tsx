import { useEffect } from "react";
import init from "./wasm";
//import * as hs from "./wasm/heapswap_browser.js";
//export * from "./wasm/heapswap_browser.js"

function App() {
	useEffect(() => {
		init();
	}, []);

	return <></>;
}

export default App;
