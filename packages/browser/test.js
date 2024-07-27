import { createLibp2p, SUBFIELD_ECHO_PROTOCOL } from "../libp2p/browser.ts"
import { pipe } from "it-pipe"
import { useEffect } from "react"

const runNode = async () => {
	console.log("running node")

	const swarm = await createLibp2p({
		bootstrapPeers: [
			"/ip4/127.0.0.1/tcp/41613/ws/p2p/12D3KooWBLx5DHRK31hqtNipXoGy7iJ4DYJg9W7yV8Un3A2h9ch2",
		],
		devMode: true,
	})

	console.log(
		"browser node is listening at",
		swarm.getMultiaddrs().map((ma) => ma.toString())
	)

	const stream = await swarm.dialProtocol(
		swarm.getConnections().map((c) => c.remoteAddr),
		SUBFIELD_ECHO_PROTOCOL
	)

	//// now it will write some data and read it back
	const output = await pipe(
		async function* () {
			// the stream input must be bytes
			yield new TextEncoder().encode("hello world")
		},
		stream,
		//@ts-ignore
		async (source) => {
			let string = ""
			const decoder = new TextDecoder()

			for await (const buf of source) {
				// buf is a `Uint8ArrayList` so we must turn it into a `Uint8Array`
				// before decoding it
				string += decoder.decode(buf.subarray())
			}

			return string
		}
	)

	console.log(`Echoed back to us: "${output}"`)
}

runNode()
