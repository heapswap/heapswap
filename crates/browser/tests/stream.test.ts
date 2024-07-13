import * as hs from "../index.ts"
import { expect, test } from "bun:test"
import { iterating } from "kahn"

await hs.init()

test("websocket", async () => {
	hs.echo()
	hs.echo()
	hs.echo()
	hs.echo()
})

//test("iterable", async () => {

//	const jsValue = hs.fromString("hello")

//	const collected = Uint8Array.from(await hs.collect_numbers(jsValue))

//	expect(jsValue).toEqual(collected)
//})

/*
test("stream", async () => {
	let iterable = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

	// Convert the array to a ReadableStream
	let stream = new ReadableStream({
		start(controller) {
			for (let item of iterable) {
				controller.enqueue(item)
			}
			controller.close()
		},
	})

	console.log(await hs.process_stream(stream))
})

test("stream with async generator", async () => {
	async function* generateNumbers() {
		let iterable = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
		for (let item of iterable) {
			// Simulate an asynchronous operation, e.g., fetching data
			//await new Promise(resolve => setTimeout(resolve, 100)); // Wait for 100ms
			await new Promise((resolve) => setTimeout(resolve, 0)) // Wait for 0ms
			yield item
		}
	}

	let stream = new ReadableStream({
		async start(controller) {
			for await (let item of generateNumbers()) {
				controller.enqueue(item)
			}
			controller.close()
		},
	})

	console.log(await hs.process_stream(stream))
})
*/

/*
test("iterable with async generator", async () => {
    async function* generateNumbers() {
        let iterable = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        for (let item of iterable) {
            // Simulate an asynchronous operation, e.g., fetching data
            await new Promise(resolve => setTimeout(resolve, 0)); // Wait for 0ms
            yield item;
        }
    }

    // Convert the async generator to an array
    let iterable = [];
    for await (let item of generateNumbers()) {
        iterable.push(item);
    }

    // Pass the iterable directly
    console.log(await hs.process_iterable(iterable));
});
*/
