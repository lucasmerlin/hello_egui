use spline::{Spline, Vec2};
use std::mem;

mod spline;

#[derive(Debug, Clone, PartialEq)]
enum AnimationState {
    Stopped,
    Idle,
    Animating {
        current: Animation,
        queue: Vec<Animation>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Animation {
    from: Vec2,
    to: Vec2,
    start: usize,
    duration: web_time::Duration,
    start_time: Option<web_time::Instant>,
}

#[derive(Debug)]
pub struct PerfectCursor {
    max_interval: web_time::Duration,
    state: AnimationState,
    timestamp: web_time::Instant,
    current_point: Option<Vec2>,
    prev_point: Option<Vec2>,
    spline: Spline,
}

impl Default for PerfectCursor {
    fn default() -> Self {
        Self {
            max_interval: web_time::Duration::from_millis(300),
            state: AnimationState::Idle,
            timestamp: web_time::Instant::now(),
            current_point: None,
            prev_point: None,
            spline: Spline::new(vec![]),
        }
    }
}

impl PerfectCursor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn should_update(&self) -> bool {
        matches!(self.state, AnimationState::Animating { .. })
    }

    pub fn add_point(&mut self, point: (f32, f32)) {
        if Some(point) == self.prev_point.map(spline::Vec2::into_tuple) {
            return;
        }
        let point = Vec2::new(point.0, point.1);
        let now = web_time::Instant::now();
        let duration = Ord::min(now - self.timestamp, self.max_interval);

        if self.prev_point.is_none() {
            self.spline.clear();
            self.prev_point = Some(point);
            self.spline.add_point(point);
            self.current_point = Some(point);
            self.state = AnimationState::Stopped;
            return;
        }

        if self.state == AnimationState::Stopped {
            if self.prev_point.unwrap().dist(point) < 4.0 {
                self.current_point = Some(point);
                return;
            }

            self.spline.clear();
            self.spline.add_point(self.prev_point.unwrap());
            self.spline.add_point(self.prev_point.unwrap());
            self.spline.add_point(point);
            self.state = AnimationState::Idle;
        } else {
            self.spline.add_point(point);
        }

        if duration < web_time::Duration::from_millis(16) {
            self.prev_point = Some(point);
            self.timestamp = now;
            self.current_point = Some(point);
            return;
        }

        let animation = Animation {
            start: self.spline.points.len() - 3,
            from: self.prev_point.unwrap(),
            to: point,
            duration,
            start_time: None,
        };

        self.prev_point = Some(point);
        self.timestamp = now;
        self.current_point = Some(point);

        match self.state {
            AnimationState::Idle => {
                self.state = AnimationState::Animating {
                    current: animation,
                    queue: vec![],
                };
            }
            AnimationState::Animating { ref mut queue, .. } => {
                queue.push(animation);
            }
            AnimationState::Stopped => {}
        }
    }

    pub fn tick(&mut self) -> Option<(f32, f32)> {
        let mut set_state = None;

        let result = match &mut self.state {
            AnimationState::Stopped | AnimationState::Idle => {
                self.current_point.map(spline::Vec2::into_tuple)
            }
            AnimationState::Animating { current, queue } => {
                let start_time = current
                    .start_time
                    .get_or_insert_with(web_time::Instant::now);
                let t = (*start_time).elapsed().as_secs_f32() / current.duration.as_secs_f32();

                let point = self.spline.get_spline_point(t + current.start as f32);

                if t <= 1.0 && !self.spline.points.is_empty() {
                    Some(point.into_tuple())
                } else if !queue.is_empty() {
                    let next = queue.remove(0);
                    set_state = Some(AnimationState::Animating {
                        current: next,
                        queue: mem::take(queue),
                    });

                    Some(point.into_tuple())
                } else {
                    set_state = Some(AnimationState::Idle);

                    Some(point.into_tuple())
                }
            }
        };

        if let Some(state) = set_state {
            self.state = state;
        }

        result
    }
}
