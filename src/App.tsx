import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/dialog';

import './App.css';

function App() {
	const [files, setFiles] = useState<string[]>([]);
	const [changeStyle, setChangeStyle] = useState(false);

	useEffect(() => {
		const unlistenFileDrop = listen(
			'tauri://file-drop',
			async ({ payload }: { payload: string[] }) => {
				console.log(payload);
				payload.forEach((file) => {
					setFiles((prev: string[]) => [...prev, file]);
					setFiles((prev: string[]) => {
						console.log(new Set(prev));
						return [...new Set(prev)];
					});
				});
				console.log(files);
				setChangeStyle(false);
			}
		);
		const unlistenFileDropCancelled = listen(
			'tauri://file-drop-cancelled',
			async () => {
				setChangeStyle(false);
			}
		);
		const unlistenFileDropHover = listen(
			'tauri://file-drop-hover',
			async () => {
				setChangeStyle(true);
			}
		);
		return () => {
			unlistenFileDrop;
			unlistenFileDropCancelled;
			unlistenFileDropHover;
		};
	}, []);

	const selectFile = async () => {
		const selected = await open({
			directory: false,
			multiple: true,
		});
		if (Array.isArray(selected)) {
			// user selected multiple files
			selected.forEach((file) => {
				console.log(files, file);
				if (!files.includes(file)) {
					setFiles((prev: string[]) => [...prev, file]);
				}
			});
		} else if (selected === null) {
			// user cancelled the selection
			return;
		} else {
			// user selected a single file
			console.log(selected);
			if (!files.includes(selected)) {
				setFiles((prev: string[]) => [...prev, selected]);
			}
		}
	};

	return (
		<div className='container'>
			<h1>PCAP Extractor</h1>
			<div className={`file-drop ${changeStyle ? 'hover' : ''}`}>
				{files.length === 0 ? (
					<>
						<div className='text'>
							<div>Drag files here </div>
							<div>or</div>
						</div>
						<button onClick={() => selectFile()}>
							Select files
						</button>
					</>
				) : (
					<ul>
						{files.map((file, index) => (
							<li key={index} value={file}>
								{file}
							</li>
						))}
						<button onClick={() => setFiles([])}>
							Clear all files
						</button>
						<button onClick={() => selectFile()}>
							Add more files
						</button>
					</ul>
				)}
			</div>
			{/* <form
				className='row'
				onSubmit={(e) => {
					e.preventDefault();
					greet();
				}}>
				<input
					id='greet-input'
					onChange={(e) => setName(e.currentTarget.value)}
					placeholder='Enter a name...'
				/>
				<button type='submit'>Greet</button>
			</form> */}
		</div>
	);
}

export default App;
