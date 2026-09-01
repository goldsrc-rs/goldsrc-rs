//! Engine physics, raycasting, and collision query operations.

/// Raycast / trace collision result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceResult {
    /// 1 if the trace started in an all-solid brush.
    pub all_solid: bool,
    /// 1 if the trace started in a solid brush and exited.
    pub start_solid: bool,
    /// 1 if the trace hit an open area.
    pub in_open: bool,
    /// 1 if the trace hit water.
    pub in_water: bool,
    /// Fraction of the total distance traveled before impact [0.0..1.0].
    pub fraction: f32,
    /// World coordinates of the impact point.
    pub end_pos: [f32; 3],
    /// Normal plane vector of the impacted surface.
    pub plane_normal: [f32; 3],
    /// Entity index hit by the trace (-1 if none, 0 for worldspawn).
    pub hit_entity: i32,
}

impl Default for TraceResult {
    fn default() -> Self {
        Self {
            all_solid: false,
            start_solid: false,
            in_open: true,
            in_water: false,
            fraction: 1.0,
            end_pos: [0.0; 3],
            plane_normal: [0.0, 0.0, 1.0],
            hit_entity: -1,
        }
    }
}

/// Physics, contents, and raycast operations.
pub trait EnginePhysics: Send + Sync {
    /// Query the contents type at a 3D point (e.g. `CONTENTS_WATER`, `CONTENTS_SOLID`).
    fn point_contents(&self, point: [f32; 3]) -> i32;

    /// Cast a ray from `start` to `end`, returning collision details.
    fn trace_line(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        flags: i32,
        ignore_ent: i32,
    ) -> TraceResult;

    /// Cast a bounding hull box from `start` to `end`.
    fn trace_hull(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        flags: i32,
        hull_number: i32,
        ignore_ent: i32,
    ) -> TraceResult;

    /// Cast an arbitrary axis-aligned bounding box from `start` to `end`.
    fn trace_hull_box(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        mins: [f32; 3],
        maxs: [f32; 3],
        flags: i32,
        ignore_ent: i32,
    ) -> TraceResult {
        // Default implementation maps to standard player hull (hull 1) or point trace
        if mins == [0.0; 3] && maxs == [0.0; 3] {
            self.trace_line(start, end, flags, ignore_ent)
        } else {
            self.trace_hull(start, end, flags, 1, ignore_ent)
        }
    }

    /// Cast a ray against a specific entity's bounding box/model.
    fn trace_model(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        flags: i32,
        ent_index: i32,
    ) -> TraceResult {
        self.trace_line(start, end, flags, ent_index)
    }

    /// Checks direct line-of-sight visibility between two world points.
    fn check_visibility(&self, src: [f32; 3], dest: [f32; 3]) -> bool {
        let trace = self.trace_line(src, dest, 0, -1);
        trace.fraction >= 0.999
    }
}
