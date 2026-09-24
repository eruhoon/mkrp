# ==============================================================================
# mkrp Universal Handheld Performance & Compatibility Patch
# Supports: All Ren'Py 7 & 8 Games on PortMaster Handheld Consoles
# ==============================================================================

init -999 python:
    import builtins
    import os
    import gc

    # 1. Python GC Optimization for Handheld Devices
    # Tune GC thresholds to eliminate micro-stutter during frequent Displayable allocations
    try:
        gc.set_threshold(25000, 10, 10)
    except Exception:
        pass

    # 2. Ren'Py 8 Screen System Compatibility Polyfill
    # Fixes AttributeError: module 'renpy' has no attribute 'has_screen' on legacy Ren'Py 7 games
    if not hasattr(renpy, 'has_screen'):
        try:
            import renpy.display.screen as _screen
            renpy.has_screen = getattr(_screen, 'has_screen', lambda name: renpy.get_screen(name) is not None)
        except Exception:
            pass

    # 3. Automatic Hardware RAM Detection & Adaptive Image Cache
    # Detects system memory via /proc/meminfo to scale from 1GB to 4GB+ devices safely
    def _detect_system_ram_mb():
        try:
            if os.path.exists('/proc/meminfo'):
                with open('/proc/meminfo', 'r') as f:
                    for line in f:
                        if line.startswith('MemTotal:'):
                            kb = int(line.split()[1])
                            return kb // 1024
        except Exception:
            pass
        return 2048  # Safe default fallback

    _sys_ram = _detect_system_ram_mb()
    if _sys_ram >= 3500:       # 4GB+ RAM devices (RG VITA Pro, RG Cube, RG556, Odin, etc.)
        config.image_cache_size_mb = 512
        config.predict_statements = 48
        config.font_cache_size = 256  # Ample CJK glyph cache for smooth text rendering
    elif _sys_ram >= 1800:     # 2GB RAM devices (RK3566 2GB, RG405M, etc.)
        config.image_cache_size_mb = 256
        config.predict_statements = 32
        config.font_cache_size = 160
    else:                      # 1GB RAM devices (RG35XX H, RG40XX H, etc.)
        config.image_cache_size_mb = 160
        config.predict_statements = 16  # Conservative prediction to prevent OOM spikes
        config.font_cache_size = 96

    config.framerate = 60

    # 4. Python 2 cmp Builtin Polyfill (Ren'Py 7 -> Ren'Py 8 Migration)
    if not hasattr(builtins, 'cmp'):
        def _py2_cmp(a, b):
            if a is None and b is None:
                return 0
            if a is None:
                return -1
            if b is None:
                return 1
            try:
                if a < b:
                    return -1
                elif a > b:
                    return 1
                return 0
            except Exception:
                return 0
        builtins.cmp = _py2_cmp

