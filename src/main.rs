use libpulse_binding::sample;
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use rustfft::{FFT, FFTplanner, num_complex::Complex};
use mutunga::{Canvas, Cell, Color, TerminalCanvas};
use std::{fmt, mem, thread, time};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct StereoSampleFrame {
	l: f32,
	r: f32,
}

impl fmt::Display for StereoSampleFrame {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}, {})", self.l, self.r)
	}
}

const FPS: usize = 60;
const SAMPLE_FREQ: usize = 44100;
const SAMPLE_SIZE: usize = SAMPLE_FREQ / FPS;
const BUFFER_SIZE: usize = mem::size_of::<StereoSampleFrame>() * SAMPLE_SIZE;
const DATA_SIZE: usize = BUFFER_SIZE / mem::size_of::<StereoSampleFrame>();

fn main() {
	let mut term = TerminalCanvas::new();
	term.attach().expect("Failed to attach terminal");

	let spec = sample::Spec {
		format: sample::Format::F32le,
		channels: 2,
		rate: SAMPLE_FREQ as u32,
	};
	let s = Simple::new(
		None,
		"Ryan's Cool Visualiser",
		Direction::Record,
		None,
		"Visualiser",
		&spec,
		None,
		None,
	)
	.unwrap();

	let mut frame = 0;
	let mut data = [0u8; BUFFER_SIZE];
	loop {
		frame += 1;
		let canvas = term.canvas_mut();
		canvas.clear();
		let w = canvas.width() as i32;
		let h = canvas.height() as i32;

		s.read(&mut data).unwrap();

		let pcm_sample: &[StereoSampleFrame; DATA_SIZE] = unsafe { mem::transmute(&data) };

		let lag = s.get_latency().unwrap();

		let mut interval = pcm_sample.len() / w as usize / 2;
		if interval == 0 {
			interval = 1;
		}
		canvas.draw_text(
			1, 1,
			Color::green(),
			Color::transparent(),
			&format!("Frame: {}   Sample length: {}   Latency: {}    FPS: {}   Int: {}", frame, pcm_sample.len(), lag, FPS, interval),
		);

		// Wave visualiser
		for x in 0..(w / 2) {
			let idx = x as usize * interval;
			let l0 = pcm_sample[idx].l;
			let r0 = pcm_sample[idx].r;
			let l1 = pcm_sample[idx + interval].l;
			let r1 = pcm_sample[idx + interval].r;

			let size = (h / 4) as f32;
			let left_offset = 0;
			let right_offset = w / 2;

			canvas.draw_line(
				x,
				(h / 4) + (size * l0) as i32,
				x,
				(h / 4) + (size * l1) as i32,
				Cell {
					bg: Color::magenta(),
					fg: Color::black(),
					symbol: ' ',
				},
			);

			canvas.draw_line(
				x + right_offset,
				(h / 4) + (size * r0) as i32,
				x + right_offset,
				(h / 4) + (size * r1) as i32,
				Cell {
					bg: Color::yellow(),
					fg: Color::black(),
					symbol: ' ',
				},
			);
		}

		// Spectrum visualiser
		let mut planner = FFTplanner::new(false);
		let fft = planner.plan_fft(DATA_SIZE);
		let mut left_input: Vec<Complex<f32>> = pcm_sample.iter().map(|s| Complex { re: s.l, im: 0.0 }).collect();
		let mut left_output = vec![Complex{ re: 0.0, im: 0.0 }; DATA_SIZE];
		fft.process(&mut left_input, &mut left_output);

		let mut right_input: Vec<Complex<f32>> = pcm_sample.iter().map(|s| Complex { re: s.r, im: 0.0 }).collect();
		let mut right_output = vec![Complex{ re: 0.0, im: 0.0 }; DATA_SIZE];
		fft.process(&mut right_input, &mut right_output);


		let scale = 1.0 / (DATA_SIZE as f32).sqrt();
		for x in 0..(w / 2) {
			let idx = x as usize * interval;
			let l0 = left_output[idx].re * scale;
			let r0 = right_output[idx].re * scale;

			let size = (h / 4) as f32;
			let left_offset = 0;
			let right_offset = w / 2;

			canvas.draw_line(
				x,
				h - 1,
				x,
				h - 1 + (size * -l0.abs()) as i32,
				Cell {
					bg: Color::green(),
					fg: Color::black(),
					symbol: ' ',
				},
			);

			canvas.draw_line(
				x + right_offset,
				h - 1,
				x + right_offset,
				h - 1 + (size * -r0.abs()) as i32,
				Cell {
					bg: Color::cyan(),
					fg: Color::black(),
					symbol: ' ',
				},
			);
		}

		term.present().expect("Failed to present terminal");
	}

	term.detach().expect("Failed to detach terminal");
}
