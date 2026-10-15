// Liquid glass, the per-part pass. In the part's own pixels: d is the signed
// distance to its rounded outline (negative inside) and x = -d its depth.
// Within the bezel (x < w), with f = rim(x / w), the backdrop is read from
//   centre + (p - centre) * (1 - zoom * f)  - normal * bend * f:
// a magnification toward the centre (lines near the edge curve inward; the
// read stays inside, since the shape is convex), plus a pull straight in.

// The rim's profile: 1 at the edge, 0 at the bezel's inner edge with no slope
// there, so the flat middle meets the rim without a visible ring. An
// exponential less its own tangent line at t = 1, normalised to 1 at t = 0.
float rim(float t) {
    float k = p_falloff;
    float e = exp(-k);
    float tangent = e * (1.0 + k * (1.0 - t));
    return clamp((exp(-k * t) - tangent) / (1.0 - e * (1.0 + k)), 0.0, 1.0);
}

vec3 seen(vec2 px, float haze) {
    vec2 at = sol_to_uv(px);
    vec3 sharp = sol_sharp(at).rgb;
#if SOL_HAS_soft
    return mix(sharp, sol_soft(at).rgb, haze);
#else
    return sharp;
#endif
}

vec4 sol_effect(vec2 uv) {
    vec2 size = sol_content.zw * sol_box_px;
    vec2 p = sol_to_content(uv);
    vec4 radii = p_radius < 0.0 ? sol_radii : vec4(p_radius);
    float d = sol_sdf_rrect(p, size, radii);
    float cover = clamp(0.5 - d, 0.0, 1.0);
    if (cover <= 0.0) return vec4(0.0);

    // Every read stays inside the part: the pull and the bezel are at most
    // half its thickness.
    float half_thick = 0.5 * min(size.x, size.y);
    float w = min(p_bezel, half_thick);
    float b = min(p_bend, half_thick);
    float x = max(-d, 0.0);
    float f = rim(clamp(x / w, 0.0, 1.0));

    // The outward normal of a companion outline whose corners are at least w
    // round: smooth across the whole bezel, so a thin bar cannot crease.
    vec4 companion = max(radii, vec4(w));
    vec2 h = vec2(0.5, 0.0);
    vec2 n = vec2(sol_sdf_rrect(p + h.xy, size, companion) - sol_sdf_rrect(p - h.xy, size, companion),
                  sol_sdf_rrect(p + h.yx, size, companion) - sol_sdf_rrect(p - h.yx, size, companion));
    float len = length(n);
    n = len > 1e-4 ? n / len : vec2(0.0);

    vec2 centre = 0.5 * size;
    vec2 lens = (p - centre) * (p_zoom * f) + n * (b * f);   // how far in this pixel reads
    float haze = mix(p_haze, p_rim_haze, f);   // the frost: rim_haze at the very edge, haze from the bezel inward
    vec3 c;
    c.r = seen(p - lens, haze).r;
    c.g = seen(p - lens * (1.0 - p_dispersion), haze).g;
    c.b = seen(p - lens * (1.0 - 2.0 * p_dispersion), haze).b;

    // The light: a thin highlight along the edges facing it, a fainter one
    // along the far edges with a soft shade under it, and if asked a fine
    // line along every edge.
    float angle = radians(p_light_angle);
    vec2 light = vec2(cos(angle), -sin(angle));
    float facing = dot(n, light);
    float lit = max(facing, 0.0);
    float away = max(-facing, 0.0);
    float glint = 1.0 - smoothstep(0.0, p_glint_band > 0.0 ? p_glint_band : w, x);
    float shaded = 1.0 - smoothstep(0.0, p_shade_band > 0.0 ? p_shade_band : w, x);
    float line = 1.0 - smoothstep(0.0, 1.5, x);
    c = mix(c, p_tint.rgb, p_tint.a);
    c *= 1.0 - p_shade * away * shaded;
    c *= p_brightness;
    c += p_shine_color.rgb * p_shine_color.a
         * ((p_shine * lit * lit + p_shine_back * away * away) * glint + p_edge_line * line);
    c += (sol_hash(floor(p)) - 0.5) * p_grain;
    return vec4(clamp(c, 0.0, 1.0) * cover, cover);
}
