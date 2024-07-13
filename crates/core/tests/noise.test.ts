import * as hs from "../index.ts"
import { expect, test } from "bun:test"

await hs.init()

test("noise", async () => {
	const initiator = hs.Noise.initiator()
	const responder = hs.Noise.responder()

	// handshake

	// step 1 : initiator -> responder
	let message1 = initiator.handshakeStep1()

	// step 2 : responder -> initiator
	let message2 = responder.handshakeStep2(message1)

	// step 3 : initiator -> responder
	initiator.handshakeStep3(message2)

	// handshake is done

	// test - initiator -> responder

	let hello = "hello world!"
	const helloMessage = hs.fromString(hello)

	let encrypted = initiator.encrypt(helloMessage)
	let decrypted = responder.decrypt(encrypted)
	expect(hs.toString(decrypted)).toBe(hello)

	// each chunk (1kb) has 16 bytes of overhead
	expect(encrypted.length - decrypted.length).toBe(16)

	// test - responder -> initiator
	let large = "a".repeat(1024 * 1024)
	const largeMessage = hs.fromString(large)
	encrypted = responder.encrypt(largeMessage)
	decrypted = initiator.decrypt(encrypted)
	expect(hs.toString(decrypted)).toBe(large)
})
