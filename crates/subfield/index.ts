import { keys } from "@libp2p/crypto"

import init, * as hs from "./pkg"
export * from "./pkg"

// initialize the wasm module
await init()

// string encoding
const encoder = new TextEncoder()
export const fromString = (str: string) => encoder.encode(str)
const decoder = new TextDecoder()
export const toString = (buf: Uint8Array) => decoder.decode(buf)

export const toLibp2pKeypair = (
	keypair: hs.Keypair
): keys.Ed25519PrivateKey => {
	return new keys.Ed25519PrivateKey(
		keypair.toBytes(),
		keypair.publicKey.toBytes()
	)
}

export const fromLibp2pKeypair = (
	keypair: keys.Ed25519PrivateKey
): hs.Keypair => {
	return new hs.Keypair(hs.PrivateKey.fromBytes(keypair.bytes))
}
