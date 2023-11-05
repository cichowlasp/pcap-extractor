import React, { useState } from 'react';
import './Form.css';

const Form = ({
	files,
	setFormView,
}: {
	files: string[];
	setFormView: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
	type iData = {
		name: string;
		surname: string;
	};
	const [data, setData] = useState<iData>({
		name: '',
		surname: '',
	});

	return (
		<div className='wrapper'>
			<form
				onSubmit={(e) => {
					e.preventDefault();
				}}>
				<h2>Fill up your data</h2>
				<input
					onChange={(event) => {
						setData((prev) => {
							return { ...prev, name: event.target.value };
						});
					}}
					placeholder='Enter a name...'
				/>
				<input
					onChange={(event) => {
						setData((prev) => {
							return { ...prev, surname: event.target.value };
						});
					}}
					placeholder='Enter a surname...'
				/>
				<div>
					<span>
						<button onClick={() => setFormView(false)} id='back'>
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
