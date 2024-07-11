# Heapswap

Heapswap is a standardized collection of WASM utility functions for [Subfield](https://subfield.org).

# Installation
```bash
npm install heapswap
```

# Usage

```javascript
import * as hs from "heapswap";

await hs.init(); // Initialize the WASM module
```

# Features

## U256

-   Most of the objects are a U256 type, which represents a 256-bit unsigned integer. U256 does not support arithmetic operations, since it is meant to be used as addresses or hashes.
-   Hash, PublicKey, PrivateKey, and SharedKey are all based on U256. This allows, for example, a Keypair's SharedKey to be used with the Cipher functions.

```javascript
// Constructors
const u256 = new hs.U256(bytes: Uint8Array): U256
const u256 = hs.U256.random(): U256

// Jaccard similarity of concat(U256, hash(U256))
// The hash provides extra entropy for routing
u256.jaccard(other: U256): number

// Equality
u256.equals(other: U256): boolean

// Bytes
u256.toBytes(): Uint8Array
u256.fromBytes(bytes: Uint8Array): U256

// Strings (base32)
u256.toString(): string
u256.fromString(str: string): U256
```

## Crypto

### Hashing (BLAKE3)

```javascript
hs.hash(bytes: Uint8Array): U256
hs.verifyHash(bytes: Uint8Array, hash: U256): boolean
```

### Cipher (CHA-CHA20)

```javascript
const secret = hs.Cipher.randomSecret(): U256
const encrypted = hs.Cipher.encrypt(secret: U256, message: Uint8Array): Uint8Array
const decrypted = hs.Cipher.decrypt(secret: U256, encrypted: Uint8Array): Uint8Array
```

### Signatures (Ed25519)

```javascript
// Constructor
const keypair = hs.Keypair.random(): Keypair
const keypair = hs.Keypair.vanity(prefix: string): Keypair

// Public and private edwards and montgomery keys
const edwards = keypair.publicKey.edwards(): U256
const montgomery = keypair.publicKey.montgomery(): U256

// Signatures
const signature = keypair.sign(message: Uint8Array): U256
const verified = keypair.verify(message: Uint8Array, signature: U256): boolean

// Shared secret
const sharedSecret = keypair.sharedSecret(publicKey: PublicKey): U256
```

## JacDHT

JacDHT is a DHT that uses [Jaccard Similarity](https://en.wikipedia.org/wiki/Jaccard_index) for its routing. This is much more computationally expensive than XOR distance (finding the nearest node is O(n) instead of O(log(n))) and has the potential for collisions. But, if it works, it should allow routing based on vector similarity.

```javascript
// Nodes

// LocalNode is the local node, and requires a full keypair
const localNode = new hs.LocalNode(obj: Object, keypair: Keypair): LocalNode

// RemoteNode is a remote node, and requires only a public key
const remoteNode = new hs.RemoteNode(
	obj: Object,
	publicKey: PublicKey,
	localNode: LocalNode, // used to calculate the jaccard similarity to self
	pingMs: number,
): RemoteNode


// DHT
const dht = new hs.JacDHT(
	localNode: LocalNode,
	maxDistNodes: number, // Recommended: 32
	maxPingNodes: number // Recommended: 32
	): JacDHT

// Both adding and removing return the node that was evicted, if any
dht.tryAddNode(node: RemoteNode): hs.RemoteNode | undefined
dht.tryRemoveNode(node: RemoteNode): hs.RemoteNode | undefined

// find the nearest node(s) in address space to a given key
// NearestNode has .node and .dist fields, typically the .node is extracted
dht.nearestNode(key: U256): NearestNode
dht.nearestNodes(key: U256, n: number): NearestNode[]

// find the nodes nearest in address space to the local node
dht.nearestNodesToLocalByDist(n: number): NearestNode[]
// find the nodes nearest in latency space to the local node
dht.nearestNodesToLocalByPing(n: number): NearestNode[]
```

## Misc

```javascript
// Bytes

// String encoding
hs.toString(bytes: Uint8Array): string
hs.fromString(str: string): Uint8Array

// Base32 encoding
hs.toBase32(bytes: Uint8Array): string
hs.fromBase32(str: string): Uint8Array
```
