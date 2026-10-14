#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Task 44 builds each wanted cut in prepare and keys memos by it"
    )
)]

//! xray (Ruling F29): the sharp picture of the layers below a part's band,
//! per monitor, from which every T2 effect reads its `backdrop`. Its source
//! is a parameter, so FX6's lock xray is a second one (Ruling F52).
//! `tests::a_part_reads_the_xray_layers_strictly_below_its_band`,
//! `tests::xray_layers_and_scale_are_read_and_checked`.

use crate::effect::tree::Tree;
use crate::scripted::Layer;
use crate::stack::Band;

/// Which of a monitor's layers an xray holds: a part reads the xray layers
/// strictly below its own band, so a part on `bottom` reads the background
/// alone and every part above it reads the background and bottom layers,
/// at most two cuts a monitor (Ruling F29).
/// `tests::a_part_reads_the_xray_layers_strictly_below_its_band`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Cut {
    /// The background layer.
    Background,
    /// The background and bottom layers.
    Bottom,
}

/// A layer `effects.xray.layers` names.
/// `tests::xray_layers_and_scale_are_read_and_checked`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XrayLayer {
    Background,
    Bottom,
}

/// `effects.xray.straddle`: a part across two monitors reads each
/// monitor's memo (a seam at the edge), or only its own pane's monitor's
/// (amended 2026-10-08, configurable; Task 44 reads it).
/// `tests::xray_keep_straddle_and_warped_are_read_and_checked`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Straddle {
    #[default]
    Each,
    Own,
}

/// `effects.xray.warped`: a warped pane's T2 boxes at its resting
/// rectangle, or where it is drawn (Task 54 reads it).
/// `tests::xray_keep_straddle_and_warped_are_read_and_checked`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Warped {
    #[default]
    Rest,
    Drawn,
}

/// `effects.xray`, held in `effect::settings::Settings.xray` (Ruling 28).
/// `tests::the_default_xray_is_the_wallpaper_alone`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct XrayConfig {
    pub(crate) layers: Vec<XrayLayer>,
    pub(crate) scale: f64,
    /// How long a cut nobody wants is kept, in ms of `state.clock`.
    pub(crate) keep_ms: u64,
    pub(crate) straddle: Straddle,
    pub(crate) warped: Warped,
}

impl Default for XrayConfig {
    fn default() -> Self {
        Self {
            layers: vec![XrayLayer::Background],
            scale: 1.0,
            keep_ms: 2000,
            straddle: Straddle::Each,
            warped: Warped::Rest,
        }
    }
}

/// The keys `effects.xray` takes.
const KEYS: [&str; 5] = ["layers", "scale", "keep_ms", "straddle", "warped"];

/// `effects.xray`, read and checked through `Tree`'s own shape (a table's
/// named keys, a list of words): each key refused by name outside its words
/// or bounds, a number that is not finite included (`settings::tree_of`
/// keeps it), an integer scale read as a number, a whole `keep_ms` Lua holds
/// as a float (`1e4`) read, and an unknown key named with the nearest known
/// one, as Ruling 28's are.
/// `tests::xray_layers_and_scale_are_read_and_checked`,
/// `tests::xray_keep_straddle_and_warped_are_read_and_checked`,
/// `tests::effects_xray_refuses_what_is_not_its_shape`.
pub(crate) fn parse(tree: &Tree) -> Result<XrayConfig, String> {
    let Tree::Table { list, fields } = tree else {
        return Err(
            "`effects.xray` is a table: { layers = { \"background\" }, scale = 1 }".to_owned(),
        );
    };
    if !list.is_empty() {
        return Err(
            "`effects.xray` takes named keys: layers, scale, keep_ms, straddle or warped"
                .to_owned(),
        );
    }
    let mut config = XrayConfig::default();
    for (key, value) in fields {
        match key.as_str() {
            "layers" => config.layers = layers(value)?,
            "scale" => {
                let scale = match value {
                    Tree::Number(number) => *number,
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "a scale of 0.25 to 1, as Lua's integer 1"
                    )]
                    Tree::Int(int) => *int as f64,
                    _ => return Err("`effects.xray.scale` is a number".to_owned()),
                };
                if !(0.25..=1.0).contains(&scale) {
                    return Err(format!("`effects.xray.scale` is 0.25 to 1, not {scale}"));
                }
                config.scale = scale;
            }
            "keep_ms" => {
                config.keep_ms = match value {
                    Tree::Int(ms) => u64::try_from(*ms).ok().filter(|ms| *ms <= 60_000),
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "whole and within 0 to 60000, checked first, so exact"
                    )]
                    Tree::Number(ms) if ms.fract() == 0.0 && (0.0..=60_000.0).contains(ms) => {
                        Some(*ms as u64)
                    }
                    _ => None,
                }
                .ok_or_else(|| {
                    "`effects.xray.keep_ms` is a whole number of ms, 0 to 60000".to_owned()
                })?;
            }
            "straddle" => {
                config.straddle = match value {
                    Tree::Text(word) if word == "each" => Straddle::Each,
                    Tree::Text(word) if word == "own" => Straddle::Own,
                    _ => return Err("`effects.xray.straddle` is each or own".to_owned()),
                };
            }
            "warped" => {
                config.warped = match value {
                    Tree::Text(word) if word == "rest" => Warped::Rest,
                    Tree::Text(word) if word == "drawn" => Warped::Drawn,
                    _ => return Err("`effects.xray.warped` is rest or drawn".to_owned()),
                };
            }
            other => {
                let meant = solium_effects::spec::nearest(other, &KEYS)
                    .map(|meant| format!("; did you mean `{meant}`?"))
                    .unwrap_or_default();
                return Err(format!(
                    "`{other}` is not an xray key: layers, scale, keep_ms, straddle or warped{meant}"
                ));
            }
        }
    }
    Ok(config)
}

