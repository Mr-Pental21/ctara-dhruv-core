# Node.js Upagraha Configuration

Node accepts an `upagrahaConfig` object with numeric values:

- `gulikaPoint`
- `maandiPoint`
- `otherPoint`
- `gulikaPlanet`
- `maandiPlanet`

Value mapping:

- points: `0=start`, `1=middle`, `2=end`
- planets: `0=rahu`, `1=saturn`

```js
const upagrahaConfig = {
  gulikaPoint: 1,
  gulikaPlanet: 1,
  maandiPoint: 2,
  maandiPlanet: 0,
  otherPoint: 0,
};

const upagrahas = jyotish.allUpagrahasForDate(
  engine,
  eop,
  utc,
  location,
  0,
  false,
  upagrahaConfig,
);

const jd = extras.timeUpagrahaJd(
  0,
  2,
  false,
  sunriseJd,
  sunsetJd,
  nextSunriseJd,
  upagrahaConfig,
);
```
