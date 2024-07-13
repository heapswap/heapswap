import * as hs from "../index.ts"
import { expect, test } from "bun:test"

await hs.init()

test("websocket", async () => {
	hs.echo()
})
