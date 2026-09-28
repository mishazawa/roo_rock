#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
extern crate alloc;

use agb::{
    display::{GraphicsFrame, HEIGHT, Priority, object::Object},
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, println,
    sound::mixer::Frequency,
};
use agb_tracker::{Track, Tracker, TrackerPosition, include_xm};

include_aseprite!(
    pub mod sprites,
    "gfx/notes_v1.aseprite",
);

pub static SONG: Track = include_xm!("sfx/test_1.xm");
type FrameCount = Num<u32, 8>;
type Fixed = Num<i32, 8>;

enum Color {
    PINK,
    GREEN,
}
struct Note {
    track_pos: TrackerPosition,
    pos: Vector2D<Fixed>,
    color: Color,
}

impl Note {
    fn new(x: Fixed, tp: TrackerPosition, color: Color) -> Self {
        Self {
            pos: vec2(x, Fixed::from(HEIGHT / 2)),
            color,
            track_pos: tp,
        }
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        let sprite = match self.color {
            Color::PINK => sprites::NOTE_PINK.sprite(0),
            Color::GREEN => sprites::NOTE_GREEN.sprite(0),
        };

        Object::new(sprite)
            .set_pos(self.pos.round())
            .set_priority(Priority::P1)
            .show(frame);
    }
}

const LEAD_ROWS: i32 = 16;
const TRAVEL_PX: i32 = 128;
const PX_PER_ROW: FrameCount = num!(TRAVEL_PX / LEAD_ROWS);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // println!("samples: {:?}", SONG.samples);
    // println!("envelopes: {:?}", SONG.patterns.len());
    // println!("pattern_data: {:?}", SONG.pattern_data.len());
    // println!("patterns: {:?}", SONG.patterns);
    // println!("patterns_to_play: {:?}", SONG.patterns_to_play);
    // println!("num_channels: {:?}", SONG.num_channels);
    // println!("frame per tick: {:?}", SONG.frames_per_tick);
    // println!("tick per step: {:?}", SONG.ticks_per_step);
    // println!("repeat: {:?}", SONG.repeat);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    let mut tracker = Tracker::new(&SONG, mixer.frequency());
    let mut gfx = gba.graphics.get();

    // println!("{:?}", tracker.position());

    let frames_per_row = SONG.frames_per_tick * SONG.ticks_per_step;

    let mut last_pos = tracker.position();
    let mut frames_skipped = num!(0);

    let vel_x = PX_PER_ROW / frames_per_row;

    let mut notes: [Option<Note>; 1] = [None];

    for row in 0..64 {
        for ch in 0..8 {
            if notes.len() == 2 {
                break;
            }

            let slot = &SONG.pattern_data[row * 8 + ch];
            if slot.sample != 0 {
                // println!("pd[{}]: {:?}", n,);

                let b: Fixed = vel_x.try_change_base().unwrap();
                let note = Note::new(
                    Fixed::from(row as i32) * b,
                    TrackerPosition { row, pattern: 0 },
                    if slot.sample == 1 {
                        Color::GREEN
                    } else {
                        Color::PINK
                    },
                );
                notes[notes.len() - 1] = Some(note);
            }
        }
    }

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

        for note in &mut notes {
            if let Some(n) = note {
                let pfixed: Fixed = pos.try_change_base().unwrap();
                let rows_left = n.pos.x - pfixed;
                n.pos.x = rows_left;
                println!("pos: {}", n.pos.x);
            }
        }
        let mut frame = gfx.frame();

        for note in &notes {
            if let Some(n) = note {
                n.show(&mut frame);
            }
        }

        tracker.step(&mut mixer);
        mixer.frame();
        frame.commit();
    }
}
