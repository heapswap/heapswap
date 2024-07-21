import React from "react"
import { createLibp2p, SUBFIELD_ECHO_PROTOCOL } from "@heapswap/libp2p/browser"
import {pipe} from "it-pipe"

function App() {
	const runNode = async () => {
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
	}

	runNode()

	return (
		<>
			<div></div>
		</>
	)
}

export default App

//import { noise } from "@chainsafe/libp2p-noise"
//import { yamux } from "@chainsafe/libp2p-yamux"
//import { tcp } from "@libp2p/tcp"
//import { pipe } from "it-pipe"
//import { createLibp2p } from "libp2p"

//async function createNode() {
//	return await createLibp2p({
//		addresses: {
//			listen: ["/ip4/0.0.0.0/tcp/0"],
//		},
//		connectionEncryption: [noise()],
//		streamMuxers: [yamux()],
//		transports: [tcp()],
//	})
//}

//// create two serverSwarms
//const remote = await createNode()
//const local = await createNode()

//const ECHO_PROTOCOL = "/echo/1.0.0"
//const SUBFIELD_PROTOCOL = "/subfield/1.0.0"

//// the remote will handle incoming streams opened on the protocol
////@ts-ignore
//await remote.handle(SUBFIELD_PROTOCOL, ({ stream }) => {
//	// pipe the stream output back to the stream input
//	pipe(stream, stream)
//})

//await remote.handle(ECHO_PROTOCOL, ({ stream }) => {
//	// pipe the stream output back to the stream input
//	pipe(stream, stream)
//})

//// the local will dial the remote on the protocol stream
//const stream = await local.dialProtocol(remote.getMultiaddrs(), SUBFIELD_PROTOCOL)

//console.info(`Echoed back to us: "${output}"`)

//await remote.stop()
//await local.stop()
