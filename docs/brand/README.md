# Brand

The canonical home for Auvora brand assets ([ADR-0113](../adr/0113-brand-assets-location.md)). Other surfaces (README, docs site, hosted UI, console, mobile app) reference or copy from here. Don't commit duplicates elsewhere.

| File | Description | Produced from |
|---|---|---|
| [`logo.jpeg`](logo.jpeg) | **Master.** Owner-supplied original logo, 1600×1200, light background | Original, unmodified |
| [`logo-mark.jpeg`](logo-mark.jpeg) | Tightly cropped mark, 640×600, used in the root README header | `sips --cropToHeightWidth 600 640 --cropOffset 244 480 logo.jpeg` |

Wanted variants: an SVG master, and a transparent PNG for dark backgrounds.

## Usage terms

The Auvora name and logo are trademarks of Kamaal Khan. They are **not** licensed under the AGPL-3.0 or Apache-2.0 terms that cover the code in this repository. You may use them unmodified to refer to the Auvora project. Don't use them in a way that suggests endorsement, or as the branding of a fork or derivative product.
