// Keep client pixels intact and cut only their outline. sol_to_content and
// sol_box_px handle captures and output scale; the result is premultiplied.
vec4 sol_effect(vec2 uv) {
    vec2 size = sol_content.zw * sol_box_px;
    float radius = min(p_radius, 0.5 * min(size.x, size.y));
    float distance = sol_sdf_rrect(sol_to_content(uv), size, vec4(radius));
    float coverage = clamp(0.5 - distance, 0.0, 1.0);
    return sol_tex(uv) * coverage;
}
