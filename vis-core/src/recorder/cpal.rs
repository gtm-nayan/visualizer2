use crate::analyzer;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct CPalBuilder {
	pub rate: Option<usize>,
	pub buffer_size: Option<usize>,
	pub read_size: Option<usize>,
}

impl CPalBuilder {
	pub fn new() -> CPalBuilder {
		Default::default()
	}

	pub fn rate(&mut self, rate: usize) -> &mut CPalBuilder {
		self.rate = Some(rate);
		self
	}

	pub fn buffer_size(&mut self, buffer_size: usize) -> &mut CPalBuilder {
		self.buffer_size = Some(buffer_size);
		self
	}

	pub fn read_size(&mut self, read_size: usize) -> &mut CPalBuilder {
		self.read_size = Some(read_size);
		self
	}

	pub fn create(&self) -> CPalRecorder {
		CPalRecorder::from_builder(self)
	}

	pub fn build(&self) -> Box<dyn super::Recorder> {
		Box::new(self.create())
	}
}

pub struct CPalRecorder {
	rate: usize,
	_stream: cpal::Stream,
	buffer: analyzer::SampleBuffer,
}

impl Debug for CPalRecorder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CPalRecorder")
			.field("rate", &self.rate)
			.field("buffer", &self.buffer)
			.finish()
	}
}

impl CPalRecorder {
	fn from_builder(build: &CPalBuilder) -> CPalRecorder {
		let rate = build
			.rate
			.unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000));
		let buffer_size = build
			.buffer_size
			.unwrap_or_else(|| crate::CONFIG.get_or("audio.buffer", 16000));
		let read_size = build
			.buffer_size
			.unwrap_or_else(|| crate::CONFIG.get_or("audio.read_size", 256));

		let buf = analyzer::SampleBuffer::new(buffer_size, rate);

		let stream = {
			use cpal::{traits::*, StreamConfig};
			let buf = buf.clone();
			let mut chunk_buffer = vec![[0.0; 2]; read_size];

			let device = cpal::default_host().default_output_device().unwrap();

			let format = device.default_output_config().unwrap();

			let error_callback = move |e| eprintln!("{e}");

			let data_callback = move |buffer: &[f32], _: &'_ _| {
				for chunk in buffer.chunks(chunk_buffer.len() * 2) {
					chunk_buffer.splice(.., chunk.array_chunks::<2>().copied());
					buf.push(&chunk_buffer);
				}
			};

			let stream = device
				.build_input_stream(
					&StreamConfig {
						channels: 2,
						sample_rate: format.sample_rate(),
						buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
					},
					data_callback,
					error_callback,
				)
				.unwrap();

			stream.play().unwrap();
			stream
		};

		CPalRecorder {
			rate,
			_stream: stream,
			buffer: buf,
		}
	}
}

impl super::Recorder for CPalRecorder {
	fn sample_buffer<'a>(&'a self) -> &'a analyzer::SampleBuffer {
		&self.buffer
	}
}
