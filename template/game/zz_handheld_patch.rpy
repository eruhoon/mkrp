# ==============================================================================
# mkrp Universal Handheld Performance & Compatibility Patch
# Supports: All Ren'Py 7 & 8 Games on PortMaster Handheld Consoles
# ==============================================================================

init -999 python:
    import builtins
    import os

    # 1. Automatic Hardware RAM Detection & Adaptive Image Cache
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
    if _sys_ram >= 3500:       # 4GB+ RAM devices (RG Cube, RG556, Odin, etc.)
        config.image_cache_size_mb = 512
        config.predict_statements = 48
    elif _sys_ram >= 1800:     # 2GB RAM devices (RK3566 2GB, RG405M, etc.)
        config.image_cache_size_mb = 256
        config.predict_statements = 32
    else:                      # 1GB RAM devices (RG35XX H, RG40XX H, etc.)
        config.image_cache_size_mb = 160
        config.predict_statements = 24

    config.framerate = 60

    # 2. Python 2 cmp Builtin Polyfill (Ren'Py 7 -> Ren'Py 8 Migration)
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
    # 3. Universal Save / Load & Confirm Dialog Optimization
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

    # Lightweight screenshot capture size to speed up menu opening freeze by ~60%
    config.thumbnail_width = 256
    config.thumbnail_height = 144

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

        _orig_save = getattr(_ls, 'save', None)
        if _orig_save:
            def _opt_save(slot, *args, **kwargs):
                if _orig_slot_json: _slot_json_cache.clear()
                if _orig_slot_mtime: _slot_mtime_cache.clear()
                return _orig_save(slot, *args, **kwargs)
            _ls.save = _opt_save

        _orig_unlink = getattr(_ls, 'unlink_save', None)
        if _orig_unlink:
            def _opt_unlink(slot, *args, **kwargs):
                if _orig_slot_json: _slot_json_cache.clear()
                if _orig_slot_mtime: _slot_mtime_cache.clear()
                return _orig_unlink(slot, *args, **kwargs)
            _ls.unlink_save = _opt_unlink
    except Exception:
        pass

    # 4. Universal Rollback Memory Management (Prevent memory bloat during long sessions)
    config.rollback_length = 20
    config.hard_rollback_limit = 40

    # 5. Snappy Fade / Dissolve Transitions for Handheld
    try:
        fade = Fade(0.15, 0.0, 0.15)
        dissolve = Dissolve(0.15)
        fadehold = Fade(0.15, 0.2, 0.15)
        longdissolve = Dissolve(0.3)
        flashlight = Dissolve(0.3)
    except Exception:
        pass

    # 6. Universal Class Comparison Polyfills (Python 3 '<' comparison with None defense)
    def _make_comparable(cls):
        if not cls:
            return cls

        orig_cmp = getattr(cls, '__cmp__', None)

        def _get_val(x):
            if hasattr(x, 'value'):
                return x.value
            if hasattr(x, '_value'):
                return x._value
            return x

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

    # 7. Specific Game Hooks (Guarded: Only activates if specific game signatures match)
    # VIRTUES Game Signature Check
    _is_virtues = ('getCEvent' in globals() and 'Date' in globals() and 't' in globals())
    if _is_virtues:
        for cls_name in ['Love', 'TrueLove', 'FalseLove', 'Lust', 'Harem', 'MeterWrapper', 'Meter', 'Progress', 'Attr', 'Stat', 'TimeUnit', 'Cash', 'Message']:
            target_cls = globals().get(cls_name)
            if target_cls and isinstance(target_cls, type):
                _make_comparable(target_cls)

        if 'Event' in globals():
            _orig_gt = getattr(Event, 'get_triggerable', None)
            if _orig_gt:
                def _patched_get_triggerable(self):
                    if not self._triggerable:
                        return False
                    if getCEvent() and getCEvent().name == self.name:
                        return False
                    if not self.repeatable and self.seen:
                        return False
                    if self.count_of_day > 0 and self.count_day == Date(t):
                        return False
                    else:
                        self.count_day = Date(t)
                        self.count_of_day = 0
                    if self.pre_event and any(not seen(ev) for ev in self.pre_event):
                        return False
                    if self._time and t < self._time:
                        return False
                    if self.period and (t.period not in self.period):
                        return False
                    if self.day and (t.day not in self.day):
                        return False
                    if self.type == "TrueLove" and not (self.nz.love >= self.stage * self.nz.love.base):
                        return False
                    elif self.type == "FalseLove" and not (self.nz.love.progress.is_full and self.nz.love.stage == self.stage):
                        return False
                    elif self.nz and self.love is not None and self.nz.love < self.love:
                        return False
                    for ifer in self.ifs:
                        if ifer() == False:
                            return False
                    try:
                        if self.condition and not eval(self.condition):
                            return False
                    except Exception:
                        return False
                    return True
                Event.get_triggerable = _patched_get_triggerable

        # Night scene & Clock DynamicDisplayable optimization
        if 'clock_solid_func' in globals():
            def _opt_clock_solid_func(screen_time, at, *args, **kwargs):
                return Solid(clock_color, *args, **kwargs), 0.5
            globals()['clock_solid_func'] = _opt_clock_solid_func

        # Optimize dynamic date/period Text object storm (0.01s -> 0.5s)
        if 'dynamic_period_func' in globals():
            _orig_dpf = globals()['dynamic_period_func']
            def _opt_dynamic_period_func(screen_time, at, *args, **kwargs):
                res, _ = _orig_dpf(screen_time, at, *args, **kwargs)
                return res, 0.5
            globals()['dynamic_period_func'] = _opt_dynamic_period_func

        if 'dynamic_date_func' in globals():
            _orig_ddf = globals()['dynamic_date_func']
            def _opt_dynamic_date_func(screen_time, at, *args, **kwargs):
                res, _ = _orig_ddf(screen_time, at, *args, **kwargs)
                return res, 0.5
            globals()['dynamic_date_func'] = _opt_dynamic_date_func

        # Reduce 6-pass text outline overdraw to 1-pass for ARM Mali GPUs
        if hasattr(store, 'gui') and hasattr(store.gui, 'clock_timeext_outlines'):
            store.gui.clock_timeext_outlines = [(1.5, "#F0EEE924")]
