import init from "../pkg/heapswap_browser.js"
import * as hs from "../pkg/heapswap_browser.js"
import { expect, test } from "bun:test"

await init()

test("nodes", async () => {
	const localKeypair = hs.Keypair.random()
	const remoteKeypair = hs.Keypair.random()

	const local = { hello: "world" }
	const remote = { goodbye: "world" }

	const localNode = new hs.LocalNode(local, localKeypair)

	expect(localNode.node.hello).toBe("world")

	const remoteNode = new hs.RemoteNode(
		remote,
		remoteKeypair.publicKey,
		localNode,
		0
	)

	expect(remoteNode.node.goodbye).toBe("world")

	expect(remoteNode.distToLocal).toBeLessThan(1)
})

test("dht", async () => {
	const localKeypair = hs.Keypair.random()
	const local = { hello: "world" }
	const localNode = new hs.LocalNode(local, localKeypair)

	let dht = new hs.JacDHT(localNode, 32, 32)

	for (let i = 0; i < 100; i++) {
		const localNode = new hs.LocalNode(local, localKeypair)
		const remote = { goodbye: "world" }
		const remoteKeypair = hs.Keypair.random()

		const remoteNode = new hs.RemoteNode(
			remote,
			remoteKeypair.publicKey,
			localNode,
			0
		)

		dht.tryAddNode(remoteNode)
	}

	const lookupCount = 3

	console.time("lookup")
	let lookupKey = hs.hash("hello")
	let lookupResult = dht.nearestNodes(lookupKey, lookupCount)
	console.timeEnd("lookup")

	expect(lookupResult.length).toBe(lookupCount)

	expect(lookupResult[0].dist).toBeLessThan(lookupResult[1].dist)
	expect(lookupResult[1].dist).toBeLessThan(lookupResult[2].dist)
})
