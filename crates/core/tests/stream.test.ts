import * as hs from "../index.ts"
import { expect, test } from "bun:test"

test("websocket", async () => {
	let ws = new WebSocket("wss://echo.websocket.org")

	hs.echo_ws(ws)
})
