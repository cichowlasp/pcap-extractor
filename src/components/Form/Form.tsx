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
	setLinks,
}: {
	files: string[];
	setFormView: React.Dispatch<React.SetStateAction<boolean>>;
	setLoading: React.Dispatch<React.SetStateAction<boolean>>;
	setFiles: React.Dispatch<React.SetStateAction<string[]>>;
	setLinks: React.Dispatch<
		React.SetStateAction<
			{
				link: string;
				stats?: {
					malicious: number;
					suspicious: number;
					undetected: number;
					harmless: number;
					timeout: number;
				};
			}[]
		>
	>;
	data: iData;
	setData: React.Dispatch<React.SetStateAction<iData>>;
}) => {
	async function isUrlSafe(url: string) {
		const apiUrl = 'https://www.virustotal.com/api/v3/urls';
		const options = {
			method: 'POST',
			headers: {
				accept: 'application/json',
				'x-apikey': import.meta.env.VITE_API,
				'content-type': 'application/x-www-form-urlencoded',
			},
			body: new URLSearchParams({ url }),
		};

		await fetch(apiUrl, options)
			.then((response) => response.json())
			.then((response) => console.log(response))
			.catch((err) => console.error(err));

		try {
			// Send URL for scanning
			const response = await fetch(apiUrl, options);
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}

			const scanData = await response.json();

			// Check scan results
			const resultUrl = scanData.data.links.self;
			const resultResponse = await fetch(resultUrl, {
				method: 'GET',
				headers: {
					accept: 'application/json',
					'x-apikey': import.meta.env.VITE_API,
				},
			});

			if (!resultResponse.ok) {
				throw new Error(`HTTP error! status: ${resultResponse.status}`);
			}

			const resultData = await resultResponse.json();
			const stats = resultData.data.attributes.stats;
			return stats;
		} catch (error) {
			console.error('Error:', error);
			return false;
		}
	}

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
						})
							.then((res) => res)
							.then((res) => {
								const paths = res as string[];
								setData((prev) => {
									return {
										...prev,
										files: [
											...(prev?.files ? prev.files : []),
											...paths,
										],
									};
								});
							});
					});
					invoke('find_urls', {
						pcapPaths: files,
					})
						.then((res) => res)
						.then((res) => {
							const urls = res as string;
							const listOfUrls = urls
								.trim()
								.split('\n')
								.filter((_, index) => index !== 0)
								.map((link) => {
									const parsedUrl = new URL(link);
									return `${parsedUrl.protocol}//${parsedUrl.hostname}/`;
								});

							listOfUrls.forEach(async (link) => {
								await isUrlSafe(link)
									.then((res) => res)
									.then((res) => {
										setLinks((pre) => [
											...pre,
											{ link, stats: res },
										]);
									});
							});
						});
					setData((prev) => {
						return { ...prev, timeStart: new Date(Date.now()) };
					});

					setFormView(false);
					setLoading(false);
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
