import { useEffect } from "react"
import init from "./wasm"
//import * as hs from "./wasm/heapswap_core.js";
//export * from "./wasm/heapswap_core.js"

function App() {
	useEffect(() => {
		init()
	}, [])

	return <></>
}

export default App
