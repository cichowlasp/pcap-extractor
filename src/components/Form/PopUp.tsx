import React, { useEffect, useState } from 'react';
import './PopUp.css';
import { tauri } from '@tauri-apps/api';

const PopUp = ({
	exported,
	setExported,
	exportPath,
}: {
	exported: boolean;
	setExported: React.Dispatch<React.SetStateAction<boolean>>;
	exportPath: string;
}) => {
	const [progress, setProgress] = useState(0);
	const totalTime = 10000;

	useEffect(() => {
		const startTime = Date.now();
		const updateProgress = () => {
			const currentTime = Date.now();
			const elapsedTime = currentTime - startTime;
			const remainingTime = totalTime - elapsedTime;

			if (remainingTime > 0) {
				const newProgress =
					((totalTime - remainingTime) / totalTime) * 100;
				setProgress(newProgress);
			} else {
				setProgress(100); // Ensure progress is at 100% when time is up
			}

			if (remainingTime > 0) {
				const newProgress =
					((totalTime - remainingTime) / totalTime) * 100;
				setProgress(newProgress);
			} else {
				setProgress(100); // Ensure progress is at 100% when time is up
			}
		};

		const interval = setInterval(updateProgress, 1); // Update progress every second
		setTimeout(() => setExported(false), 10000);
		return () => clearInterval(interval);
	}, []);

	async function showInFolder() {
		console.log(exportPath);
		await tauri.invoke('show_in_folder', { path: exportPath });
	}

	return (
		<div
			className='popup'
			style={{
				top: exported ? '' : '-100%',
			}}>
			<div className='checkmark' />{' '}
			<div className='text'>Files Extracted Succesfully</div>
			<button
				className='button'
				type='button'
				style={{
					minHeight: '3rem',
					minWidth: '3rem',
					padding: '10px',
				}}
				onClick={async () => await showInFolder()}>
				<div className='search' />
			</button>
			<div
				className='progress'
				style={{
					width: `${100 - progress}%`,
				}}
			/>
		</div>
	);
};

export default PopUp;
