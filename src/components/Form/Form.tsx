import React from 'react';
import './Form.css';
import { invoke } from '@tauri-apps/api/tauri';
// When using the Tauri global script (if not using the npm package)
// Be sure to set `build.withGlobalTauri` in `tauri.conf.json` to true

type iData = {
	name: string;
	surname: string;
	timeStart?: Date;
	timeEnd?: Date;
	files?: string[];
};

const Form = ({
	files,
	setFormView,
	setLoading,
	data,
	setData,
}: {
	files: string[];
	setFormView: React.Dispatch<React.SetStateAction<boolean>>;
	setLoading: React.Dispatch<React.SetStateAction<boolean>>;
	setFiles: React.Dispatch<React.SetStateAction<string[]>>;
	data: iData;
	setData: React.Dispatch<React.SetStateAction<iData>>;
}) => {
	return (
		<div className='wrapper'>
			<form
				onSubmit={(e) => {
					e.preventDefault();
					setLoading(true);
					// Invoke the command
					files.forEach(async (file) => {
						await invoke('read_pcap_file', {
							filePath: file,
							outputFolderPath: '/Users/cichowlasp/Downloads/',
						})
							.then((res) => res)
							.then((res) => {
								const paths = res as string[];
								setData((prev) => {
									return {
										...prev,
										files: [
											...(prev?.files ? files : []),
											...paths,
										],
									};
								});
							});
					});
					setData((prev) => {
						return { ...prev, timeStart: new Date(Date.now()) };
					});
					setFormView(false);
					setLoading(false);
					console.log(data);
				}}>
				<h2>Fill up your data</h2>
				<input
					minLength={3}
					onChange={(event) => {
						setData((prev) => {
							return { ...prev, name: event.target.value };
						});
					}}
					placeholder='Enter a name...'
				/>
				<input
					minLength={3}
					onChange={(event) => {
						setData((prev) => {
							return { ...prev, surname: event.target.value };
						});
					}}
					placeholder='Enter a surname...'
				/>
				<div>
					<span>
						<button
							onClick={(e) => {
								e.preventDefault();
								setFormView(false);
							}}
							id='back'>
							Back
						</button>
					</span>
					<span>
						<button
							style={{
								backgroundColor:
									data.name.trim() === '' ||
									data.surname.trim() === ''
										? ''
										: '#08A045',
							}}
							disabled={
								data.name.trim() === '' ||
								data.surname.trim() === ''
							}
							id='submit'
							type='submit'>
							Submit
						</button>
					</span>
				</div>
			</form>
		</div>
	);
};

export default Form;
