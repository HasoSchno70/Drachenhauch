"""Tests fuer die reine Partikel-Simulation (gamebasic.particle_sim).

Diese Klasse ist die Vorschau-Sim des Partikel-Editors -- Qt-frei, per
NumPy vektorisiert (bis auf emit(), siehe Kommentar dort). Der Editor
selbst haelt min<=max per UI-Sync ein (siehe test_editor_qt_particle_editor
.py::test_minmax_enforced_on_change), aber die Klasse wird auch direkt
genutzt (Tests, zukuenftige Aufrufer) und muss daher selbst robust sein.
"""
from gamebasic.particle_sim import ParticleSystem


def test_emit_adds_particles():
    sys = ParticleSystem(10.0, 20.0)
    assert sys.count() == 0
    sys.emit(5)
    assert sys.count() == 5
    assert all(x == 10.0 for x in sys._xs)


def test_emit_zero_or_negative_is_noop():
    sys = ParticleSystem(0.0, 0.0)
    sys.emit(0)
    sys.emit(-5)
    assert sys.count() == 0


def test_clear_removes_all_particles():
    sys = ParticleSystem(0.0, 0.0)
    sys.emit(10)
    sys.clear()
    assert sys.count() == 0


def test_update_ages_and_kills_expired_particles():
    sys = ParticleSystem(0.0, 0.0)
    sys.lifetime_min = sys.lifetime_max = 100
    sys.emit(3)
    sys.update(50)
    assert sys.count() == 3          # noch nicht abgelaufen
    sys.update(60)                   # Alter jetzt 110 > 100
    assert sys.count() == 0


def test_update_applies_gravity_and_velocity():
    sys = ParticleSystem(0.0, 0.0)
    sys.vx_min = sys.vx_max = 10.0
    sys.vy_min = sys.vy_max = 0.0
    sys.gravity_x = 0.0
    sys.gravity_y = 100.0
    sys.lifetime_min = sys.lifetime_max = 10_000
    sys.emit(1)
    sys.update(1000)   # dt = 1s
    assert sys._xs[0] > 0.0          # per vx bewegt
    assert sys._vys[0] > 0.0         # Gravity hat vy erhoeht


def test_emit_handles_inverted_size_minmax():
    """Review-Fund: random.randint(a, b) verlangt a <= b. Direktes Setzen
    von size_min > size_max (am Editor vorbei) durfte nicht abstuerzen."""
    sys = ParticleSystem(0.0, 0.0)
    sys.size_min, sys.size_max = 8, 2   # vertauscht
    sys.emit(20)   # darf nicht werfen
    assert sys.count() == 20
    assert all(2 <= int(s) <= 8 for s in sys._sizes)


def test_emit_handles_inverted_lifetime_minmax():
    sys = ParticleSystem(0.0, 0.0)
    sys.lifetime_min, sys.lifetime_max = 900, 100   # vertauscht
    sys.emit(20)   # darf nicht werfen
    assert sys.count() == 20
    assert all(100 <= int(t) <= 900 for t in sys._lifetimes)


def test_emit_handles_inverted_velocity_minmax():
    """vx/vy nutzen random.uniform() (kein a<=b-Zwang) -- nur zur
    Vollstaendigkeit, dass auch das nicht crasht."""
    sys = ParticleSystem(0.0, 0.0)
    sys.vx_min, sys.vx_max = 50.0, -50.0
    sys.emit(10)
    assert sys.count() == 10
