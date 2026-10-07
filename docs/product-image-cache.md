# Product image cache

Native device rows and the home screen load catalog art through
`cache_product_image`. The backend stores PNG/JPEG/WebP files in
`AppCache/product-images`; Tauri's asset protocol serves these files directly.
Cached hits work offline and do not issue HTTP requests. Plain browser previews
continue to use the original CDN URL.

The cache key is the SHA-256 of the complete URL. Updated URLs use new files.
The cache holds up to 64 MiB of completed images, with an 8 MiB limit per image;
older downloaded files are removed when adding an image would exceed the budget.
Two downloads may run at once, with a 15-second request timeout. Cached reads
do not wait for download slots. Downloads accept only exact HTTPS image URLs
from the embedded catalog; redirects are disabled and response signatures are
checked before atomic publication. Invalid cache files are removed and retried.

The frontend shares in-flight requests and resolved asset URLs between views.
Repeated failures back off for a minute. A default image stays visible while a
first download runs; an image from the previously selected model is never shown
as the new model's photo. Startup warms only the remembered device's image and
does not download the entire catalog. Visible scan results load their own art.
Device rows use the catalog thumbnail when available; the home hero uses the
large product image. Explicit per-profile image overrides remain authoritative.

The first uncached image still depends on CDN speed. Files persist across app
restarts and do not need to be fetched again until removed by cache eviction or
the operating system. This cache does not store personal device information.

Validation includes deduplicated frontend requests, offline cache hits, corrupt
file recovery, image format/catalog constraints and bounded disk eviction.
Native visual performance must still be verified in a running desktop build.
