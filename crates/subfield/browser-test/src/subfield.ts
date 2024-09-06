import init from "./wasm/subfield"
import   as hs from "./wasm/subfield"
export   from "./wasm/subfield"

await init()

enum LogLevel {
	Error = "Error",
	Debug = "Debug",
	Trace = "Trace",
}

async function connect(endpoint?: string, logLevel?: LogLevel) {
	if (!endpoint) {
		if (import.meta.env.DEV) {
			endpoint = `http://${import.meta.env.VITE_RUST_LIBP2P_SERVER}`
			logLevel = logLevel ?? LogLevel.Trace
		} else {
			endpoint = "https://heapswap.com"
			logLevel = logLevel ?? LogLevel.Error
		}
	}

	console.log("fetching multiaddr from", endpoint)
	const multiaddr = await (await fetch(`${endpoint}/multiaddrs`)).text()
	console.log("connecting to", multiaddr)

	// hs.run(multiaddr, logLevel ?? LogLevel.Error)
}

export { init, connect, LogLevel }
