-- Independent mobile configuration; the desktop init/config are not loaded.
return {
    profile = os.getenv("SOLIUM_MOBILE_PROFILE") or "auto",
    status = 32,
    navigation = 64,
    motion = { duration = 300, easing = "outCubic" },
    quick = { duration = 180, easing = "outCubic" },
    qml_runner = os.getenv("SOLIUM_QML_RUNNER") or "qml",
}
