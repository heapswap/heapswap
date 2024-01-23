import { useState, useEffect } from "react";
// import reactLogo from './assets/react.svg'
// import viteLogo from '/vite.svg'
import * as Y from "yjs";
import { WebsocketProvider } from "y-websocket";
// import './App.css'

function App() {
	
	const ydoc = new Y.Doc();
	const ymap = ydoc.getMap('my-map') 
	
	const [status, setStatus] = useState("disconnected") // "connected" | "disconnected"
	const now = Date.now()

	useEffect(() => {
	console.log("useEffect fired")
	
	const wsProvider = new WebsocketProvider('ws://localhost:3000/ws', 'my-roomname', ydoc)

	wsProvider.on('status', (event) => {
	// console.log(event.status) // logs "connected" or "disconnected"
	setStatus(event.status)
	})
	
	
	// wait 1 second, then refresh the page
	//setTimeout(() => {
	//	window.location.reload()
	//}, 5000)
	
	
	ymap.set('prop-name', 'value') // value can be anything json-encodable

	}, []); // Empty dependency array ensures this runs once on mount and not on updates
	
	
	return <>
		<h1>Yjs + Vite + React</h1>
		<h1>{now}</h1>
		<p>map:</p>
		{JSON.stringify(ymap.toJSON())}
		<p>status: {status}</p>
	</>;
	
	/*
	useEffect(() => {
        // create a test websocket  and send "ping"
        const ws = new WebSocket('ws://localhost:3000/ws/test')
        ws.onopen = () => {
            ws.send('ping')
            console.log('sent ping')
            
            // send a message every 5 seconds
            setInterval(() => {
                ws.send('ping')
                console.log('sent ping')
            }, 5000)
        }
        
        ws.onmessage = (event) => {
            console.log(event.data)
        }

        // Clean up the connection when the component unmounts
        return () => ws.close();
    }, []); // Empty dependency array ensures this runs once on mount and not on updates
	return <>
	
	</>
		*/
}

export default App;
