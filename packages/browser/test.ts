import { createLibp2p, SUBFIELD_ECHO_PROTOCOL } from "@heapswap/libp2p/browser"
import {pipe} from "it-pipe"

const swarm = await createLibp2p({
	bootstrapPeers: [
		'/ip4/127.0.0.1/tcp/34403/ws/p2p/12D3KooWJmDCYzYsnczGag8Noanzw3fWUH8EqrNCSNtekb9YqLHK'
	],
})

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

console.info(`Echoed back to us: "${output}"`)

