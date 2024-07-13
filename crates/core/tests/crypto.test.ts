import * as hs from "../index.ts"
import { expect, test } from "bun:test"

await hs.init()

test("hash", async () => {
	const hash = hs.hash("hello")

	const verify = hs.verifyHash("hello", hash)

	expect(verify).toBe(true)
})

test("jaccard", async () => {
	const a = hs.hash("hello")
	const b = hs.hash("world")

	expect(a.jaccard(b)).toBeLessThan(1)
	expect(a.jaccard(a)).toBe(1)
	expect(a.jaccard(b)).toEqual(b.jaccard(a))
})

test("cipher", async () => {
	const cipherKey = hs.Cipher.randomSecret()

	const cipher = new hs.Cipher(cipherKey)

	const encrypted = cipher.encrypt(hs.fromString("hello"))

	const decrypted = cipher.decrypt(encrypted)

	expect(hs.toString(decrypted)).toBe("hello")
})

test("keys", async () => {
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

test("vanity keypair", async () => {
	// 1 character - Instant
	// 2 characters - <1s
	// 3 characters - <1m
	// 4 characters - <20m
	// 5 characters - <10h
	const prefix = "a"
	const keypair = hs.Keypair.vanity(prefix)
	expect(keypair.publicKey.toString().slice(0, prefix.length)).toBe(prefix)
})
