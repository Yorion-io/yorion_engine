# @yorion/engine

Nepali (Bikram Sambat) calendar engine for JavaScript and TypeScript: BS ↔ AD conversion, panchanga (tithi, nakshatra, yoga, karana, sunrise/sunset) and festival recurrence. It is the [YorionEngine](https://github.com/Yorion-io/yorion_engine) Rust library compiled to WebAssembly, so it runs offline in Node.js, bundlers and the browser.

```bash
npm install @yorion/engine
```

## Node.js and bundlers (Vite, webpack, Next.js, Rollup)

```js
import { gregorian_to_bs, bs_to_gregorian, get_daily_astro_info_with_location, get_tithi_name, Location, Language }
  from '@yorion/engine';

const bs = gregorian_to_bs(2024, 4, 13);
console.log(bs.year, bs.month, bs.day);          // 2081 1 1

console.log(bs_to_gregorian(2081, 1, 1));         // "2024-04-13"

const kathmandu = new Location(27.7172, 85.324, 'Kathmandu', 345); // UTC+5:45
const info = get_daily_astro_info_with_location(2024, 4, 13, kathmandu);
console.log(get_tithi_name(info.tithi, Language.Nepali), info.sunrise, info.sunset);
```

CommonJS works too: `const { gregorian_to_bs } = require('@yorion/engine');`

## Browser without a bundler

```html
<script type="module">
  import init, { gregorian_to_bs } from 'https://cdn.jsdelivr.net/npm/@yorion/engine/web/yorion_engine.js';
  await init();
  console.log(gregorian_to_bs(2024, 4, 13));
</script>
```

The `@yorion/engine/web` entry point needs `await init()` once before any other call.

## Accuracy

- BS ↔ AD conversion covers BS 1975–2100 (AD 1918–2044).
- Tithi values are corrected against the official Nepali Panchanga almanac **through BS 2083** (`tithi_verified_through_bs()`). Later years are raw astronomical output.

## More

- [Festival recurrence rules (BS-RRULE spec)](https://github.com/Yorion-io/yorion_engine/blob/main/docs/bs-rrule-spec.md)
- [Full WASM API guide](https://github.com/Yorion-io/yorion_engine/blob/main/docs/consuming-wasm.md)
- [Changelog](https://github.com/Yorion-io/yorion_engine/blob/main/CHANGELOG.md)

License: MIT OR Apache-2.0
