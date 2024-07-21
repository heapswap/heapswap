//import * as hs from '../../crates/core'

export const SUBFIELD_PROTOCOL = "/subfield/1.0.0"
export const SUBFIELD_PUBSUB_PROTOCOL = "/subfield/pubsub/1.0.0"
export const SUBFIELD_KAD_PROTOCOL = "/subfield/kad/1.0.0"
export const SUBFIELD_ECHO_PROTOCOL = "/subfield/echo/1.0.0"

export interface createLibp2pOptions {
	devMode?: boolean
	bootstrapPeers?: string[]
	datastorePath?: string
}
