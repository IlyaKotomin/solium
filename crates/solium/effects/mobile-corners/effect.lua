-- A client mask, using the existing self-input/replace effect slot.
-- Radius is supplied by the shell; no mesh or compositor changes.
return {
    api = 1,
    inputs = { "self" },
    params = { radius = { 0, min = 0, max = 512 } },
    frag = "corners.frag",
}
