import * as hs from "../index.ts"
import { expect, test } from "bun:test"

test("hash", async () => {
	const hash = hs.hash("hello")

	const verify = hs.hashVerify("hello", hash)

	expect(verify).toBe(true)
})

test("cipher", async () => {
	const cipherKey = hs.Cipher.randomKey()

	const cipher = new hs.Cipher(cipherKey)

	const encrypted = cipher.encrypt(hs.fromString("hello"))

	const decrypted = cipher.decrypt(encrypted)

	expect(hs.toString(decrypted)).toBe("hello")
})

test("keypair - sign/verify/shared", async () => {
	const alice = hs.Keypair.random()
	const bob = hs.Keypair.random()

	// Alice signs a message
	const message = hs.fromString("hello")
	const signature = alice.sign(message)

	// Bob verifies the message
	expect(alice.publicKey.verify(message, signature)).toBe(true)
	expect(bob.publicKey.verify(message, signature)).toBe(false)

	// compute shared secret
	const aliceShared = alice.sharedSecret(bob.publicKey)
	const bobShared = bob.sharedSecret(alice.publicKey)

	expect(aliceShared.toString()).toEqual(bobShared.toString())
})

test("keypair - vanity", async () => {
	// 1 character - Instant
	// 2 characters - <1s
	// 3 characters - <1m
	// 4 characters - <20m
	// 5 characters - <10h
	const prefix = "aa"
	const keypair = await hs.Keypair.vanity(prefix)
	expect(keypair.publicKey.toString().slice(0, prefix.length)).toBe(prefix)
})

test("keypair - serialization", async () => {
	let keypair = hs.Keypair.random()
	let serialized = keypair.toBytes()
	let deserialized = hs.Keypair.fromBytes(serialized)
	expect(keypair.toString()).toBe(deserialized.toString())
})
