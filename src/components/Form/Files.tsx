import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './Files.css';
import { open } from '@tauri-apps/api/dialog';
import { type } from '@tauri-apps/api/os';

type iData = {
	name: string;
	surname: string;
	timeStart?: Date;
	timeEnd?: Date;
	files?: string[];
};

function Files({
	data,
	setData,
	setFiles,
	setLoading,
	setExported,
	files,
	setExportPath,
}: {
	data: iData;
	setData: React.Dispatch<React.SetStateAction<iData>>;
	setFiles: React.Dispatch<React.SetStateAction<string[]>>;
	setLoading: React.Dispatch<React.SetStateAction<boolean>>;
	setExported: React.Dispatch<React.SetStateAction<boolean>>;
	setExportPath: React.Dispatch<React.SetStateAction<string>>;
	files: string[];
}) {
	const [changeStyle] = useState(false);
	const [selectedFiles, setSelectedFiles] =
		useState<{ path: string; selected: boolean }[]>();

	useEffect(() => {
		setSelectedFiles(() => {
			if (data?.files && data.files.length !== 0) {
				return data.files.map((path) => {
					return {
						path,
						selected: false,
					};
				});
			}
			return [];
		});
	}, []);

	const selectFolder = async () => {
		return await open({
			directory: true,
			multiple: false,
		});
	};
	return (
		<div className='container'>
			<h1>
				{data.name} {data.surname}
			</h1>

			<>
				<div className={`file-drop ${changeStyle ? 'hover' : ''}`}>
					<>
						<ul>
							{selectedFiles ? (
								selectedFiles.map((file, index) => {
									return (
										<li key={index} value={file.path}>
											<div
												style={{
													wordBreak: 'break-all',
													maxWidth: '90%',
												}}>
												{file.path}
											</div>
											<input
												type='checkbox'
												checked={file.selected}
												onChange={() => {
													setSelectedFiles((prev) => {
														return prev?.map(
															(toSelect) => {
																if (
																	toSelect ===
																	file
																) {
																	return {
																		path: toSelect.path,
																		selected:
																			!toSelect.selected,
																	};
																}
																return toSelect;
															}
														);
													});
												}}></input>
										</li>
									);
								})
							) : (
								<>error</>
							)}
						</ul>
						<div
							style={{
								display: 'flex',
								margin: '1rem 0',
							}}>
							<button
								className='test'
								style={{
									marginRight: '0.5rem',
									background: '#FF4029',
								}}
								onClick={() => {
									setFiles([]);
									setData((prev) => {
										return { ...prev, files: undefined };
									});
								}}>
								Exit
							</button>
							<button
								style={{
									marginLeft: '0.5rem',
								}}
								onClick={async () => {
									const selected = await selectFolder();
									const osType = await type();
									if (
										selected &&
										typeof selected === 'string'
									) {
										setExportPath(
											selected +
												`${
													osType === 'Windows_NT'
														? '\\'
														: '/'
												}${data.name}_${
													data.surname
												}_PCAP_Dump.zip`
										);
									}
									setData((prev) => {
										return {
											...prev,
											timeEnd: new Date(Date.now()),
										};
									});
									setLoading(true);
									await invoke('zip_and_save_to_directory', {
										filePaths: selectedFiles
											?.filter(
												(file) => file.selected === true
											)
											.map((file) => file.path),
										outputDirectory: selected,
										zipFileName: `${data.name}_${data.surname}_PCAP_Dump.zip`,
										pcapPaths: files,
										name: data.name,
										surname: data.surname,
										timeStart: data.timeStart
											? data.timeStart.toLocaleString()
											: '',
										timeEnd: new Date(
											Date.now()
										).toLocaleString(),
									});
									setData({
										name: '',
										surname: '',
										timeStart: undefined,
										timeEnd: undefined,
										files: [],
									});
									setFiles([]);
									setLoading(false);
									setExported(true);
								}}>
								Create raport
							</button>
						</div>
					</>
				</div>
			</>
		</div>
	);
}

export default Files;
