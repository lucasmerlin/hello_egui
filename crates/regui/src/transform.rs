use egui::{Pos2, Rect, Vec2, emath::Rot2};

/// Maps the child ui's coordinates to the parent ui's coordinates.
///
/// A uniform scale, then a rotation, then a translation. Rotation and scale are both
/// around the child's origin; [`Regui`](crate::Regui) picks the translation so that the
/// result lands in the space it allocated in the parent.
///
/// Skew and non-uniform scale are left out on purpose: egui strokes, corner radii and
/// blur widths are all single numbers, so they cannot survive a transform that scales x
/// and y differently.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Uniform scale, applied first.
    pub scale: f32,

    /// Rotation, applied after the scale.
    pub rotation: Rot2,

    /// Translation, applied last.
    pub translation: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// Leaves everything where it is.
    pub const IDENTITY: Self = Self {
        scale: 1.0,
        rotation: Rot2::IDENTITY,
        translation: Vec2::ZERO,
    };

    /// A uniform scale around the origin.
    pub fn from_scale(scale: f32) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    /// A rotation around the origin, in radians, clockwise on screen.
    pub fn from_rotation(angle: f32) -> Self {
        Self {
            rotation: Rot2::from_angle(angle),
            ..Self::IDENTITY
        }
    }

    /// A translation.
    pub fn from_translation(translation: Vec2) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Map a position from child space to parent space.
    pub fn mul_pos(self, pos: Pos2) -> Pos2 {
        (self.rotation * (self.scale * pos.to_vec2()) + self.translation).to_pos2()
    }

    /// Map a direction or a distance from child space to parent space.
    ///
    /// Unlike [`Self::mul_pos`], this ignores the translation.
    pub fn mul_vec(self, vec: Vec2) -> Vec2 {
        self.rotation * (self.scale * vec)
    }

    /// The transform that undoes this one.
    pub fn inverse(self) -> Self {
        let rotation = self.rotation.inverse();
        let scale = 1.0 / self.scale;
        Self {
            scale,
            rotation,
            translation: rotation * (-self.translation * scale),
        }
    }

    /// The smallest axis-aligned rectangle in parent space that contains the transformed
    /// `rect`.
    ///
    /// This is the same as the transformed rectangle when [`Self::is_axis_aligned`], and
    /// larger than it otherwise.
    pub fn bounding_rect(self, rect: Rect) -> Rect {
        Rect::from_points(&[
            self.mul_pos(rect.left_top()),
            self.mul_pos(rect.right_top()),
            self.mul_pos(rect.right_bottom()),
            self.mul_pos(rect.left_bottom()),
        ])
    }

    /// Does this transform keep horizontal lines horizontal?
    ///
    /// If it does, rectangles stay rectangles, so egui's clip rectangles survive the
    /// transform exactly. If it doesn't, they have to be widened to their bounding box.
    pub fn is_axis_aligned(self) -> bool {
        // A tenth of a degree. Well below what anyone can see, and well above the error
        // that building a `Rot2` from an angle introduces.
        const EPSILON: f32 = 0.001_745;
        self.rotation.angle().abs() < EPSILON
    }

    /// Is this transform usable, i.e. finite and not collapsed to nothing?
    pub fn is_valid(self) -> bool {
        self.scale.is_finite()
            && self.scale.abs() > f32::EPSILON
            && self.rotation.is_finite()
            && self.translation.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::Transform;
    use egui::{Vec2, emath::Rot2, pos2, vec2};

    fn assert_close(a: egui::Pos2, b: egui::Pos2) {
        assert!((a - b).length() < 1e-4, "{a:?} != {b:?}");
    }

    #[test]
    fn inverse_undoes_the_transform() {
        let transform = Transform {
            scale: 2.5,
            rotation: Rot2::from_angle(0.7),
            translation: vec2(13.0, -4.0),
        };
        let point = pos2(3.0, 8.0);
        assert_close(transform.inverse().mul_pos(transform.mul_pos(point)), point);
        assert_close(transform.mul_pos(transform.inverse().mul_pos(point)), point);
    }

    #[test]
    fn identity_is_axis_aligned_but_a_quarter_turn_is_not() {
        assert!(Transform::IDENTITY.is_axis_aligned());
        assert!(Transform::from_scale(3.0).is_axis_aligned());
        assert!(Transform::from_translation(Vec2::splat(5.0)).is_axis_aligned());
        assert!(!Transform::from_rotation(std::f32::consts::FRAC_PI_2).is_axis_aligned());
        // A half turn keeps lines horizontal, but it flips them, which egui's clip
        // rectangles cannot express either.
        assert!(!Transform::from_rotation(std::f32::consts::PI).is_axis_aligned());
    }

    #[test]
    fn bounding_rect_grows_when_rotated() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(10.0, 10.0));
        assert_eq!(Transform::IDENTITY.bounding_rect(rect), rect);

        let rotated = Transform::from_rotation(std::f32::consts::FRAC_PI_4).bounding_rect(rect);
        let expected = 10.0 * std::f32::consts::SQRT_2;
        assert!((rotated.width() - expected).abs() < 1e-4);
        assert!((rotated.height() - expected).abs() < 1e-4);
    }
}
