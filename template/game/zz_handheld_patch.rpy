# ==============================================================================
# mkrp Handheld Performance & Compatibility Patch
# Optimized for ARM64 PortMaster Handheld Consoles (Knulli, ROCKNIX, ArkOS)
# ==============================================================================

init -999 python:
    import builtins
    import renpy

    # 1. Image Cache & Rendering Performance Tuning
    # Allocate 256MB for image cache to eliminate stutter when switching dialogues/CGs
    config.image_cache_size_mb = 256
    config.predict_statements = 32
    config.framerate = 60

    # 2. Python 2 cmp builtin polyfill for Ren'Py 8 / Python 3
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
    # 3. Class comparison polyfills (Python 2 -> Python 3 migration)
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

    for cls_name in ['Love', 'TrueLove', 'FalseLove', 'Lust', 'Harem', 'MeterWrapper', 'Meter', 'Progress', 'Attr', 'Stat', 'TimeUnit', 'Cash', 'Message']:
        target_cls = globals().get(cls_name)
        if target_cls and isinstance(target_cls, type):
            _make_comparable(target_cls)

    for name, obj in list(globals().items()):
        if isinstance(obj, type) and hasattr(obj, '__cmp__'):
            _make_comparable(obj)

    # 4. Direct patch for Event.get_triggerable (VIRTUES and similar games)
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

    # 5. Night scene & Clock DynamicDisplayable optimization
    # In VIRTUES, clock_solid_func requests redraw every 0.01s (100fps), causing severe
    # GIL thrashing and TBDR GPU alpha overdraw during night/time transition.
    if 'clock_solid_func' in globals():
        def _opt_clock_solid_func(screen_time, at, *args, **kwargs):
            return Solid(clock_color, *args, **kwargs), 0.5
        globals()['clock_solid_func'] = _opt_clock_solid_func

    # Reduce 6-pass text outline overdraw to 1-pass for ARM Mali GPUs
    if hasattr(store, 'gui') and hasattr(store.gui, 'clock_timeext_outlines'):
        store.gui.clock_timeext_outlines = [(1.5, "#F0EEE924")]
