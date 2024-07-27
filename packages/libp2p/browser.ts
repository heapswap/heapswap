import { gossipsub } from "@chainsafe/libp2p-gossipsub"
import { noise } from "@chainsafe/libp2p-noise"
import { yamux } from "@chainsafe/libp2p-yamux"
import { circuitRelayTransport } from "@libp2p/circuit-relay-v2"
import { dcutr } from "@libp2p/dcutr"
import { identify } from "@libp2p/identify"
import { webSockets } from "@libp2p/websockets"
import { webRTC } from "@libp2p/webrtc"
import * as filters from "@libp2p/websockets/filters"
//import { multiaddr } from '@multiformats/multiaddr'
import { createLibp2p as _createLibp2p } from "libp2p"
//import { fromString, toString } from 'uint8arrays'
import { pubsubPeerDiscovery } from "@libp2p/pubsub-peer-discovery"
import { kadDHT } from "@libp2p/kad-dht"
import { bootstrap } from "@libp2p/bootstrap"
import { IDBDatastore } from "datastore-idb"
import * as common from "./common.ts"
export * from "./common.ts"
//export {SUBFIELD_ECHO_PROTOCOL, SUBFIELD_KAD_PROTOCOL, SUBFIELD_PROTOCOL, SUBFIELD_PUBSUB_PROTOCOL, createLibp2pOptions } from "./common"

export async function createLibp2p(options: common.createLibp2pOptions) {
	console.log("creating libp2p")

	let datastore = new IDBDatastore(options.datastorePath ?? "_datastore")
	await datastore.open()

	// peer discovery
	const peerDiscovery: any = [
		pubsubPeerDiscovery({
			interval: 1000,
		}),
	]

	// add bootstrap peers to peer discovery
	if (options.bootstrapPeers && options.bootstrapPeers.length > 0) {
		peerDiscovery.push(
			bootstrap({
				list: options.bootstrapPeers,
			})
		)
	}

	const libp2p = await _createLibp2p({
		addresses: {
			listen: [
				// create listeners for incoming WebRTC connection attempts on on all
				// available Circuit Relay connections
				"/webrtc",
			],
		},
		transports: [
			// the WebSocket transport lets us dial a local relay
			webSockets({
				// this allows non-secure WebSocket connections for purposes of the demo
				filter: filters.all,
			}),
			// support dialing/listening on WebRTC addresses
			webRTC(),
			// support dialing/listening on Circuit Relay addresses
			circuitRelayTransport({
				// make a reservation on any discovered relays - this will let other
				// peers use the relay to contact us
				discoverRelays: 1,
			}),
		],
		connectionEncryption: [noise()],
		streamMuxers: [yamux()],
		connectionGater: {
			denyDialMultiaddr: () => {
				return !options.devMode
			},
		},
		services: {
			identify: identify(),
			pubsub: gossipsub(),
			dcutr: dcutr(),
			dht: kadDHT({
				protocol: common.SUBFIELD_KAD_PROTOCOL,
				clientMode: true,
			}),
		},
		connectionManager: {
			minConnections: 0,
		},
		datastore,
	})

	console.log("libp2p created", libp2p)

	return libp2p
}
