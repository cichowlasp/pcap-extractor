import './Loading.css';

function Loading() {
	return (
		<div className='container'>
			<div className='spinner'></div>
			<div
				style={{
					fontWeight: 'bold',
					fontSize: '1.5rem',
					marginTop: '2rem',
				}}>
				Processsing PCAP files...
			</div>
		</div>
	);
}

export default Loading;
