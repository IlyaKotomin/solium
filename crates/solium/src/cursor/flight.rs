//! The drawn pointer's flight to where a warp put the pointer (#220).
//!
//! A warp moves the pointer at once: what it is over, what is hovered and
//! which client has it are the destination's from the moment it is carried
//! out (`crate::input::warp`). Only its picture travels, from where it was
//! drawn to where the pointer now is, along the motion the script asked for.
//! This is the picture's half: arithmetic on the compositor's clock and
//! nothing else, so it is tested without a session.

use std::time::Duration;

use smithay::utils::{Logical, Point};
use solium_animation::{Animation, lerp};

use crate::script::AnimationSpec;

/// How far ahead the velocity is measured, on the compositor's clock.
///
/// Fixed, and not a setting: it is the step of a numerical derivative, not a
/// behaviour. A millisecond is finer than any frame the velocity is read at,
/// so it reads the curve's own slope there, a spring's turn back past the
/// target included.
/// `tests::the_velocity_is_the_flights_own_and_nothing_once_it_lands`.
const STEP: Duration = Duration::from_millis(1);

/// One flight of the drawn pointer: from where it was drawn when the warp was
/// carried out to where the warp put the pointer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Flight {
    from: Point<f64, Logical>,
    to: Point<f64, Logical>,
    animation: Animation,
}

impl Flight {
    /// A flight from `from` to `to`, starting at `now`, timed by `motion`. A
    /// spring takes as long as it takes, whatever the duration says.
    /// `tests::the_drawn_pointer_flies_from_where_it_was_to_where_the_pointer_is`.
    pub(crate) fn new(
        from: Point<f64, Logical>,
        to: Point<f64, Logical>,
        now: Duration,
        motion: AnimationSpec,
    ) -> Self {
        Self {
            from,
            to,
            animation: Animation::new(now, motion.duration, motion.easing),
        }
    }

    /// Where the pointer is drawn at `now`: `from` at the start, `to` once it
    /// has landed, and in between by the curve, overshoot and all.
    /// `tests::the_drawn_pointer_flies_from_where_it_was_to_where_the_pointer_is`.
    pub(crate) fn at(&self, now: Duration) -> Point<f64, Logical> {
        if self.landed(now) {
            return self.to;
        }
        let progress = self.animation.progress(now);
        (
            lerp(self.from.x, self.to.x, progress),
            lerp(self.from.y, self.to.y, progress),
        )
            .into()
    }

    /// Whether it has arrived by `now`.
    /// `tests::the_drawn_pointer_flies_from_where_it_was_to_where_the_pointer_is`.
    pub(crate) fn landed(&self, now: Duration) -> bool {
        self.animation.done(now)
    }

    /// How fast the drawn pointer is moving at `now`, in logical pixels a
    /// second, and nothing once it has landed: what `Solium.cursor.velocity`
    /// says while it flies.
    /// `tests::the_velocity_is_the_flights_own_and_nothing_once_it_lands`.
    pub(crate) fn velocity(&self, now: Duration) -> Point<f64, Logical> {
        if self.landed(now) {
            return Point::default();
        }
        let (here, ahead) = (self.at(now), self.at(now.saturating_add(STEP)));
        let seconds = STEP.as_secs_f64();
        ((ahead.x - here.x) / seconds, (ahead.y - here.y) / seconds).into()
    }

    /// Where it set off from.
    pub(crate) const fn from(&self) -> Point<f64, Logical> {
        self.from
    }

    /// Where it is headed, which is where the pointer is.
    pub(crate) const fn to(&self) -> Point<f64, Logical> {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use smithay::utils::{Logical, Point};
    use solium_animation::Curve;

    use super::Flight;
    use crate::script::AnimationSpec;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    fn point(x: f64, y: f64) -> Point<f64, Logical> {
        (x, y).into()
    }

    fn linear(millis: u64) -> AnimationSpec {
        AnimationSpec {
            duration: ms(millis),
            easing: Curve::Linear,
        }
    }

    /// **The drawn pointer flies from where it was to where the pointer is**:
    /// at the start it is where it set off from, half way through a linear
    /// flight it is half way, and from the landing on it is where the
    /// pointer is, and has landed.
    #[test]
    fn the_drawn_pointer_flies_from_where_it_was_to_where_the_pointer_is() {
        let flight = Flight::new(
            point(100.0, 100.0),
            point(500.0, 300.0),
            ms(1000),
            linear(200),
        );
        assert_eq!(
            (
                flight.at(ms(1000)),
                flight.at(ms(1100)),
                flight.at(ms(1200)),
                flight.at(ms(5000)),
            ),
            (
                point(100.0, 100.0),
                point(300.0, 200.0),
                point(500.0, 300.0),
                point(500.0, 300.0),
            ),
            "(at the start, half way, at the landing, long after)"
        );
        assert_eq!(
            (flight.landed(ms(1199)), flight.landed(ms(1200))),
            (false, true),
            "(landed just before the end, landed at it)"
        );
    }

    /// **The velocity is the flight's own, and nothing once it lands**: a
    /// linear flight of 400 pixels in 200 ms is 2000 pixels a second along x
    /// all the way, and zero from the landing on.
    #[test]
    fn the_velocity_is_the_flights_own_and_nothing_once_it_lands() {
        let flight = Flight::new(point(0.0, 0.0), point(400.0, 0.0), ms(0), linear(200));
        let near = |velocity: Point<f64, Logical>, x: f64| {
            (velocity.x - x).abs() < 1.0 && velocity.y.abs() < 1.0
        };
        assert!(
            near(flight.velocity(ms(0)), 2000.0) && near(flight.velocity(ms(150)), 2000.0),
            "a linear flight moves at one speed: {:?} at the start, {:?} at 150 ms",
            flight.velocity(ms(0)),
            flight.velocity(ms(150))
        );
        assert_eq!(flight.velocity(ms(200)), Point::default(), "landed");
    }
}
