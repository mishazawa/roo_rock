#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
extern crate alloc;

use agb::{
    display::{GraphicsFrame, HEIGHT, Priority, object::Object},
    fixnum::{Num, num},
    include_aseprite, println,
    sound::mixer::Frequency,
};
use agb_tracker::{Track, Tracker, TrackerPosition, include_xm};
use alloc::vec::Vec;

include_aseprite!(
    pub mod sprites,
    "gfx/notes_v1.aseprite",
);

static SONG: Track = include_xm!("sfx/test_2.xm");
type FrameCount = Num<u32, 8>;
type Fixed = Num<i32, 8>;

const PX_PER_ROW: Fixed = num!(32);

#[derive(Clone, Copy)]
enum Color {
    PINK,
    GREEN,
}
#[derive(Clone, Copy)]
struct Note {
    color: Color,
    pat: usize,
    row: usize,
}

impl Note {
    fn draw(&self, pos: FrameCount, frame: &mut GraphicsFrame) {
        let b: Fixed = pos.try_change_base().unwrap();
        let rows_left = Fixed::from(self.row as i32) - b;
        let next_pos: i32 = (rows_left * PX_PER_ROW).round();

        if next_pos < 0 {
            return;
        }

        let sprite = match self.color {
            Color::PINK => sprites::NOTE_PINK.sprite(0),
            Color::GREEN => sprites::NOTE_GREEN.sprite(0),
        };

        Object::new(sprite)
            .set_pos((next_pos, HEIGHT / 2))
            .set_priority(Priority::P1)
            .show(frame);
    }
}

struct Pool {
    items: Vec<Note>,
}

impl Pool {
    fn new() -> Self {
        let mut items = Vec::<Note>::new();

        for pat in SONG.patterns_to_play.into_iter() {
            let ch: usize = 1; // temporary look only 1st channel
            let rows = SONG.patterns[*pat].length;

            for row in 0..rows {
                let slot = &SONG.pattern_data[row * 8 + ch];
                if slot.sample != 0 {
                    items.push(Note {
                        color: Color::GREEN,
                        pat: *pat,
                        row,
                    });
                }
            }
        }

        // items.truncate(1);
        Self { items: items }
    }

    fn update(&self, tp: TrackerPosition, pos: FrameCount, frame: &mut GraphicsFrame) {
        for note in &self.items {
            if tp.pattern == note.pat {
                note.draw(pos, frame);
            }
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    println!("samples: {:?}", SONG.samples.len());
    println!("envelopes: {:?}", SONG.envelopes.len());
    println!("pattern_data: {:?}", SONG.pattern_data.len());
    println!("patterns: {:?}", SONG.patterns);
    println!("patterns_to_play: {:?}", SONG.patterns_to_play);
    println!("num_channels: {:?}", SONG.num_channels);
    println!("frame per tick: {:?}", SONG.frames_per_tick);
    println!("tick per step: {:?}", SONG.ticks_per_step);
    println!("repeat: {:?}", SONG.repeat);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    let mut tracker = Tracker::new(&SONG, mixer.frequency());
    let mut gfx = gba.graphics.get();

    tracker.set_should_loop(false);

    let total_rows: usize = SONG
        .patterns_to_play
        .iter()
        .map(|p| SONG.patterns[*p].length)
        .sum();

    println!("{:?}", SONG.patterns[0]);
    println!("total_rows: {:?}", total_rows);
    // println!("{:?}", tracker.position());

    let pool = Pool::new();

    let frames_per_row = SONG.frames_per_tick * SONG.ticks_per_step;
    let mut last_pos = tracker.position();
    let mut frames_skipped = num!(0);
    loop {
        let current_pos = tracker.position();

        if current_pos != last_pos {
            last_pos = current_pos;
            frames_skipped = num!(0);
        } else {
            frames_skipped += 1;
        }

        let pos = FrameCount::from(current_pos.row as u32)
            + (frames_skipped / frames_per_row).min(num!(1));

        let mut frame = gfx.frame();

        pool.update(current_pos, pos, &mut frame);

        tracker.step(&mut mixer);
        mixer.frame();
        frame.commit();
    }
}