/// `effects.xray.layers`: a list of words, nothing named beside them, so a
/// table of names is refused rather than read as no layers.
/// `tests::effects_xray_refuses_what_is_not_its_shape`.
fn layers(value: &Tree) -> Result<Vec<XrayLayer>, String> {
    let list = match value {
        Tree::Table { list, fields } if fields.is_empty() => list,
        _ => return Err("`effects.xray.layers` is a list of words".to_owned()),
    };
    list.iter()
        .map(|word| match word {
            Tree::Text(word) if word == "background" => Ok(XrayLayer::Background),
            Tree::Text(word) if word == "bottom" => Ok(XrayLayer::Bottom),
            Tree::Text(other) => Err(format!(
                "`{other}` is not an xray layer: background or bottom"
            )),
            _ => Err("`effects.xray.layers` is a list of words".to_owned()),
        })
        .collect()
}

impl XrayConfig {
    /// The cut a part on `band` reads: the configured layers strictly below
    /// it, or none. `tests::a_part_reads_the_xray_layers_strictly_below_its_band`.
    pub(crate) fn cut_below(&self, band: Band) -> Option<Cut> {
        let has = |layer| self.layers.contains(&layer);
        match band {
            Band::Layer(Layer::Background, _) => None,
            Band::Layer(Layer::Bottom, _) => has(XrayLayer::Background).then_some(Cut::Background),
            _ if has(XrayLayer::Bottom) => Some(Cut::Bottom),
            _ => has(XrayLayer::Background).then_some(Cut::Background),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cut, XrayConfig, XrayLayer, parse};
    use crate::scripted::Layer;
    use crate::stack::{Band, Owner};

    fn both() -> XrayConfig {
        XrayConfig {
            layers: vec![XrayLayer::Background, XrayLayer::Bottom],
            ..XrayConfig::default()
        }
    }

    /// **The default is the wallpaper alone** ([16] decision 2), a cut
    /// kept 2 s, one result per monitor, read at rest.
    #[test]
    fn the_default_xray_is_the_wallpaper_alone() {
        assert_eq!(
            XrayConfig::default(),
            XrayConfig {
                layers: vec![XrayLayer::Background],
                scale: 1.0,
                keep_ms: 2000,
                straddle: super::Straddle::Each,
                warped: super::Warped::Rest
            }
        );
    }

    /// **A part reads the xray layers strictly below its own band**: a
    /// widget on `bottom` reads the wallpaper only, never itself; a window
    /// reads both; a part on `background` reads nothing.
    #[test]
    fn a_part_reads_the_xray_layers_strictly_below_its_band() {
        let config = both();
        assert_eq!(
            config.cut_below(Band::Layer(Layer::Bottom, Owner::Client)),
            Some(Cut::Background)
        );
        assert_eq!(config.cut_below(Band::Windows), Some(Cut::Bottom));
        assert_eq!(
            config.cut_below(Band::Layer(Layer::Top, Owner::Script)),
            Some(Cut::Bottom)
        );
        assert_eq!(
            config.cut_below(Band::Layer(Layer::Background, Owner::Script)),
            None
        );
        assert_eq!(
            XrayConfig::default().cut_below(Band::Windows),
            Some(Cut::Background)
        );
    }

    fn tree(lua: &str) -> crate::effect::tree::Tree {
        crate::effect::tree::tests::tree(lua).expect("a table")
    }

    /// **`layers` and `scale` are read and checked by name**; the shipped
    /// `scale = 1` is a Lua integer (`Tree::Int`) and reads as 1.
    #[test]
    fn xray_layers_and_scale_are_read_and_checked() {
        assert_eq!(
            parse(&tree(
                r#"{ layers = { "background", "bottom" }, scale = 0.5 }"#
            )),
            Ok(XrayConfig {
                scale: 0.5,
                ..both()
            })
        );
        assert_eq!(
            parse(&tree(r#"{ layers = { "background" }, scale = 1 }"#)),
            Ok(XrayConfig::default())
        );
        assert!(
            parse(&tree(r#"{ layers = { "top" } }"#))
                .expect_err("refused")
                .contains("background or bottom")
        );
        assert!(
            parse(&tree(r#"{ scale = 0.1 }"#))
                .expect_err("refused")
                .contains("0.25")
        );
        assert!(
            parse(&tree(r#"{ sale = 1 }"#))
                .expect_err("refused")
                .contains("layers, scale, keep_ms, straddle or warped")
        );
    }

    /// **`keep_ms`, `straddle` and `warped` are read and checked**
    /// (amended 2026-10-08, configurable).
    #[test]
    fn xray_keep_straddle_and_warped_are_read_and_checked() {
        let read = parse(&tree(
            r#"{ keep_ms = 5000, straddle = "own", warped = "drawn" }"#,
        ))
        .expect("read");
        assert_eq!(
            (read.keep_ms, read.straddle, read.warped),
            (5000, super::Straddle::Own, super::Warped::Drawn)
        );
        assert!(
            parse(&tree("{ keep_ms = 70000 }"))
                .expect_err("refused")
                .contains("0 to 60000")
        );
        assert!(
            parse(&tree(r#"{ straddle = "both" }"#))
                .expect_err("refused")
                .contains("each or own")
        );
        assert!(
            parse(&tree(r#"{ warped = "tilted" }"#))
                .expect_err("refused")
                .contains("rest or drawn")
        );
    }

    /// **`effects.xray` is a table of named keys in their shapes**: a
    /// whole `keep_ms` Lua holds as a float (`1e4`) is read; anything else
    /// is refused naming the key, a number that is not finite included (as
    /// `settings::tree_of` keeps it), a `layers` table with names in it
    /// rather than read as no layers, and a misspelt key with the nearest.
    #[test]
    fn effects_xray_refuses_what_is_not_its_shape() {
        use crate::effect::tree::Tree;
        assert_eq!(
            parse(&tree("{ keep_ms = 1e4 }")).map(|config| config.keep_ms),
            Ok(10_000)
        );
        let not_finite = |key: &str, number: f64| Tree::Table {
            list: Vec::new(),
            fields: [(key.to_owned(), Tree::Number(number))].into(),
        };
        for (tree, says) in [
            (
                not_finite("scale", f64::NAN),
                "`effects.xray.scale` is 0.25 to 1",
            ),
            (
                not_finite("scale", f64::INFINITY),
                "`effects.xray.scale` is 0.25 to 1",
            ),
            (
                not_finite("keep_ms", f64::INFINITY),
                "`effects.xray.keep_ms` is a whole number",
            ),
        ] {
            let refused = parse(&tree).expect_err(says);
            assert!(refused.contains(says), "{says}: {refused}");
        }
        for (lua, says) in [
            ("true", "`effects.xray` is a table"),
            (r#"{ "background" }"#, "takes named keys"),
            (
                "{ layers = { background = true } }",
                "`effects.xray.layers` is a list of words",
            ),
            (
                "{ layers = 'background' }",
                "`effects.xray.layers` is a list of words",
            ),
            (
                "{ layers = { 1 } }",
                "`effects.xray.layers` is a list of words",
            ),
            ("{ scale = 'half' }", "`effects.xray.scale` is a number"),
            (
                "{ keep_ms = 1.5 }",
                "`effects.xray.keep_ms` is a whole number",
            ),
            (
                "{ keep_ms = -1 }",
                "`effects.xray.keep_ms` is a whole number",
            ),
            ("{ straddle = 1 }", "`effects.xray.straddle` is each or own"),
            ("{ sale = 1 }", "did you mean `scale`?"),
        ] {
            let refused = parse(&tree(lua)).expect_err(lua);
            assert!(refused.contains(says), "{lua}: {refused}");
        }
    }
}
