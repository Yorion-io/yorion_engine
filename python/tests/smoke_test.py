"""Smoke test for an installed yorion-engine wheel. Run with: python python/tests/smoke_test.py"""

import yorion_engine as yorion

engine = yorion.create_engine()
day = yorion.GregorianDate(year=2024, month=4, day=13)

bs = engine.gregorian_to_bs(day)
assert (bs.year, bs.month, bs.day) == (2081, 1, 1), bs

back = engine.bs_to_gregorian(bs)
assert (back.year, back.month, back.day) == (2024, 4, 13), back

tithi = engine.get_tithi(day)
assert tithi.number == 5, tithi

kathmandu = yorion.Location(
    latitude=27.7172, longitude=85.324, name="Kathmandu",
    timezone_offset_mins=345, follow_nepal_social_calendar=True,
)
info = engine.get_daily_astro_info(day, kathmandu)
assert info.tithi.number == 5, info

print("yorion-engine smoke test passed:", bs, tithi)
