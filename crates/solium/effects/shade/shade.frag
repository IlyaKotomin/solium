// The edge shade, in the part's own pixels: d is the signed distance to its
// rounded outline (negative inside) and x = -d how far in a pixel is. Across
// the band x < width it is the colour at strength * (1 - x / width)^falloff,
// darkest at the very edge and nothing at the band's inner edge or past it,
// so every line of equal shade is concentric with the outline, the corners
// included. Premultiplied, and antialiased across the outline itself.
vec4 sol_effect(vec2 uv) {
    vec2 size = sol_content.zw * sol_box_px;
    vec2 p = sol_to_content(uv);
    vec4 radii = p_radius < 0.0 ? sol_radii : vec4(p_radius);
    float d = sol_sdf_rrect(p, size, radii);
    float cover = clamp(0.5 - d, 0.0, 1.0);
    if (cover <= 0.0 || p_width <= 0.0) return vec4(0.0);
    float x = max(-d, 0.0);
    float fall = pow(clamp(1.0 - x / p_width, 0.0, 1.0), p_falloff);
    float a = clamp(p_strength * p_color.a * fall, 0.0, 1.0) * cover;
    return vec4(p_color.rgb * a, a);
}
