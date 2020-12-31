use libpulse_binding::sample;
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
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

const FPS: usize = 20;
const SAMPLE_FREQ: usize = 44100;
const SAMPLE_SIZE: usize = SAMPLE_FREQ / FPS;
const BUFFER_SIZE: usize = mem::size_of::<StereoSampleFrame>() * (SAMPLE_SIZE / 3);

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
	loop {
		frame += 1;
		let canvas = term.canvas_mut();
		canvas.clear();
		let w = canvas.width() as i32;
		let h = canvas.height() as i32;

		let mut data = [0u8; BUFFER_SIZE];
		s.read(&mut data).unwrap();
		s.flush();

		let data: [StereoSampleFrame; BUFFER_SIZE / 8] = unsafe { mem::transmute(data) };

		let lag = s.get_latency().unwrap();

		let pcm_sample: Vec<StereoSampleFrame> = data.into();
		let output = pcm_sample
			.iter()
			.map(|s| format!("({}, {})", s.l, s.r))
			.collect::<Vec<String>>()
			.join(", ");

		canvas.draw_text(0, 0, Color::red(), Color::transparent(), &format!("{}", frame));
		canvas.draw_text(
			2,
			2,
			Color::red(),
			Color::transparent(),
			&format!("Sample Len: {} -- {}", pcm_sample.len(), pcm_sample[0]),
		);

		let interval = pcm_sample.len() / w as usize;
		for x in 0..w {
			let idx = x as usize * interval;
			let l0 = pcm_sample[idx].l;
			let r0 = pcm_sample[idx].r;
			let l1 = pcm_sample[idx + interval].l;
			let r1 = pcm_sample[idx + interval].r;

			let size = (h / 2) as f32;
			let left_offset = (h / 2) - (h / 4);
			let right_offset = (h / 2) + (h / 4);

			canvas.draw_line(
				x,
				left_offset + (size * l0) as i32,
				x,
				left_offset + (size * l1) as i32,
				Cell {
					bg: Color::blue(),
					fg: Color::white(),
					symbol: ' ',
				},
			);

			canvas.draw_line(
				x,
				right_offset + (size * r0) as i32,
				x,
				right_offset + (size * r1) as i32,
				Cell {
					bg: Color::red(),
					fg: Color::white(),
					symbol: ' ',
				},
			);
		}

		term.present().expect("Failed to present terminal");
	}

	term.detach().expect("Failed to detach terminal");
}
