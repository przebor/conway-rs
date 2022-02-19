use std::env;

use pixels::{SurfaceTexture, Pixels};
use rand::{Rng, distributions::Uniform, prelude::Distribution};
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::LogicalSize, window::WindowBuilder, event::{Event, VirtualKeyCode}};
use winit_input_helper::WinitInputHelper;

const SIZE_OF_CELL: usize = 10;
const N: usize = 50;
const OFFSET_X: usize = 10;

struct LifeSimulation {
    surface: Vec<bool>,
    live: Vec<usize>,
    live_back: Vec<usize>,
}

fn get_cell(i: usize) -> usize {
    (i.rem_euclid(N*SIZE_OF_CELL) as f64/SIZE_OF_CELL as f64).floor() as usize + (i as f64 / (N*SIZE_OF_CELL*SIZE_OF_CELL) as f64).floor() as usize * N
}

impl LifeSimulation {
    fn new(size: usize, live: Vec<usize>, live_back: Vec<usize>) -> Self {
        let mut s = Self {
            surface: vec![false; size as usize * size as usize],
            live: live,
            live_back: live_back,
        };
        /*let u = Uniform::from(0..10);
        for i in 0..s.surface.len() {
            let r = u.sample(&mut rand::thread_rng());
            s.surface[i] = if r > 7 { true } else { false };
        }*/
        s
    }

    fn change_at(&mut self, index: usize) { 
        self.surface[index] = !self.surface[index];
    }

    fn simulate_step(&mut self) {
        let mut temp = self.surface.clone();
        let adj: [i32; 8] = [-1 * (N as i32) - 1, -1 * (N as i32), -1 * (N as i32) + 1, -1, 1, (N as i32) - 1, (N as i32), (N as i32) + 1];

        for cell in 0..self.surface.len() {
            let mut count: usize = 0;
            for a in &adj {
                let mut adj_c = cell as i32 + a;
                if adj_c < 0 || adj_c >= (N*N) as i32 { continue; }
                if self.surface[adj_c as usize] {
                    count += 1;
                }
            }
            if self.surface[cell] {
                if !(self.live.contains(&count)) {
                    temp[cell] = false;
                }
            } else {
                if self.live_back.contains(&count) {
                    temp[cell] = true;
                }
            }
        }
        self.surface = temp;
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = if self.surface[get_cell(i)] {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            };
            pixel.copy_from_slice(&color);
        }
    }
}

fn main() {
    let size: usize = N * SIZE_OF_CELL;

    let arg: String = env::args().collect::<Vec<String>>()[1].clone();
    let rules_raw: Vec<&str> = arg.split("-").collect();
    let mut live: Vec<usize> = vec![];
    let mut live_back: Vec<usize> = vec![];

    for i in rules_raw[0].chars() {
        live.push(i.to_string().parse::<usize>().unwrap());
    }
    for i in rules_raw[1].chars() {
        live_back.push(i.to_string().parse::<usize>().unwrap());
    }

    let mut sim: LifeSimulation = LifeSimulation::new(size, live, live_back);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(size as f64, size as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(size as u32, size as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            sim.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.mouse_pressed(0) {
                let mouse = input.mouse().unwrap();
                let val = (mouse.0).floor() as usize+(mouse.1).floor() as usize*N*SIZE_OF_CELL;
                sim.change_at(get_cell(val));
            }
            // Update internal state and request a redraw
            if input.key_pressed(VirtualKeyCode::Space) { 
                sim.simulate_step();
            };
        }
        window.request_redraw();
    });
}
