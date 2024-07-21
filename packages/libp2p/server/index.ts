/* eslint-disable no-console */
import { noise } from '@chainsafe/libp2p-noise'
import { yamux } from '@chainsafe/libp2p-yamux'
import { circuitRelayServer } from '@libp2p/circuit-relay-v2'
import { identify } from '@libp2p/identify'
import { mplex } from '@libp2p/mplex'
import { webSockets } from '@libp2p/websockets'
import * as filters from '@libp2p/websockets/filters'
import { createLibp2p as _createLibp2p } from 'libp2p'
import { kadDHT } from '@libp2p/kad-dht'
import { bootstrap } from '@libp2p/bootstrap'
import {mdns} from '@libp2p/mdns'
import { LevelDatastore } from 'datastore-level'
import * as common from '../common.ts'
export * from '../common.ts'

export async function createLibp2p(options: common.createLibp2pOptions) {
  
  const peerDiscovery: any = [
    mdns({
      interval: 20e3
    })
  ];

  if (options.bootstrapPeers && options.bootstrapPeers.length > 0) {
    peerDiscovery.push(
      bootstrap({
        list: options.bootstrapPeers
      })
    );
  }

const libp2p = await _createLibp2p({
  addresses: {
    listen: ['/ip4/127.0.0.1/tcp/0/ws']
  },
  transports: [
    webSockets({
      filter: filters.all
    })
  ],
  connectionEncryption: [noise()],
  streamMuxers: [yamux(), mplex()],
  services: {
    identify: identify(),
    relay: circuitRelayServer({
      reservations: {
        maxReservations: Infinity
      }
    }),
    dht: kadDHT({
      protocol: common.SUBFIELD_KAD_PROTOCOL,
      clientMode: false
    })
  },
  connectionManager: {
    minConnections: 0
  },
  peerDiscovery,
  datastore: new LevelDatastore(options.datastorePath ?? "_datastore")
})

return libp2p
}

