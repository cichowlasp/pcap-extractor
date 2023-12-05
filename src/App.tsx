import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/api/dialog';
import Form from './components/Form/Form';
import Loading from './components/Form/Loading';
import Files from './components/Form/Files';
import './App.css';

function App() {
	const [files, setFiles] = useState<string[]>([]);
	const [changeStyle, setChangeStyle] = useState(false);
	const [error, setError] = useState('');
	const [form, setFormView] = useState(false);
	const [loading, setLoading] = useState(false);
	type iData = {
		name: string;
		surname: string;
		timeStart?: Date;
		timeEnd?: Date;
		files?: string[];
	};
	const [data, setData] = useState<iData>({
		name: '',
		surname: '',
	});

	useEffect(() => {
		const unlistenFileDrop = listen(
			'tauri://file-drop',
			async ({ payload }: { payload: string[] }) => {
				setError('');
				payload.forEach((file) => {
					if (
						file.split('.')[file.split('.').length - 1] === 'pcap'
					) {
						setFiles((prev: string[]) => [...prev, file]);
						setFiles((prev: string[]) => {
							return [...new Set(prev)];
						});
					} else {
						setError(
							'Sorry not all selected files were .pcap files'
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
				if (file.split('.')[file.split('.').length - 1] === 'pcap') {
					setFiles((prev: string[]) => [...prev, file]);
					setFiles((prev: string[]) => {
						return [...new Set(prev)];
					});
				} else {
					setError('Sorry not all selected files are .pcap files');
				}
			});
		} else if (selected === null) {
			// user cancelled the selection
			return;
		} else {
			const file = selected;
			// user selected a single file
			if (file.split('.')[file.split('.').length - 1] === 'pcap') {
				setFiles((prev: string[]) => [...prev, file]);
				setFiles((prev: string[]) => {
					return [...new Set(prev)];
				});
			} else {
				setError('Sorry not all selected files are .pcap files');
			}
		}
	};

	if (loading) return <Loading />;

	if (data?.files && data.files.length !== 0)
		return (
			<Files
				data={data}
				setData={setData}
				setFiles={setFiles}
				setLoading={setLoading}
				files={files}
			/>
		);

	return (
		<div className='container'>
			<h1>PCAP Extractor</h1>

			<>
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
											<div
												style={{
													wordBreak: 'break-all',
												}}>
												{file}
											</div>

											<button
												onClick={() =>
													removeFile(index)
												}
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
													style={{
														width: '20px',
														height: '20px',
													}}
												/>
											</button>
										</li>
									);
								})}
							</ul>
							<div
								style={{
									display: 'flex',
									margin: '1rem 0',
								}}>
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

				<button
					onClick={() => setFormView(true)}
					disabled={files.length === 0}
					style={{
						marginTop: '1rem',
						backgroundColor: files.length === 0 ? '' : '#08A045',
					}}>
					Proceed with analysis
				</button>
				<div
					style={{
						minHeight: '20px',
						display: 'flex',
						justifyContent: 'center',
						alignItems: 'center',
						marginBottom: '10px',
						marginTop: '10px',
						color: '#FF4029',
						fontWeight: 'bold',
					}}>
					{error}
				</div>
			</>
			{form && (
				<Form
					files={files}
					setFormView={setFormView}
					setLoading={setLoading}
					setFiles={setFiles}
					data={data}
					setData={setData}
				/>
			)}
		</div>
	);
}

export default App;
