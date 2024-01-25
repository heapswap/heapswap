import { useState, useEffect, useLayoutEffect, useRef } from "react";
import * as Y from "yjs";
import { WebsocketProvider } from "y-websocket";
import { yCollab } from "y-codemirror.next";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import {
	syntaxHighlighting,
	defaultHighlightStyle,
} from "@codemirror/language";
import * as random from "lib0/random";

function App() {
	
	
	// Create a ref for the editor div
	const editorRef = useRef(null);	
	const editorRef2 = useRef(null);	
	
	const view = useRef(null);
	const view2 = useRef(null);
	
	useLayoutEffect(() => {
	const usercolors = [
		{ color: "#30bced", light: "#30bced33" },
		{ color: "#6eeb83", light: "#6eeb8333" },
		{ color: "#ffbc42", light: "#ffbc4233" },
		{ color: "#ecd444", light: "#ecd44433" },
		{ color: "#ee6352", light: "#ee635233" },
		{ color: "#9ac2c9", light: "#9ac2c933" },
		{ color: "#8acb88", light: "#8acb8833" },
		{ color: "#1be7ff", light: "#1be7ff33" },
	];

	// select a random color for this user
	const userColor = usercolors[random.uint32() % usercolors.length];

	const doc = new Y.Doc();
	const ytext = doc.getText("codemirror");
	
	const doc2 = new Y.Doc();
	const ytext2 = doc2.getText("codemirror");


	const provider = new WebsocketProvider(
		"ws://localhost:8000",
		"my-room",
		doc,
		{ disableBc: true },
	);

	const undoManager = new Y.UndoManager(ytext);

	provider.awareness.setLocalStateField("user", {
		name: "Anonymous " + Math.floor(Math.random() * 100),
		color: userColor.color,
		colorLight: userColor.light,
	});

	const state = EditorState.create({
		doc: ytext.toString(),
		extensions: [
			basicSetup,
			javascript(),
			syntaxHighlighting(defaultHighlightStyle),
			yCollab(ytext, provider.awareness, { undoManager }),
		],
	});
	
    if (editorRef.current && !view.current) {
      view.current = new EditorView({ state: state, parent: editorRef.current });
    }
	
	
	const provider2 = new WebsocketProvider(
		"ws://localhost:8000",
		"my-room",
		doc2,
		{ disableBc: true },
	);

	const undoManager2 = new Y.UndoManager(ytext2);

	provider2.awareness.setLocalStateField("user", {
		name: "Anonymous " + Math.floor(Math.random() * 100),
		color: userColor.color,
		colorLight: userColor.light,
	});

	const state2 = EditorState.create({
		doc: ytext2.toString(),
		extensions: [
			basicSetup,
			javascript(),
			syntaxHighlighting(defaultHighlightStyle),
			yCollab(ytext2, provider2.awareness, { undoManager2 }),
		],
	});
	
    if (editorRef2.current && !view2.current) {
      view2.current = new EditorView({ state: state2, parent: editorRef2.current });
    }
	
	
}, []);

	return (
		<div className="App">
			<header className="App-header">
				<h1>Yjs + CodeMirror</h1>
				<div id="editor" ref={editorRef}></div>
				<div id="editor2" ref={editorRef2}></div>
			</header>
		</div>
	);
}

export default App;
