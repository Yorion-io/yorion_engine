# yorion-engine

Nepali (Bikram Sambat) calendar engine for Python: BS ↔ AD conversion, panchanga (tithi, nakshatra, yoga, karana, sunrise/sunset) and festival recurrence. It wraps the [YorionEngine](https://github.com/Yorion-io/yorion_engine) Rust library, so it is fast, works offline and gives the same answers as the JavaScript, Swift and Kotlin packages.

```bash
pip install yorion-engine
```

```python
import yorion_engine as yorion

engine = yorion.create_engine()
day = yorion.GregorianDate(year=2024, month=4, day=13)

print(engine.gregorian_to_bs(day))
# BsDate(year=2081, month=1, day=1)

print(engine.get_tithi(day))
# Tithi(number=5, name=Shukla Panchami, paksha=Paksha.SHUKLA)

kathmandu = yorion.Location(
    latitude=27.7172, longitude=85.324, name="Kathmandu",
    timezone_offset_mins=345, follow_nepal_social_calendar=True,
)
print(engine.get_daily_astro_info(day, kathmandu))
```

Prebuilt wheels are published for Linux (x86_64, aarch64), macOS (Intel, Apple silicon) and Windows (x64), and work on every Python version from 3.9 up.

## Accuracy

- BS ↔ AD conversion covers BS 1975–2100 (AD 1918–2044).
- Tithi values are corrected against the official Nepali Panchanga almanac **through BS 2083**. Later years are raw astronomical output.

## More

- [Festival recurrence rules (BS-RRULE spec)](https://github.com/Yorion-io/yorion_engine/blob/main/docs/bs-rrule-spec.md)
- [Changelog](https://github.com/Yorion-io/yorion_engine/blob/main/CHANGELOG.md)

License: MIT OR Apache-2.0
