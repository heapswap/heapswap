/* eslint-disable no-console */

import * as libp2p from "@heapswap/libp2p/server"
import { pipe } from "it-pipe"

const swarm = await libp2p.createLibp2p({
	//bootstrapPeers: [],
	//datastorePath: "./_datastore",
	devMode: true,
})

console.log(
	"server node is listening at",
	swarm.getMultiaddrs().map((ma) => ma.toString())
)

await swarm.handle(libp2p.SUBFIELD_ECHO_PROTOCOL, ({ stream }) => {
	// pipe the stream output back to the stream input
	pipe(stream, stream)
})