init 999 python:
    # 5. Universal Save / Load & Confirm Dialog Optimization
    # Instant menu opening without sluggish FBO Dissolve freezes
    config.enter_yesno_transition = None
    config.exit_yesno_transition = None
    config.enter_transition = None
    config.exit_transition = None
    config.intra_transition = None
    try:
        config.after_load_transition = Dissolve(0.15)
    except Exception:
        pass

    # Save screenshot compression to optimize MicroSD writing speed without downscaling resolution
    try:
        config.thumbnail_quality = 75
    except Exception:
        pass

    # Ensure save slot screenshots always seamlessly fit the UI slot dimensions
    try:
        import renpy.loadsave as _ls
        _orig_slot_ss = getattr(_ls, 'slot_screenshot', None)
        if _orig_slot_ss:
            def _fitted_slot_screenshot(slotname):
                ss = _orig_slot_ss(slotname)
                target_w = getattr(config, 'thumbnail_width', None)
                target_h = getattr(config, 'thumbnail_height', None)
                if ss is not None and target_w and target_h:
                    return Transform(ss, xsize=target_w, ysize=target_h)
                return ss
            _ls.slot_screenshot = _fitted_slot_screenshot
    except Exception:
        pass

    # Slot metadata in-memory cache to eliminate massive microSD random I/O during save/load
    try:
        import renpy.loadsave as _ls

        _orig_slot_json = getattr(_ls, 'slot_json', None)
        if _orig_slot_json:
            _slot_json_cache = {}
            def _cached_slot_json(slot):
                if slot in _slot_json_cache:
                    return _slot_json_cache[slot]
                res = _orig_slot_json(slot)
                _slot_json_cache[slot] = res
                return res
            _ls.slot_json = _cached_slot_json

        _orig_slot_mtime = getattr(_ls, 'slot_mtime', None)
        if _orig_slot_mtime:
            _slot_mtime_cache = {}
            def _cached_slot_mtime(slot):
                if slot in _slot_mtime_cache:
                    return _slot_mtime_cache[slot]
                res = _orig_slot_mtime(slot)
                _slot_mtime_cache[slot] = res
                return res
            _ls.slot_mtime = _cached_slot_mtime

        # Granular cache invalidation: only evict modified slot instead of clearing everything
        _orig_save = getattr(_ls, 'save', None)
        if _orig_save:
            def _opt_save(slot, *args, **kwargs):
                if _orig_slot_json: _slot_json_cache.pop(slot, None)
                if _orig_slot_mtime: _slot_mtime_cache.pop(slot, None)
                return _orig_save(slot, *args, **kwargs)
            _ls.save = _opt_save

        _orig_unlink = getattr(_ls, 'unlink_save', None)
        if _orig_unlink:
            def _opt_unlink(slot, *args, **kwargs):
                if _orig_slot_json: _slot_json_cache.pop(slot, None)
                if _orig_slot_mtime: _slot_mtime_cache.pop(slot, None)
                return _orig_unlink(slot, *args, **kwargs)
            _ls.unlink_save = _opt_unlink
    except Exception:
        pass

    # 6. Universal Rollback Memory Management (Prevent memory bloat during long sessions)
    config.rollback_length = 20
    config.hard_rollback_limit = 40

    # 7. Snappy Fade / Dissolve Transitions for Handheld
    try:
        fade = Fade(0.15, 0.0, 0.15)
        dissolve = Dissolve(0.15)
        fadehold = Fade(0.15, 0.2, 0.15)
        longdissolve = Dissolve(0.3)
        flashlight = Dissolve(0.3)
    except Exception:
        pass

    # 8. Universal Class Comparison Polyfills (Python 3 '<' comparison with None defense)
    # Optimized value extraction avoiding repeated hasattr exception overhead
    def _make_comparable(cls):
        if not cls:
            return cls

        orig_cmp = getattr(cls, '__cmp__', None)

        def _get_val(x):
            return getattr(x, 'value', getattr(x, '_value', x))

        def _do_cmp(self, other):
            if other is None:
                return 1
            if orig_cmp is not None:
                try:
                    res = orig_cmp(self, other)
                    if res is not None:
                        return res
                except Exception:
                    pass
            v_self = _get_val(self)
            v_other = _get_val(other)
            if v_other is None:
                return 1
            try:
                if v_self < v_other:
                    return -1
                elif v_self > v_other:
                    return 1
                return 0
            except Exception:
                return 1

        cls.__lt__ = lambda self, other: False if other is None else (_do_cmp(self, other) < 0)
        cls.__le__ = lambda self, other: False if other is None else (_do_cmp(self, other) <= 0)
        cls.__gt__ = lambda self, other: True if other is None else (_do_cmp(self, other) > 0)
        cls.__ge__ = lambda self, other: True if other is None else (_do_cmp(self, other) >= 0)
        cls.__eq__ = lambda self, other: False if other is None else (_do_cmp(self, other) == 0)
        cls.__ne__ = lambda self, other: True if other is None else (_do_cmp(self, other) != 0)
        return cls

    for name, obj in list(globals().items()):
        if isinstance(obj, type) and hasattr(obj, '__cmp__'):
            _make_comparable(obj)
