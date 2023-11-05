import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/dialog';

import './App.css';

function App() {
	const [files, setFiles] = useState<string[]>([]);
	const [changeStyle, setChangeStyle] = useState(false);
	const [error, setError] = useState('');

	useEffect(() => {
		const unlistenFileDrop = listen(
			'tauri://file-drop',
			async ({ payload }: { payload: string[] }) => {
				setError('');
				payload.forEach((file) => {
					console.log(file.split('.')[file.split('.').length - 1]);
					if (
						file.split('.')[file.split('.').length - 1] === 'pcap'
					) {
						setFiles((prev: string[]) => [...prev, file]);
						setFiles((prev: string[]) => {
							return [...new Set(prev)];
						});
					} else {
						setError(
							'Sorry not all selected files are .pcap files'
						);
					}
				});
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

	const removeFile = async (index: number) => {
		setFiles((prev) => prev.filter((_, i) => i !== index));
	};

	const selectFile = async () => {
		const selected = await open({
			directory: false,
			multiple: true,
		});
		if (Array.isArray(selected)) {
			// user selected multiple files
			selected.forEach((file) => {
				if (!files.includes(file)) {
					setFiles((prev: string[]) => [...prev, file]);
				}
			});
		} else if (selected === null) {
			// user cancelled the selection
			return;
		} else {
			// user selected a single file
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
					<>
						<ul>
							{files.map((file, index) => {
								return (
									<li key={index} value={file}>
										<div style={{ wordBreak: 'break-all' }}>
											{file}
										</div>

										<button
											onClick={() => removeFile(index)}
											style={{
												display: 'flex',
												justifyContent: 'center',
												alignItems: 'center',
												maxWidth: '30px',
												maxHeight: '30px',
												padding: '5px',
												marginLeft: '10px',
												background: '#FF4029',
												boxShadow: 'none',
											}}>
											<img
												width={'20px'}
												height={'20px'}
												src='/trash.svg'
												alt='trash icon'
											/>
										</button>
									</li>
								);
							})}
						</ul>
						<div style={{ display: 'flex', margin: '1rem 0' }}>
							<button
								style={{ marginRight: '0.5rem' }}
								onClick={() => selectFile()}>
								Add more files
							</button>
							<button
								style={{
									marginLeft: '0.5rem',
									background: '#FF4029',
								}}
								onClick={() => setFiles([])}>
								Clear all files
							</button>
						</div>
					</>
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
			<div
				style={{
					minHeight: '20px',
					display: 'flex',
					justifyContent: 'center',
					alignItems: 'center',
					marginBottom: '1rem',
					marginTop: '1rem',
					color: '#FF4029',
					fontWeight: 'bold',
				}}>
				{error}
			</div>
		</div>
	);
}

export default App;
