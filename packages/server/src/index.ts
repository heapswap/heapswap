/* eslint-disable no-console */

import * as libp2p from "@heapswap/libp2p-server"
import { pipe } from "it-pipe"

const serverSwarm = await libp2p.createLibp2p({
	bootstrapPeers: [],
	datastorePath: "./_datastore",
	devMode: true
})

console.log("server node is listening at", serverSwarm.getMultiaddrs().map((ma) => ma.toString()))

await serverSwarm.handle(libp2p.SUBFIELD_ECHO_PROTOCOL, ({ stream }) => {
	// pipe the stream output back to the stream input
	pipe(stream, stream)
})