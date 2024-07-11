import init from "../pkg/heapswap_browser.js";
import * as hs from "../pkg/heapswap_browser.js";
import { expect, test } from "bun:test";

await init();

const encoder = new TextEncoder();
const decoder = new TextDecoder(); 

test("hash", async () => {
	const hash = hs.hash("hello");

	console.log(hash.toString());

	const verify = hs.verify("hello", hash);

	expect(verify).toBe(true);
});

test("cipher", async () => {
	const cipherKey = hs.Cipher.randomKey();

	console.log(cipherKey.toString());

	const cipher = new hs.Cipher(cipherKey);

	const message = encoder.encode("hello");

	const encrypted = cipher.encrypt(message);

	const decrypted = cipher.decrypt(encrypted);

	expect(decoder.decode(decrypted)).toBe("hello");
});

test("keys", async () => {
	const alice = hs.Keypair.random();
	const bob = hs.Keypair.random();

	// Alice signs a message
	const message = encoder.encode("hello");
	const signature = alice.sign(message);

	// Bob verifies the message
	expect(alice.publicKey().verify(message, signature)).toBe(true);
	expect(bob.publicKey().verify(message, signature)).toBe(false);

	// compute shared secret
	const aliceShared = alice.sharedSecret(bob.publicKey());
	const bobShared = bob.sharedSecret(alice.publicKey());

	expect(aliceShared).toEqual(bobShared);
});

test("vanity keypair", async () => {
	// 1 character - Instant
	// 2 characters - <1s
	// 3 characters - <1m 
	// 4 characters - <20m
	// 5 characters - <10h 
	const prefix = "a";
	const keypair = hs.Keypair.vanity(prefix);
	expect(keypair.publicKey().toString().slice(0, prefix.length)).toBe(prefix);
})