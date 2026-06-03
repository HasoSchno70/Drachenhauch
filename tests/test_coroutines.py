"""Coroutines / YIELD -- alle drei Pfade bit-identisch.

Thread-basierte Coroutinen (Worker-Thread pro Coroutine, striktes Ping-Pong).
Jeder Test laeuft via run_all durch Tree-Walker, Python-VM und Cython-VM und
prueft identische Ausgabe.
"""


def test_basic_yield_sequence(run_all):
    assert run_all('''
        FUNCTION zaehler() AS INTEGER
            YIELD 1
            YIELD 2
            YIELD 3
        END FUNCTION

        DIM c AS COROUTINE
        c = zaehler()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
    ''') == "1\n2\n3\n"


def test_yield_send_expression(run_all):
    # `x = YIELD v` evaluiert zum via CORO_SEND uebergebenen Wert.
    assert run_all('''
        FUNCTION echo() AS INTEGER
            DIM r AS INTEGER
            r = YIELD 10
            PRINT "got " + STR$(r)
            YIELD r * 2
        END FUNCTION

        DIM e AS COROUTINE
        e = echo()
        PRINT CORO_RESUME(e)
        PRINT CORO_SEND(e, 7)
    ''') == "10\ngot 7\n14\n"


def test_return_value_via_result(run_all):
    assert run_all('''
        FUNCTION g() AS INTEGER
            YIELD 1
            RETURN 99
        END FUNCTION

        DIM c AS COROUTINE
        c = g()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
        PRINT CORO_DONE(c)
        PRINT CORO_RESULT(c)
    ''') == "1\n99\nTRUE\n99\n"


def test_for_each_drain(run_all):
    # FOR EACH konsumiert die Coroutine eager bis zum Ende (RETURN-Wert nicht
    # enthalten).
    assert run_all('''
        FUNCTION nums() AS INTEGER
            YIELD 5
            YIELD 10
            YIELD 15
            RETURN 0
        END FUNCTION

        DIM total AS INTEGER
        total = 0
        FOR EACH v IN nums()
            total = total + v
        NEXT
        PRINT total
    ''') == "30\n"


def test_comprehension_over_coroutine(run_all):
    assert run_all('''
        FUNCTION nums() AS INTEGER
            YIELD 1
            YIELD 2
            YIELD 3
        END FUNCTION

        DIM sq AS TUPLE
        sq = [n * n FOR n IN nums()]
        PRINT sq
    ''') == "(1, 4, 9)\n"


def test_sub_coroutine_no_return_type(run_all):
    # Eine SUB-Coroutine yieldet ohne Typ-Coercion; endet ohne RETURN-Wert.
    assert run_all('''
        SUB ticker()
            YIELD 1
            YIELD 2
        END SUB

        DIM c AS COROUTINE
        c = ticker()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
        PRINT CORO_DONE(c)
    ''') == "1\n2\nNIL\nTRUE\n"


def test_helper_coroutine_no_cross_frame(run_all):
    # Ein Helfer mit YIELD ist selbst eine Coroutine -- der Aufruf liefert ein
    # Handle, fuehrt NICHT inline aus. YIELD ueberquert also nie einen Call.
    assert run_all('''
        FUNCTION inner() AS INTEGER
            YIELD 100
            YIELD 200
        END FUNCTION

        FUNCTION outer() AS INTEGER
            DIM ic AS COROUTINE
            ic = inner()
            YIELD CORO_RESUME(ic)
            YIELD CORO_RESUME(ic)
        END FUNCTION

        DIM c AS COROUTINE
        c = outer()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
    ''') == "100\n200\n"


def test_error_propagates_through_resume(run_all):
    # THROW im Coroutine-Body wird beim CORO_RESUME auf dem aufrufenden Thread
    # re-raised -> TRY/CATCH greift.
    assert run_all('''
        FUNCTION bad() AS INTEGER
            YIELD 1
            THROW "kaputt"
        END FUNCTION

        DIM c AS COROUTINE
        c = bad()
        PRINT CORO_RESUME(c)
        TRY
            PRINT CORO_RESUME(c)
        CATCH msg
            PRINT "fehler: " + msg
        END TRY
    ''') == "1\nfehler: kaputt\n"


def test_close_and_idempotent(run_all):
    assert run_all('''
        SUB endless()
            DIM i AS INTEGER
            i = 0
            WHILE TRUE
                YIELD i
                i = i + 1
            WEND
        END SUB

        DIM c AS COROUTINE
        c = endless()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
        CORO_CLOSE(c)
        PRINT CORO_DONE(c)
        CORO_CLOSE(c)
        PRINT CORO_DONE(c)
    ''') == "0\n1\nTRUE\nTRUE\n"


def test_coroutine_type_and_print(run_all):
    assert run_all('''
        FUNCTION g() AS INTEGER
            YIELD 1
        END FUNCTION

        DIM c AS COROUTINE
        c = g()
        PRINT c
    ''') == "<COROUTINE g>\n"


def test_method_coroutine_with_self(run_all):
    # Methode als Coroutine; Zugriff auf Self-Felder im Body.
    assert run_all('''
        CLASS Gen
            DIM base AS INTEGER
            SUB stream()
                YIELD Self.base + 1
                YIELD Self.base + 2
            END SUB
        END CLASS

        DIM g AS Gen
        g = NEW Gen()
        g.base = 100
        DIM c AS COROUTINE
        c = g.stream()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
    ''') == "101\n102\n"


def test_interleaved_coroutines_deterministic(run_all):
    # Zwei Coroutinen abwechselnd treiben -- Reihenfolge muss in allen drei
    # Pfaden gleich sein.
    assert run_all('''
        FUNCTION a() AS STRING
            YIELD "a1"
            YIELD "a2"
        END FUNCTION
        FUNCTION b() AS STRING
            YIELD "b1"
            YIELD "b2"
        END FUNCTION

        DIM ca AS COROUTINE
        DIM cb AS COROUTINE
        ca = a()
        cb = b()
        PRINT CORO_RESUME(ca)
        PRINT CORO_RESUME(cb)
        PRINT CORO_RESUME(ca)
        PRINT CORO_RESUME(cb)
    ''') == "a1\nb1\na2\nb2\n"


def test_yield_coerces_to_return_type(run_all):
    # FUNCTION ... AS FLOAT: YIELD-Werte werden auf FLOAT gecoerct.
    assert run_all('''
        FUNCTION halves() AS FLOAT
            YIELD 1
            YIELD 2
        END FUNCTION

        DIM c AS COROUTINE
        c = halves()
        PRINT CORO_RESUME(c)
        PRINT CORO_RESUME(c)
    ''') == "1.0\n2.0\n"
